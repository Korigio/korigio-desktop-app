use std::collections::HashSet;

use rand::RngCore;
use rusqlite::Connection;
use time::{Duration, OffsetDateTime};

use crate::db::repository::now_utc_rfc3339;
use crate::domain::identity;
use crate::domain::ids::{new_entity_id, parse_entity_id, parse_entity_id_field};
use crate::domain::staff::repository as staff_repo;
use crate::domain::staff::{self, StaffRole};
use crate::domain::sync::{self, begin_write, tick_hlc_value, WriteContext};
use crate::domain::team::constants::{DEFAULT_INVITE_HOURS, MAX_INVITE_HOURS};
use crate::domain::team::invite::{generate_invite_code, hash_invite_code, normalize_invite_code};
use crate::domain::team::pin::{generate_team_pin, hash_team_pin, validate_team_pin};
use crate::domain::team::repository;
use crate::domain::team::types::{
    CreateTeamInput, CreateTeamResult, JoinTeamInput, JoinTeamResult, NearbyTeam,
    NearbyTeamsResult, RenameDeviceInput, Team, TeamDevice, TeamInvite, TeamInviteInput,
    TeamMember, TeamMembersResult, TeamPinResult,
};
use crate::domain::team::validation::{
    validate_device_name, validate_member_name, validate_team_name,
};
use crate::error::AppError;

pub fn get_team(conn: &Connection) -> Result<Option<Team>, AppError> {
    let identity = identity::require_local_identity(conn)?;
    let Some(team_id) = identity.team_id else {
        return Ok(None);
    };
    let Some((id, name, created_at, updated_at)) = repository::get_team_row(conn, &team_id)? else {
        return Ok(None);
    };
    let member_device_count = repository::count_active_devices(conn, &id)?;
    Ok(Some(Team {
        id,
        name,
        created_at,
        updated_at,
        this_device_id: identity.device_id,
        this_device_code: identity.device_code.unwrap_or_else(|| "AA".into()),
        member_device_count,
    }))
}

pub fn create_team(
    conn: &Connection,
    input: CreateTeamInput,
) -> Result<CreateTeamResult, AppError> {
    let identity = identity::require_local_identity(conn)?;
    if identity.team_id.is_some() {
        return Err(AppError::conflict("This computer is already in a team."));
    }
    let name = validate_team_name(&input.name)?;
    let member_name = validate_member_name(&input.member_name)?;
    let device_name = identity.device_name.clone();

    let pin = generate_team_pin();
    let pin_hash = hash_team_pin(&pin);
    let mut psk = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut psk);
    let psk_hex = hex::encode(psk);
    let team_id = new_entity_id();
    let now = now_utc_rfc3339()?;

    let ctx = begin_write(conn)?;
    repository::insert_team(conn, &team_id, &name, Some(&pin_hash), &now, &ctx)?;
    sync::record_upsert(
        conn,
        "teams",
        &team_id,
        team_sync_payload(&team_id, &name, &now, &now, Some(&pin_hash)),
        &ctx,
    )?;

    let ctx = begin_write(conn)?;
    repository::insert_team_device(
        conn,
        &identity.device_id,
        &team_id,
        "AA",
        &device_name,
        &now,
        &ctx,
    )?;
    sync::record_upsert(
        conn,
        "team_devices",
        &identity.device_id,
        serde_json::json!({
            "id": identity.device_id,
            "teamId": team_id,
            "deviceCode": "AA",
            "deviceName": device_name,
            "joinedAt": now,
            "removedAt": null,
            "createdAt": now,
            "updatedAt": now,
        }),
        &ctx,
    )?;

    staff::create_signed_in_member(conn, &member_name, StaffRole::Admin, Some(&team_id))?;
    repository::update_local_identity_team(
        conn,
        Some(&team_id),
        Some(&psk_hex),
        Some("AA"),
        Some(&device_name),
        Some(&pin),
    )?;

    let team = get_team(conn)?.ok_or(AppError::Internal {
        message: "team missing after create".into(),
    })?;
    let session = staff::get_current_session(conn)?.ok_or(AppError::Internal {
        message: "session missing after create".into(),
    })?;
    Ok(CreateTeamResult { team, pin, session })
}

/// Last-admin check + deactivate current staff + optional device tombstone.
/// PSK and session stay valid so gossip can still push. Returns `false` when
/// this computer is not in a team (no-op).
pub fn write_leave_tombstones(conn: &Connection) -> Result<bool, AppError> {
    let session = staff::require_session(conn)?;
    let identity = identity::require_local_identity(conn)?;
    let Some(team_id) = identity.team_id.clone() else {
        return Ok(false);
    };
    let active = repository::count_active_devices(conn, &team_id)?;
    if active > 1 && session.staff.role == StaffRole::Admin {
        let others = staff_repo::count_active_admins(conn, Some(&session.staff.id))?;
        if others == 0 {
            return Err(AppError::forbidden(
                "Leave is blocked while other computers are still in the team and you are the last administrator.",
            ));
        }
    }

    staff::deactivate_current_on_leave(conn)?;

    if active > 1 {
        let ctx = begin_write(conn)?;
        let now = now_utc_rfc3339()?;
        let device = repository::set_device_removed(conn, &identity.device_id, &now, &now, &ctx)?;
        sync::record_upsert(
            conn,
            "team_devices",
            &device.id,
            serde_json::json!({
                "id": device.id,
                "teamId": team_id,
                "deviceCode": device.device_code,
                "deviceName": device.device_name,
                "joinedAt": device.joined_at,
                "removedAt": device.removed_at,
            }),
            &ctx,
        )?;
    }

    Ok(true)
}

pub fn finish_leave(conn: &Connection) -> Result<(), AppError> {
    repository::update_local_identity_team(conn, None, None, None, None, None)?;
    staff::sign_out_staff(conn)?;
    Ok(())
}

/// Unit-test helper: tombstones then clear identity. No gossip wait.
pub fn leave_team(conn: &Connection) -> Result<(), AppError> {
    if write_leave_tombstones(conn)? {
        finish_leave(conn)?;
    }
    Ok(())
}

pub fn get_team_pin(conn: &Connection) -> Result<TeamPinResult, AppError> {
    let identity = identity::require_local_identity(conn)?;
    let Some(team_id) = identity.team_id.clone() else {
        return Err(AppError::conflict("This computer is not in a team."));
    };
    if let Some(pin) = identity
        .team_pin
        .as_deref()
        .map(str::trim)
        .filter(|pin| !pin.is_empty())
    {
        return Ok(TeamPinResult {
            pin: pin.to_string(),
        });
    }

    let pin = generate_team_pin();
    let pin_hash = hash_team_pin(&pin);
    let now = now_utc_rfc3339()?;
    let ctx = WriteContext {
        hlc: tick_hlc_value(conn)?,
        staff_id: identity.current_staff_id.clone(),
    };
    repository::set_team_pin_hash(conn, &team_id, &pin_hash, &now, &ctx)?;
    repository::update_local_identity_team(
        conn,
        Some(&team_id),
        identity.team_psk.as_deref(),
        None,
        None,
        Some(&pin),
    )?;
    if let Some((_, name, created_at, _)) = repository::get_team_row(conn, &team_id)? {
        sync::record_upsert(
            conn,
            "teams",
            &team_id,
            team_sync_payload(&team_id, &name, &created_at, &now, Some(&pin_hash)),
            &ctx,
        )?;
    }
    Ok(TeamPinResult { pin })
}

pub fn list_nearby_teams(
    conn: &Connection,
    cached: Vec<NearbyTeam>,
) -> Result<NearbyTeamsResult, AppError> {
    let identity = identity::require_local_identity(conn)?;
    if identity.team_id.is_some() {
        return Ok(NearbyTeamsResult { items: Vec::new() });
    }
    Ok(NearbyTeamsResult { items: cached })
}

pub fn list_team_members(
    conn: &Connection,
    online_device_ids: &HashSet<String>,
) -> Result<TeamMembersResult, AppError> {
    let identity = identity::require_local_identity(conn)?;
    let Some(team_id) = identity.team_id.as_deref() else {
        return Ok(TeamMembersResult { items: Vec::new() });
    };
    let this_staff_id = identity.current_staff_id.clone();
    let peer_staff = repository::presence_staff_ids_for_devices(
        conn,
        &online_device_ids.iter().cloned().collect::<Vec<_>>(),
    )?;
    let mut online_staff: HashSet<String> = peer_staff.into_iter().collect();
    if let Some(id) = this_staff_id.as_ref() {
        online_staff.insert(id.clone());
    }

    let mut items: Vec<TeamMember> = repository::list_active_staff_gigs(conn, team_id)?
        .into_iter()
        .map(|(id, name, role, gig_count)| {
            let online = online_staff.contains(&id);
            TeamMember {
                id,
                name,
                role,
                online,
                gig_count,
            }
        })
        .collect();

    items.sort_by(|a, b| {
        let a_self = this_staff_id.as_deref() == Some(a.id.as_str());
        let b_self = this_staff_id.as_deref() == Some(b.id.as_str());
        b_self
            .cmp(&a_self)
            .then(b.online.cmp(&a.online))
            .then_with(|| {
                a.name
                    .to_ascii_lowercase()
                    .cmp(&b.name.to_ascii_lowercase())
            })
            .then(a.id.cmp(&b.id))
    });
    Ok(TeamMembersResult { items })
}

pub fn create_team_invite(
    conn: &Connection,
    input: TeamInviteInput,
) -> Result<TeamInvite, AppError> {
    let session = staff::require_admin(conn)?;
    let team = get_team(conn)?.ok_or(AppError::conflict("This computer is not in a team."))?;
    let hours = input
        .expires_in_hours
        .unwrap_or(DEFAULT_INVITE_HOURS)
        .clamp(1, MAX_INVITE_HOURS);
    let expires = OffsetDateTime::now_utc()
        .saturating_add(Duration::hours(hours))
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|err| AppError::Internal {
            message: format!("timestamp format failed: {err}"),
        })?;
    let code = generate_invite_code();
    let hash = hash_invite_code(&code);
    let id = new_entity_id();
    let now = now_utc_rfc3339()?;
    let ctx = begin_write(conn)?;
    let mut invite = repository::insert_invite(
        conn,
        &id,
        &team.id,
        &hash,
        &session.staff.id,
        &expires,
        &now,
        &ctx,
    )?;
    invite.code = Some(code);
    sync::record_upsert(
        conn,
        "team_invites",
        &invite.id,
        serde_json::json!({
            "id": invite.id,
            "teamId": team.id,
            "codeHash": hash,
            "createdByStaffId": session.staff.id,
            "expiresAt": invite.expires_at,
            "revokedAt": null,
            "createdAt": invite.created_at,
        }),
        &ctx,
    )?;
    Ok(invite)
}

pub fn list_team_invites(conn: &Connection) -> Result<Vec<TeamInvite>, AppError> {
    staff::require_admin(conn)?;
    let team = get_team(conn)?.ok_or(AppError::conflict("This computer is not in a team."))?;
    repository::list_invites(conn, &team.id)
}

pub fn revoke_team_invite(conn: &Connection, id: &str) -> Result<TeamInvite, AppError> {
    staff::require_admin(conn)?;
    let id = parse_entity_id(id)?;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let invite = repository::revoke_invite(conn, &id, &now, &ctx)?;
    let team = get_team(conn)?.ok_or(AppError::NotFound)?;
    sync::record_upsert(
        conn,
        "team_invites",
        &invite.id,
        serde_json::json!({
            "id": invite.id,
            "teamId": team.id,
            "expiresAt": invite.expires_at,
            "revokedAt": invite.revoked_at,
            "createdAt": invite.created_at,
        }),
        &ctx,
    )?;
    Ok(invite)
}

pub fn list_team_devices(conn: &Connection) -> Result<Vec<TeamDevice>, AppError> {
    let identity = identity::require_local_identity(conn)?;
    let Some(team_id) = identity.team_id else {
        return Ok(Vec::new());
    };
    repository::list_team_devices(conn, &team_id, &identity.device_id)
}

pub fn remove_team_device(conn: &Connection, device_id: &str) -> Result<TeamDevice, AppError> {
    staff::require_admin(conn)?;
    let device_id = parse_entity_id(device_id)?;
    let identity = identity::require_local_identity(conn)?;
    if device_id == identity.device_id {
        return Err(AppError::Validation {
            field: Some("deviceId".into()),
            message: "Use leave team to remove this computer.".into(),
        });
    }
    let team_id = identity
        .team_id
        .ok_or_else(|| AppError::conflict("This computer is not in a team."))?;
    let active = repository::count_active_devices(conn, &team_id)?;
    if active <= 1 {
        return Err(AppError::forbidden(
            "Cannot remove the last remaining device.",
        ));
    }
    let existing = repository::get_team_device(conn, &device_id, &identity.device_id)?
        .ok_or(AppError::NotFound)?;
    if existing.removed_at.is_some() {
        return Ok(existing);
    }
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let device = repository::set_device_removed(conn, &device_id, &now, &now, &ctx)?;
    sync::record_upsert(
        conn,
        "team_devices",
        &device.id,
        serde_json::json!({
            "id": device.id,
            "teamId": team_id,
            "deviceCode": device.device_code,
            "deviceName": device.device_name,
            "joinedAt": device.joined_at,
            "removedAt": device.removed_at,
        }),
        &ctx,
    )?;
    Ok(device)
}

pub fn rename_this_device(
    conn: &Connection,
    input: RenameDeviceInput,
) -> Result<TeamDevice, AppError> {
    identity::require_write_access(conn)?;
    let name = validate_device_name(&input.device_name)?;
    let identity = identity::require_local_identity(conn)?;
    repository::update_local_identity_team(
        conn,
        identity.team_id.as_deref(),
        identity.team_psk.as_deref(),
        None,
        Some(&name),
        identity.team_pin.as_deref(),
    )?;
    if identity.team_id.is_none() {
        return Ok(TeamDevice {
            id: identity.device_id,
            device_code: identity.device_code.unwrap_or_else(|| "AA".into()),
            device_name: name,
            joined_at: String::new(),
            removed_at: None,
            is_this_device: true,
        });
    }
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let device = repository::rename_device(conn, &identity.device_id, &name, &now, &ctx)?;
    sync::record_upsert(
        conn,
        "team_devices",
        &device.id,
        serde_json::json!({
            "id": device.id,
            "teamId": identity.team_id,
            "deviceCode": device.device_code,
            "deviceName": device.device_name,
            "joinedAt": device.joined_at,
            "removedAt": device.removed_at,
        }),
        &ctx,
    )?;
    Ok(device)
}

/// Member-side check used when a joiner announces an invite hash.
pub fn validate_invite_for_grant(conn: &Connection, invite_code: &str) -> Result<String, AppError> {
    let canonical = normalize_invite_code(invite_code)?;
    let hash = hash_invite_code(&canonical);
    let Some((_id, expires_at, revoked_at)) = repository::find_invite_by_hash(conn, &hash)? else {
        return Err(AppError::conflict("Invite code is not valid."));
    };
    if revoked_at.is_some() {
        return Err(AppError::conflict("This invite has been revoked."));
    }
    if invite_is_expired(&expires_at) {
        return Err(AppError::conflict("This invite has expired."));
    }
    Ok(canonical)
}

pub fn next_device_code(conn: &Connection, team_id: &str) -> Result<String, AppError> {
    let used = repository::used_device_codes(conn, team_id)?;
    for a in b'A'..=b'Z' {
        for b in b'A'..=b'Z' {
            let code = format!("{}{}", a as char, b as char);
            if !used.iter().any(|c| c == &code) {
                return Ok(code);
            }
        }
    }
    Err(AppError::conflict(
        "The team has no free device codes left.",
    ))
}

/// Validate local preconditions for joining. Returns the canonical PIN.
pub fn prepare_join(conn: &Connection, input: &JoinTeamInput) -> Result<String, AppError> {
    let identity = identity::require_local_identity(conn)?;
    if identity.team_id.is_some() {
        return Err(AppError::conflict("This computer is already in a team."));
    }
    let _team_id = parse_entity_id_field(&input.team_id, "teamId")?;
    let pin = validate_team_pin(&input.pin)?;
    validate_member_name(&input.member_name)?;
    Ok(pin)
}

/// Join requires an online peer (LAN). Validates local preconditions, then
/// returns a sync error when no runtime grant is supplied (unit tests).
pub fn join_team(conn: &Connection, input: JoinTeamInput) -> Result<JoinTeamResult, AppError> {
    let _ = prepare_join(conn, &input)?;
    Err(AppError::sync_err(
        "No team member answered on the network. Make sure another computer is online.",
    ))
}

#[derive(Debug, Clone)]
pub struct JoinGrant {
    pub team_id: String,
    pub team_name: String,
    pub team_psk_hex: String,
    pub device_code: String,
    pub snapshot_applied: bool,
}

/// Member-side: validate PIN hash and reserve the next device code for the joiner.
pub fn issue_join_grant(
    conn: &Connection,
    pin: &str,
    joiner_device_id: &str,
    joiner_name: &str,
) -> Result<JoinGrant, AppError> {
    let pin = validate_team_pin(pin)?;
    let identity = identity::require_local_identity(conn)?;
    let team_id = identity
        .team_id
        .ok_or_else(|| AppError::conflict("This computer is not in a team."))?;
    let stored_hash = repository::get_team_pin_hash(conn, &team_id)?
        .ok_or_else(|| AppError::conflict("Team PIN is not set."))?;
    if stored_hash != hash_team_pin(&pin) {
        return Err(AppError::conflict("Team PIN is not valid."));
    }
    let psk = identity
        .team_psk
        .ok_or_else(|| AppError::sync_err("Team key is missing."))?;
    let team = get_team(conn)?.ok_or_else(|| AppError::conflict("Team was not found."))?;
    if let Some(existing) =
        repository::get_team_device(conn, joiner_device_id, &identity.device_id)?
    {
        if existing.removed_at.is_none() {
            return Ok(JoinGrant {
                team_id,
                team_name: team.name,
                team_psk_hex: psk,
                device_code: existing.device_code,
                snapshot_applied: false,
            });
        }
    }
    let device_code = next_device_code(conn, &team_id)?;
    let name = validate_device_name(joiner_name).unwrap_or_else(|_| "PC".into());
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    repository::insert_team_device(
        conn,
        joiner_device_id,
        &team_id,
        &device_code,
        &name,
        &now,
        &ctx,
    )?;
    sync::record_upsert(
        conn,
        "team_devices",
        joiner_device_id,
        serde_json::json!({
            "id": joiner_device_id,
            "teamId": team_id,
            "deviceCode": device_code,
            "deviceName": name,
            "joinedAt": now,
            "removedAt": null,
        }),
        &ctx,
    )?;
    Ok(JoinGrant {
        team_id,
        team_name: team.name,
        team_psk_hex: psk,
        device_code,
        snapshot_applied: false,
    })
}

/// Joiner-side: persist grant onto local_identity, create staff, and sign in.
pub fn apply_join_grant(
    conn: &Connection,
    grant: &JoinGrant,
    member_name: &str,
    pin: &str,
) -> Result<JoinTeamResult, AppError> {
    let identity = identity::require_local_identity(conn)?;
    if identity.team_id.is_some() {
        return Err(AppError::conflict("This computer is already in a team."));
    }
    let member_name = validate_member_name(member_name)?;
    let pin = validate_team_pin(pin)?;
    let pin_hash = hash_team_pin(&pin);
    let name = identity.device_name.clone();
    let now = now_utc_rfc3339()?;
    if repository::get_team_row(conn, &grant.team_id)?.is_none() {
        let ctx = begin_write(conn)?;
        repository::insert_team(
            conn,
            &grant.team_id,
            &grant.team_name,
            Some(&pin_hash),
            &now,
            &ctx,
        )?;
    }
    if repository::get_team_device(conn, &identity.device_id, &identity.device_id)?.is_none() {
        let ctx = begin_write(conn)?;
        repository::insert_team_device(
            conn,
            &identity.device_id,
            &grant.team_id,
            &grant.device_code,
            &name,
            &now,
            &ctx,
        )?;
    }
    staff::create_signed_in_member(conn, &member_name, StaffRole::Staff, Some(&grant.team_id))?;
    repository::update_local_identity_team(
        conn,
        Some(&grant.team_id),
        Some(&grant.team_psk_hex),
        Some(&grant.device_code),
        Some(&name),
        Some(&pin),
    )?;
    let team = get_team(conn)?.ok_or(AppError::Internal {
        message: "team missing after join".into(),
    })?;
    let session = staff::get_current_session(conn)?.ok_or(AppError::Internal {
        message: "session missing after join".into(),
    })?;
    Ok(JoinTeamResult {
        device_code: grant.device_code.clone(),
        snapshot_applied: grant.snapshot_applied,
        team,
        session,
    })
}

pub fn invite_is_expired(expires_at: &str) -> bool {
    match OffsetDateTime::parse(expires_at, &time::format_description::well_known::Rfc3339) {
        Ok(expires) => OffsetDateTime::now_utc() > expires,
        Err(_) => true,
    }
}

fn team_sync_payload(
    id: &str,
    name: &str,
    created_at: &str,
    updated_at: &str,
    pin_hash: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": name,
        "createdAt": created_at,
        "updatedAt": updated_at,
        "pinHash": pin_hash,
    })
}
