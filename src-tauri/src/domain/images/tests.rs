//! Repair image domain tests (tempfile AppData only).

use std::path::PathBuf;

use image::{ImageBuffer, Rgb, Rgba};

use crate::db::Db;
use crate::domain::companies::{CompanyInput, create_company};
use crate::domain::customers::{CustomerInput, create_customer};
use crate::domain::devices::{DeviceInput, create_device};
use crate::domain::images::constants::MAX_IMAGES_PER_REPAIR;
use crate::domain::images::service::resolve_safe_absolute;
use crate::domain::images::types::{
    AttachRepairImagesInput, ImageVariant, UpdateRepairImageInput,
};
use crate::domain::images::{
    attach_repair_images, delete_repair_image, list_repair_images, resolve_repair_image_path,
    update_repair_image,
};
use crate::domain::repairs::{RepairInput, create_repair};
use crate::error::AppError;

fn open_temp_db() -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = Db::open_temp(dir.path().to_path_buf()).expect("db");
    (dir, db)
}


fn company(db: &Db) -> i64 {
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

fn seed_repair(db: &Db) -> i64 {
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
            customer_id,
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
    create_repair(
        db.conn(),
        RepairInput {
            customer_id,
            device_id,
            company_id,
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
    .id
}

fn write_test_png(path: &std::path::Path) {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(64, 48, |x, y| {
        Rgba([
            (x % 255) as u8,
            (y % 255) as u8,
            120,
            255,
        ])
    });
    img.save(path).expect("png");
}

fn write_test_jpeg(path: &std::path::Path) {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_fn(80, 60, |x, y| Rgb([(x % 255) as u8, (y % 255) as u8, 90]));
    img.save(path).expect("jpeg");
}

#[test]
fn attach_valid_jpeg_and_png() {
    let (_dir, db) = open_temp_db();
    let repair_id = seed_repair(&db);

    let png = db.paths().root.join("sample.png");
    let jpeg = db.paths().root.join("sample.jpg");
    write_test_png(&png);
    write_test_jpeg(&jpeg);

    let attached = attach_repair_images(
        &db,
        AttachRepairImagesInput {
            repair_id,
            source_paths: vec![
                png.to_string_lossy().into_owned(),
                jpeg.to_string_lossy().into_owned(),
            ],
        },
    )
    .expect("attach");

    assert_eq!(attached.len(), 2);
    assert!(db.paths().root.join(&attached[0].original_path).is_file());
    assert!(
        attached[0]
            .thumb_path
            .as_ref()
            .map(|p| db.paths().root.join(p).is_file())
            .unwrap_or(false)
    );

    let listed = list_repair_images(&db, repair_id).expect("list");
    assert_eq!(listed.len(), 2);
    assert!(listed[0].sort_order <= listed[1].sort_order);
}

#[test]
fn rejects_bad_extension_and_oversize() {
    let (_dir, db) = open_temp_db();
    let repair_id = seed_repair(&db);

    let gif = db.paths().root.join("nope.gif");
    std::fs::write(&gif, b"GIF89a").expect("gif");
    let err = attach_repair_images(
        &db,
        AttachRepairImagesInput {
            repair_id,
            source_paths: vec![gif.to_string_lossy().into_owned()],
        },
    )
    .expect_err("gif");
    assert!(matches!(err, AppError::Validation { .. }));

    let big = db.paths().root.join("big.jpg");
    write_test_jpeg(&big);
    // Inflate past 10 MB while keeping .jpg extension.
    let mut data = std::fs::read(&big).expect("read");
    data.resize(10 * 1024 * 1024 + 1, 0);
    std::fs::write(&big, &data).expect("write");
    let err = attach_repair_images(
        &db,
        AttachRepairImagesInput {
            repair_id,
            source_paths: vec![big.to_string_lossy().into_owned()],
        },
    )
    .expect_err("oversize");
    assert!(matches!(err, AppError::Validation { .. }));
}

#[test]
fn rejects_over_thirty_images() {
    let (_dir, db) = open_temp_db();
    let repair_id = seed_repair(&db);
    let png = db.paths().root.join("one.png");
    write_test_png(&png);
    let path = png.to_string_lossy().into_owned();

    for _ in 0..MAX_IMAGES_PER_REPAIR {
        attach_repair_images(
            &db,
            AttachRepairImagesInput {
                repair_id,
                source_paths: vec![path.clone()],
            },
        )
        .expect("attach");
    }

    let err = attach_repair_images(
        &db,
        AttachRepairImagesInput {
            repair_id,
            source_paths: vec![path],
        },
    )
    .expect_err("limit");
    assert!(matches!(err, AppError::Validation { .. }));
}

#[test]
fn update_caption_and_delete_removes_files() {
    let (_dir, db) = open_temp_db();
    let repair_id = seed_repair(&db);
    let png = db.paths().root.join("cap.png");
    write_test_png(&png);

    let attached = attach_repair_images(
        &db,
        AttachRepairImagesInput {
            repair_id,
            source_paths: vec![png.to_string_lossy().into_owned()],
        },
    )
    .expect("attach");
    let id = attached[0].id;
    let original = db.paths().root.join(&attached[0].original_path);
    let thumb = db
        .paths()
        .root
        .join(attached[0].thumb_path.as_ref().expect("thumb"));

    let updated = update_repair_image(
        &db,
        id,
        UpdateRepairImageInput {
            caption: Some(Some("Front cracked".into())),
            sort_order: Some(5),
        },
    )
    .expect("update");
    assert_eq!(updated.caption.as_deref(), Some("Front cracked"));
    assert_eq!(updated.sort_order, 5);

    delete_repair_image(&db, id).expect("delete");
    assert!(!original.exists());
    assert!(!thumb.exists());
    assert!(list_repair_images(&db, repair_id).expect("list").is_empty());
}

#[test]
fn resolve_rejects_path_escape() {
    let (_dir, db) = open_temp_db();
    let repair_id = seed_repair(&db);
    let png = db.paths().root.join("ok.png");
    write_test_png(&png);
    let attached = attach_repair_images(
        &db,
        AttachRepairImagesInput {
            repair_id,
            source_paths: vec![png.to_string_lossy().into_owned()],
        },
    )
    .expect("attach");

    let resolved = resolve_repair_image_path(&db, attached[0].id, ImageVariant::Original)
        .expect("resolve");
    assert!(PathBuf::from(&resolved.absolute_path).is_file());

    let err = resolve_safe_absolute(db.paths(), "images/../database.sqlite").expect_err("escape");
    assert!(matches!(err, AppError::Validation { .. }));

    let err = resolve_safe_absolute(db.paths(), "logs/secret.txt").expect_err("outside");
    assert!(matches!(err, AppError::Validation { .. }));
}
