//! Backup domain tests (tempfile AppData only).

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use image::{ImageBuffer, Rgba};
use zip::ZipArchive;

use crate::db::Db;
use crate::domain::backup::constants::MANIFEST_NAME;
use crate::domain::backup::types::{BackupManifest, CreateBackupInput};
use crate::domain::backup::{
    create_backup, list_local_backups, restore_backup, run_auto_backup_if_due, validate_backup,
};
use crate::domain::companies::{create_company, CompanyInput};
use crate::domain::customers::{create_customer, CustomerInput};
use crate::domain::devices::{create_device, DeviceInput};
use crate::domain::images::types::AttachRepairImagesInput;
use crate::domain::images::{attach_repair_images, list_repair_images};
use crate::domain::repairs::{create_repair, RepairInput};

fn open_temp_db() -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = Db::open_temp(dir.path().to_path_buf()).expect("db");
    (dir, db)
}

fn company(db: &Db) -> String {
    create_company(
        db.conn(),
        CompanyInput {
            legal_name: "Test Company".into(),
            trade_name: None,
            tax_id: None,
            address: None,
            phone: None,
            email: None,
            website: None,
        },
    )
    .expect("company")
    .id
}

fn seed_repair_with_image(db: &Db) -> String {
    let company_id = company(db);
    let customer_id = create_customer(
        db.conn(),
        CustomerInput {
            name: "Owner".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .expect("customer")
    .id;
    let device_id = create_device(
        db.conn(),
        DeviceInput {
            customer_id: customer_id.to_string(),
            device_type: Some("Phone".into()),
            manufacturer: Some("Acme".into()),
            model: Some("X1".into()),
            serial_number: Some("SN-1".into()),
            accessories: None,
            notes: None,
        },
    )
    .expect("device")
    .id;
    let repair_id = create_repair(
        db.conn(),
        RepairInput {
            customer_id: customer_id.to_string(),
            device_id: device_id.to_string(),
            company_id: company_id.to_string(),
            status: None,
            reported_problem: Some("Broken".into()),
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
        },
    )
    .expect("repair")
    .id;

    let png = db.paths().root.join("cam.png");
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(32, 32, Rgba([10, 20, 30, 255]));
    img.save(&png).expect("png");
    attach_repair_images(
        db,
        AttachRepairImagesInput {
            repair_id: repair_id.clone(),
            source_paths: vec![png.to_string_lossy().into_owned()],
        },
    )
    .expect("attach");
    repair_id
}

#[test]
fn create_backup_contains_manifest_and_db() {
    let (_dir, db) = open_temp_db();
    let _ = seed_repair_with_image(&db);

    let info = create_backup(
        &db,
        CreateBackupInput {
            destination_path: None,
        },
    )
    .expect("backup");

    assert!(Path::new(&info.path).is_file());
    assert!(info.file_name.starts_with("Servioo-"));
    assert!(info.file_name.ends_with(".backup"));

    let file = File::open(&info.path).expect("open");
    let mut archive = ZipArchive::new(file).expect("zip");
    let mut manifest_file = archive.by_name(MANIFEST_NAME).expect("manifest");
    let mut raw = String::new();
    manifest_file.read_to_string(&mut raw).expect("read");
    drop(manifest_file);
    let manifest: BackupManifest = serde_json::from_str(&raw).expect("parse");
    assert_eq!(manifest.app_version, env!("CARGO_PKG_VERSION"));
    assert!(manifest.files.iter().any(|f| f.path == "database.sqlite"));
    assert!(archive.by_name("database.sqlite").is_ok());

    let validation = validate_backup(Path::new(&info.path)).expect("validate");
    assert!(validation.valid, "{:?}", validation.errors);
}

#[test]
fn corrupt_checksum_is_invalid() {
    let (_dir, db) = open_temp_db();
    let _ = seed_repair_with_image(&db);
    let info = create_backup(
        &db,
        CreateBackupInput {
            destination_path: None,
        },
    )
    .expect("backup");

    // Rewrite manifest with a bad checksum.
    let file = File::open(&info.path).expect("open");
    let mut archive = ZipArchive::new(file).expect("zip");
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).expect("entry");
        let name = entry.name().to_string();
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).expect("read");
        entries.push((name, buf));
    }
    drop(archive);

    for (name, buf) in &mut entries {
        if name == MANIFEST_NAME {
            let mut manifest: BackupManifest = serde_json::from_slice(buf).expect("manifest");
            if let Some(first) = manifest.files.first_mut() {
                first.sha256 = "0".repeat(64);
            }
            *buf = serde_json::to_vec_pretty(&manifest).expect("ser");
        }
    }

    let corrupt_path = db.paths().backups.join("corrupt.backup");
    {
        let out = File::create(&corrupt_path).expect("create");
        let mut zip = zip::ZipWriter::new(out);
        let options = zip::write::SimpleFileOptions::default();
        for (name, buf) in &entries {
            zip.start_file(name.as_str(), options).expect("start");
            zip.write_all(buf).expect("write");
        }
        zip.finish().expect("finish");
    }

    let validation = validate_backup(&corrupt_path).expect("validate");
    assert!(!validation.valid);
    assert!(validation
        .errors
        .iter()
        .any(|e| e.to_lowercase().contains("checksum")));
}

#[test]
fn restore_creates_safety_and_reopens_db() {
    let (dir, mut db) = open_temp_db();
    let repair_id = seed_repair_with_image(&db);
    assert_eq!(
        list_repair_images(&db, repair_id.clone())
            .expect("list")
            .len(),
        1
    );

    let backup = create_backup(
        &db,
        CreateBackupInput {
            destination_path: None,
        },
    )
    .expect("backup");

    // Mutate live DB so restore is observable.
    create_customer(
        db.conn(),
        CustomerInput {
            name: "After Backup".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .expect("extra customer");

    let result = restore_backup(&mut db, Path::new(&backup.path)).expect("restore");
    assert!(Path::new(&result.safety_backup_path).is_file());
    assert!(result.safety_backup_path.contains("-safety-"));

    // DB is usable after reopen.
    let health = db.health_check().expect("health");
    assert!(health.migrations_applied >= 1);
    assert_eq!(health.customers, 1);

    let images = list_repair_images(&db, repair_id).expect("images");
    assert_eq!(images.len(), 1);
    assert!(dir.path().join(&images[0].original_path).is_file());

    let listed = list_local_backups(db.paths()).expect("list");
    assert!(listed.items.iter().any(|b| b.path == backup.path));
    assert!(listed
        .items
        .iter()
        .any(|b| b.path == result.safety_backup_path));
}

#[test]
fn auto_backup_skips_same_day_second_run() {
    let (_dir, db) = open_temp_db();
    let _ = seed_repair_with_image(&db);

    let first = run_auto_backup_if_due(&db).expect("first");
    assert!(first.ran);
    assert!(first.backup.is_some());

    let second = run_auto_backup_if_due(&db).expect("second");
    assert!(!second.ran);
    assert!(second.backup.is_none());
}
