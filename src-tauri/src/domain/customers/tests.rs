//! Customer domain integration tests (temporary / in-memory DB only).

use crate::db::Db;
use crate::domain::customers::{
    CustomerInput, CustomerListQuery, archive_customer, create_customer, get_customer,
    list_customers, unarchive_customer, update_customer,
};

fn sample(name: &str) -> CustomerInput {
    CustomerInput {
        name: name.into(),
        phone: Some("+491234".into()),
        email: Some("a@b.co".into()),
        address: Some("Street 1".into()),
        notes: None,
    }
}

#[test]
fn create_list_get_update_archive_flow() {
    let db = Db::open_in_memory().expect("db");
    let created = create_customer(db.conn(), sample("Ada Lovelace")).expect("create");
    assert!(created.id > 0);

    let listed = list_customers(
        db.conn(),
        CustomerListQuery {
            query: Some("ada".into()),
            include_archived: Some(false),
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("list");
    assert_eq!(listed.total, 1);
    assert_eq!(listed.items[0].name, "Ada Lovelace");

    let updated = update_customer(
        db.conn(),
        created.id,
        CustomerInput {
            name: "Ada L.".into(),
            phone: None,
            email: Some("ada@example.com".into()),
            address: None,
            notes: Some("VIP".into()),
        },
    )
    .expect("update");
    assert_eq!(updated.name, "Ada L.");
    assert_eq!(updated.notes.as_deref(), Some("VIP"));

    let archived = archive_customer(db.conn(), created.id).expect("archive");
    assert!(archived.archived_at.is_some());

    let active = list_customers(
        db.conn(),
        CustomerListQuery {
            query: None,
            include_archived: Some(false),
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("active");
    assert_eq!(active.total, 0);

    let with_archived = list_customers(
        db.conn(),
        CustomerListQuery {
            query: None,
            include_archived: Some(true),
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("all");
    assert_eq!(with_archived.total, 1);

    let restored = unarchive_customer(db.conn(), created.id).expect("unarchive");
    assert!(restored.archived_at.is_none());
    assert_eq!(get_customer(db.conn(), created.id).expect("get").name, "Ada L.");
}

#[test]
fn rejects_invalid_email() {
    let db = Db::open_in_memory().expect("db");
    let err = create_customer(
        db.conn(),
        CustomerInput {
            name: "Test".into(),
            phone: None,
            email: Some("not-an-email".into()),
            address: None,
            notes: None,
        },
    )
    .expect_err("bad email");
    assert_eq!(err.code(), "validation");
}
