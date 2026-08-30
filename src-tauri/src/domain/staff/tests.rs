//! Staff / session tests (temporary / in-memory DB only).

use crate::db::Db;
use crate::domain::ids::{new_entity_id, parse_entity_id};
use crate::domain::staff::{
    change_staff_role, create_staff, deactivate_staff, get_staff, sign_in_staff, StaffInput,
    StaffRole,
};
use crate::domain::team::{create_team, CreateTeamInput};
use crate::error::AppError;

fn staff_input(name: &str, pin: &str, role: Option<StaffRole>) -> StaffInput {
    StaffInput {
        name: name.into(),
        pin: pin.into(),
        role,
    }
}

fn solo_admin(db: &Db) -> crate::domain::staff::Staff {
    let created = create_team(
        db.conn(),
        CreateTeamInput {
            name: "Shop".into(),
            member_name: "Ada".into(),
        },
    )
    .expect("team");
    assert_eq!(created.session.staff.role, StaffRole::Admin);
    get_staff(db.conn(), &created.session.staff.id.clone()).expect("admin")
}

#[test]
fn uuid_create_and_get() {
    let db = Db::open_in_memory().expect("db");
    let created = create_staff(db.conn(), staff_input("Ada", "1234", None)).expect("create");
    parse_entity_id(&created.id.clone()).expect("v7");
    let fetched = get_staff(db.conn(), &created.id.clone()).expect("get");
    assert_eq!(fetched.id.clone(), created.id.clone());
    assert_eq!(fetched.name, "Ada");
    assert_eq!(fetched.role, StaffRole::Staff);
}

#[test]
fn wrong_pin_is_unauthorized() {
    let db = Db::open_in_memory().expect("db");
    let created = create_staff(db.conn(), staff_input("Ada", "1234", None)).expect("create");
    let err = sign_in_staff(db.conn(), &created.id.clone(), "9999").expect_err("pin");
    assert!(matches!(err, AppError::Unauthorized { .. }));
    assert_eq!(err.code(), "unauthorized");
    assert_eq!(err.user_message(), "PIN or staff is not valid.");

    let missing = new_entity_id();
    let err = sign_in_staff(db.conn(), &missing, "1234").expect_err("unknown");
    assert_eq!(err.user_message(), "PIN or staff is not valid.");
}

#[test]
fn last_admin_cannot_be_demoted() {
    let db = Db::open_in_memory().expect("db");
    let admin = solo_admin(&db);
    let err =
        change_staff_role(db.conn(), &admin.id.clone(), StaffRole::Staff).expect_err("last admin");
    assert!(matches!(err, AppError::Forbidden { .. }));
    assert_eq!(err.code(), "forbidden");
}

#[test]
fn last_admin_cannot_be_deactivated() {
    let db = Db::open_in_memory().expect("db");
    let admin = solo_admin(&db);
    let err = deactivate_staff(db.conn(), &admin.id.clone()).expect_err("last admin");
    assert!(matches!(err, AppError::Forbidden { .. }));
}

#[test]
fn non_admin_cannot_change_role() {
    let db = Db::open_in_memory().expect("db");
    let admin = solo_admin(&db);
    let other = create_staff(
        db.conn(),
        staff_input("Bob", "5678", Some(StaffRole::Staff)),
    )
    .expect("other");
    sign_in_staff(db.conn(), &other.id.clone(), "5678").expect("sign in other");
    let err =
        change_staff_role(db.conn(), &admin.id.clone(), StaffRole::Staff).expect_err("forbidden");
    assert!(matches!(err, AppError::Forbidden { .. }));
    assert_eq!(err.code(), "forbidden");
}
