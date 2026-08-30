//! Thin team / PIN / device IPC adapters.

use tauri::State;

use crate::db::DbState;
use crate::domain::team::{
    self, CreateTeamInput, CreateTeamResult, JoinTeamInput, JoinTeamResult, NearbyTeamsResult,
    RenameDeviceInput, Team, TeamDevice, TeamInvite, TeamInviteInput, TeamMembersResult,
    TeamPinResult,
};
use crate::error::{AppError, CommandError};
use crate::sync_net::SyncRuntime;

fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, crate::db::Db>, AppError> {
    state.0.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })
}

#[tauri::command]
pub fn get_team(state: State<'_, DbState>) -> Result<Option<Team>, CommandError> {
    let db = lock_db(&state)?;
    Ok(team::get_team(db.conn())?)
}

#[tauri::command]
pub fn create_team(
    state: State<'_, DbState>,
    input: CreateTeamInput,
) -> Result<CreateTeamResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(team::create_team(db.conn(), input)?)
}

#[tauri::command]
pub fn leave_team(state: State<'_, DbState>) -> Result<(), CommandError> {
    let db = lock_db(&state)?;
    Ok(team::leave_team(db.conn())?)
}

#[tauri::command]
pub fn get_team_pin(state: State<'_, DbState>) -> Result<TeamPinResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(team::get_team_pin(db.conn())?)
}

#[tauri::command]
pub fn list_nearby_teams(
    state: State<'_, DbState>,
    runtime: State<'_, SyncRuntime>,
) -> Result<NearbyTeamsResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(team::list_nearby_teams(db.conn(), runtime.nearby_teams())?)
}

#[tauri::command]
pub fn list_team_members(
    state: State<'_, DbState>,
    runtime: State<'_, SyncRuntime>,
) -> Result<TeamMembersResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(team::list_team_members(
        db.conn(),
        &runtime.online_device_ids(),
    )?)
}

#[tauri::command]
pub fn create_team_invite(
    state: State<'_, DbState>,
    runtime: State<'_, SyncRuntime>,
    input: TeamInviteInput,
) -> Result<TeamInvite, CommandError> {
    let db = lock_db(&state)?;
    let invite = team::create_team_invite(db.conn(), input)?;
    if let Some(code) = invite.code.as_deref() {
        runtime.remember_invite(code);
    }
    Ok(invite)
}

#[tauri::command]
pub fn list_team_invites(state: State<'_, DbState>) -> Result<Vec<TeamInvite>, CommandError> {
    let db = lock_db(&state)?;
    Ok(team::list_team_invites(db.conn())?)
}

#[tauri::command]
pub fn revoke_team_invite(
    state: State<'_, DbState>,
    id: String,
) -> Result<TeamInvite, CommandError> {
    let db = lock_db(&state)?;
    Ok(team::revoke_team_invite(db.conn(), &id)?)
}

#[tauri::command]
pub fn join_team(
    state: State<'_, DbState>,
    runtime: State<'_, SyncRuntime>,
    input: JoinTeamInput,
) -> Result<JoinTeamResult, CommandError> {
    {
        let db = lock_db(&state)?;
        team::prepare_join(db.conn(), &input)?;
    }
    runtime.begin_join(
        input.team_id.clone(),
        input.pin.clone(),
        input.member_name.clone(),
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while std::time::Instant::now() < deadline {
        if let Some(grant) = runtime.take_join_grant() {
            let result = (|| {
                let db = lock_db(&state)?;
                team::apply_join_grant(db.conn(), &grant, &input.member_name, &input.pin)
            })();
            runtime.clear_join();
            runtime.nudge();
            return Ok(result?);
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    runtime.clear_join();
    Err(AppError::sync_err(
        "No team member answered on the network. Make sure another computer is online.",
    )
    .into())
}

#[tauri::command]
pub fn list_team_devices(state: State<'_, DbState>) -> Result<Vec<TeamDevice>, CommandError> {
    let db = lock_db(&state)?;
    Ok(team::list_team_devices(db.conn())?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn remove_team_device(
    state: State<'_, DbState>,
    device_id: String,
) -> Result<TeamDevice, CommandError> {
    let db = lock_db(&state)?;
    Ok(team::remove_team_device(db.conn(), &device_id)?)
}

#[tauri::command]
pub fn rename_this_device(
    state: State<'_, DbState>,
    input: RenameDeviceInput,
) -> Result<TeamDevice, CommandError> {
    let db = lock_db(&state)?;
    Ok(team::rename_this_device(db.conn(), input)?)
}
