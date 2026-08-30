//! Company domain integration tests (temporary / in-memory DB only).

use image::{ImageBuffer, Rgba};

use crate::db::Db;
use crate::domain::companies::{
    archive_company, attach_company_logo, clear_company_logo, create_company, get_company,
    list_companies, resolve_company_logo_path, set_default_company, unarchive_company,
    update_company, AttachCompanyLogoInput, CompanyInput, CompanyListQuery,
};
use crate::error::AppError;

fn sample(legal_name: &str) -> CompanyInput {
    CompanyInput {
        legal_name: legal_name.into(),
        trade_name: Some("Trade".into()),
        tax_id: Some("DE123".into()),
        address: Some("Street 1".into()),
        phone: Some("+49123".into()),
        email: Some("a@b.co".into()),
        website: Some("https://example.com".into()),
    }
}

fn open_temp_db() -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = Db::open_temp(dir.path().to_path_buf()).expect("db");
    (dir, db)
}

#[test]
fn create_list_get_update_archive_flow() {
    let db = Db::open_in_memory().expect("db");
    let created = create_company(db.conn(), sample("Acme GmbH")).expect("create");
    assert!(!created.id.clone().is_empty());
    assert!(created.is_default);

    let listed = list_companies(
        db.conn(),
        CompanyListQuery {
            query: Some("acme".into()),
            include_archived: Some(false),
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("list");
    assert_eq!(listed.total, 1);
    assert_eq!(listed.items[0].legal_name, "Acme GmbH");

    let updated = update_company(
        db.conn(),
        created.id.clone(),
        CompanyInput {
            legal_name: "Acme AG".into(),
            trade_name: None,
            tax_id: Some("DE999".into()),
            address: None,
            phone: None,
            email: Some("info@acme.test".into()),
            website: None,
        },
    )
    .expect("update");
    assert_eq!(updated.legal_name, "Acme AG");
    assert_eq!(updated.tax_id.as_deref(), Some("DE999"));

    let archived = archive_company(db.conn(), created.id.clone()).expect("archive");
    assert!(archived.archived_at.is_some());
    assert!(!archived.is_default);

    let active = list_companies(
        db.conn(),
        CompanyListQuery {
            query: None,
            include_archived: Some(false),
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("active");
    assert_eq!(active.total, 0);

    let restored = unarchive_company(db.conn(), created.id.clone()).expect("unarchive");
    assert!(restored.archived_at.is_none());
    assert!(!restored.is_default);

    let _ = get_company(db.conn(), created.id.clone()).expect("get");
}

#[test]
fn first_company_is_default_second_is_not() {
    let db = Db::open_in_memory().expect("db");
    let first = create_company(db.conn(), sample("First Co")).expect("first");
    let second = create_company(db.conn(), sample("Second Co")).expect("second");
    assert!(first.is_default);
    assert!(!second.is_default);
}

#[test]
fn set_default_clears_previous_in_one_transaction() {
    let db = Db::open_in_memory().expect("db");
    let first = create_company(db.conn(), sample("First Co")).expect("first");
    let second = create_company(db.conn(), sample("Second Co")).expect("second");

    let updated = set_default_company(db.conn(), second.id.clone()).expect("set default");
    assert!(updated.is_default);
    assert!(
        !get_company(db.conn(), first.id.clone())
            .expect("first")
            .is_default
    );

    let defaults: i64 = db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE is_default = 1",
            [],
            |row| row.get(0),
        )
        .expect("count");
    assert_eq!(defaults, 1);
}

#[test]
fn archive_default_clears_is_default() {
    let db = Db::open_in_memory().expect("db");
    let company = create_company(db.conn(), sample("Default Co")).expect("create");
    assert!(company.is_default);

    let archived = archive_company(db.conn(), company.id.clone()).expect("archive");
    assert!(archived.archived_at.is_some());
    assert!(!archived.is_default);
}

#[test]
fn attach_and_resolve_logo() {
    let (_dir, db) = open_temp_db();
    let company = create_company(db.conn(), sample("Logo Co")).expect("create");
    let company_id = company.id.clone();

    let png = db.paths().root.join("logo.png");
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(32, 32, Rgba([10, 20, 30, 255]));
    img.save(&png).expect("png");

    let with_logo = attach_company_logo(
        &db,
        AttachCompanyLogoInput {
            company_id: company_id.clone(),
            source_path: png.to_string_lossy().into_owned(),
        },
    )
    .expect("attach");
    assert!(with_logo.logo_path.is_some());
    let rel = with_logo.logo_path.as_deref().expect("path");
    assert!(rel.starts_with(&format!("images/companies/{company_id}/")));

    let resolved = resolve_company_logo_path(&db, company_id.clone()).expect("resolve");
    assert!(std::path::Path::new(&resolved.absolute_path).is_file());

    let cleared = clear_company_logo(&db, company_id.clone()).expect("clear");
    assert!(cleared.logo_path.is_none());

    let err = resolve_company_logo_path(&db, company_id).expect_err("missing");
    assert!(matches!(err, AppError::NotFound));
}
