//! Team PIN / join tests (temporary / in-memory DB only).

use crate::db::Db;
use crate::domain::ids::new_entity_id;
use crate::domain::staff::StaffRole;
use crate::domain::sync::apply_snapshot_row;
use crate::domain::sync::snapshot::dump_snapshot;
use crate::domain::team::pin::hash_team_pin;
use crate::domain::team::{
    apply_join_grant, create_team, get_team_pin, leave_team, list_nearby_teams, list_team_members,
    prepare_join, CreateTeamInput, JoinGrant, JoinTeamInput, NearbyTeam,
};
use crate::error::AppError;
use std::collections::HashSet;

fn shop_team(db: &Db) -> crate::domain::team::CreateTeamResult {
    create_team(
        db.conn(),
        CreateTeamInput {
            name: "Shop".into(),
            member_name: "Ada".into(),
        },
    )
    .expect("team")
}

#[test]
fn create_team_returns_pin_and_session() {
    let db = Db::open_in_memory().expect("db");
    let result = shop_team(&db);
    assert_eq!(result.pin.len(), 6);
    assert!(result.pin.chars().all(|c| c.is_ascii_digit()));
    assert_eq!(result.session.staff.name, "Ada");
    assert_eq!(result.session.staff.role, StaffRole::Admin);

    let hash: Option<String> = db
        .conn()
        .query_row("SELECT pin_hash FROM teams LIMIT 1", [], |row| row.get(0))
        .expect("hash");
    let hash = hash.expect("pin_hash stored");
    assert_eq!(hash, hash_team_pin(&result.pin));
    assert_ne!(hash, result.pin);

    let stored: Option<String> = db
        .conn()
        .query_row(
            "SELECT team_pin FROM local_identity WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("local pin");
    assert_eq!(stored.as_deref(), Some(result.pin.as_str()));

    let name_on_team: String = db
        .conn()
        .query_row("SELECT name FROM teams LIMIT 1", [], |row| row.get(0))
        .expect("name");
    assert_eq!(name_on_team, "Shop");
}

#[test]
fn join_validation_rejects_bad_pin() {
    let db = Db::open_in_memory().expect("db");
    let err = prepare_join(
        db.conn(),
        &JoinTeamInput {
            team_id: new_entity_id(),
            pin: "12".into(),
            member_name: "Bob".into(),
        },
    )
    .expect_err("bad pin");
    assert!(matches!(err, AppError::Validation { .. }));
    assert_eq!(err.field(), Some("pin"));
}

#[test]
fn list_team_members_gig_count() {
    let db = Db::open_in_memory().expect("db");
    let created = shop_team(&db);
    let staff_id = created.session.staff.id.clone();
    let now = "2026-01-01T00:00:00Z";
    db.conn()
        .execute(
            "INSERT INTO customers (
                id, name, phone, email, address, notes, created_at, updated_at, archived_at,
                hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000001',
                'Cust', NULL, NULL, NULL, NULL, ?1, ?1, NULL, 0, 0, 'dev', NULL, NULL
             )",
            [now],
        )
        .expect("customer");
    db.conn()
        .execute(
            "INSERT INTO devices (
                id, customer_id, device_type, manufacturer, model, serial_number,
                accessories, notes, created_at, updated_at, archived_at,
                hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000002',
                '01900000-0000-7000-8000-000000000001',
                'laptop', 'Lenovo', 'T480', 'ABC', NULL, NULL, ?1, ?1, NULL,
                0, 0, 'dev', NULL, NULL
             )",
            [now],
        )
        .expect("device");
    db.conn()
        .execute(
            "INSERT INTO repairs (
                id, repair_number, customer_id, device_id, assigned_to_staff_id, status, received_at,
                reported_problem, accessories_received, device_condition,
                diagnosis_notes, work_performed, notes, ready_at, collected_at,
                created_at, updated_at, archived_at,
                hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000003',
                '2026-AA-000001',
                '01900000-0000-7000-8000-000000000001',
                '01900000-0000-7000-8000-000000000002',
                ?1,
                'received', '2026-01-01',
                NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
                ?2, ?2, NULL,
                0, 0, 'dev', NULL, NULL
             )",
            rusqlite::params![staff_id, now],
        )
        .expect("repair");

    let members = list_team_members(db.conn(), &HashSet::new()).expect("members");
    assert_eq!(members.items.len(), 1);
    assert_eq!(members.items[0].id, staff_id);
    assert_eq!(members.items[0].gig_count, 1);
    assert!(members.items[0].online);
}

#[test]
fn last_admin_leave_blocked_with_other_devices() {
    let db = Db::open_in_memory().expect("db");
    shop_team(&db);
    let now = "2026-01-01T00:00:00Z";
    let team_id: String = db
        .conn()
        .query_row(
            "SELECT team_id FROM local_identity WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("team");
    db.conn()
        .execute(
            "INSERT INTO team_devices (
                id, team_id, device_code, device_name, joined_at, removed_at,
                created_at, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
                updated_by_staff_id, deleted_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000099',
                ?1, 'AB', 'Other PC', ?2, NULL, ?2, ?2, 0, 0, 'dev', NULL, NULL
             )",
            rusqlite::params![team_id, now],
        )
        .expect("other device");
    let err = leave_team(db.conn()).expect_err("blocked");
    assert!(matches!(err, AppError::Forbidden { .. }));
    assert_eq!(
        err.user_message(),
        "Leave is blocked while other computers are still in the team and you are the last administrator."
    );
}

#[test]
fn nearby_list_empty_when_in_a_team() {
    let db = Db::open_in_memory().expect("db");
    shop_team(&db);
    let result = list_nearby_teams(
        db.conn(),
        vec![NearbyTeam {
            team_id: new_entity_id(),
            name: "Other Shop".into(),
        }],
    )
    .expect("nearby");
    assert!(result.items.is_empty());
}

#[test]
fn get_team_pin_returns_local_reveal() {
    let db = Db::open_in_memory().expect("db");
    let created = shop_team(&db);
    let revealed = get_team_pin(db.conn()).expect("pin");
    assert_eq!(revealed.pin, created.pin);
}

#[test]
fn apply_join_grant_uses_snapshot_team_name() {
    let host = Db::open_in_memory().expect("host");
    let created = shop_team(&host);
    let identity = crate::domain::identity::require_local_identity(host.conn()).expect("id");
    let team_id = identity.team_id.expect("team");
    let psk = identity.team_psk.expect("psk");
    let (_version, rows) = dump_snapshot(host.conn()).expect("dump");
    let teams = rows.iter().find(|row| row.table == "teams").expect("teams");

    let joiner = Db::open_in_memory().expect("joiner");
    apply_snapshot_row(joiner.conn(), "teams", teams.payload.clone()).expect("apply teams");

    let result = apply_join_grant(
        joiner.conn(),
        &JoinGrant {
            team_id,
            team_name: String::new(),
            team_psk_hex: psk,
            device_code: "AB".into(),
            snapshot_applied: true,
        },
        "Bob",
        &created.pin,
    )
    .expect("join");
    assert!(result.snapshot_applied);
    assert_eq!(result.team.name, "Shop");
    assert_eq!(result.device_code, "AB");
    assert_eq!(result.session.staff.name, "Bob");
    assert_eq!(result.session.staff.role, StaffRole::Staff);
}
