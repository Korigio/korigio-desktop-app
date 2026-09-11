use super::*;

mod discovery;
mod session;
mod transfer;
mod wire;

pub(super) use discovery::lan_loop;

#[derive(Clone)]
struct NetIdentity {
    device_id: String,
    device_name: String,
    team_id: Option<String>,
    team_name: Option<String>,
    team_psk: Option<String>,
    team_pin: Option<String>,
    staff_id: Option<String>,
}

fn snapshot_identity(db: &Arc<Mutex<crate::db::Db>>) -> Option<NetIdentity> {
    let guard = db.lock().ok()?;
    let identity = identity::get_local_identity(guard.conn()).ok().flatten()?;
    let team_name = identity.team_id.as_ref().and_then(|id| {
        crate::domain::team::repository::get_team_row(guard.conn(), id)
            .ok()
            .flatten()
            .map(|(_, name, _, _)| name)
    });
    Some(NetIdentity {
        device_id: identity.device_id,
        device_name: identity.device_name,
        team_id: identity.team_id,
        team_name,
        team_psk: identity.team_psk,
        team_pin: identity.team_pin,
        staff_id: identity.current_staff_id,
    })
}

#[cfg(test)]
mod tests {
    use super::session::{is_default_empty_cursor, should_dial_on_hello, use_join_key};
    use crate::domain::sync::hlc::Hlc;

    #[test]
    fn hello_always_dials_regardless_of_device_id_order() {
        assert!(should_dial_on_hello("aaa", "zzz"));
        assert!(should_dial_on_hello("zzz", "aaa"));
        assert!(should_dial_on_hello("same", "same"));
    }

    #[test]
    fn default_peer_cursor_is_empty_wall_and_counter() {
        assert!(is_default_empty_cursor(&Hlc::new(0, 0, String::new())));
        assert!(is_default_empty_cursor(&Hlc::new(0, 0, "device-1")));
        assert!(!is_default_empty_cursor(&Hlc::new(1, 0, String::new())));
        assert!(!is_default_empty_cursor(&Hlc::new(0, 1, String::new())));
    }

    #[test]
    fn joiner_not_yet_in_team_uses_pending_pin() {
        assert!(use_join_key(None, Some("123456"), None));
    }

    #[test]
    fn leftover_pending_after_join_uses_team_psk() {
        assert!(!use_join_key(None, Some("123456"), Some("team-1")));
    }

    #[test]
    fn host_session_join_pin_uses_join_key() {
        assert!(use_join_key(Some("123456"), None, Some("team-1")));
    }

    #[test]
    fn neither_pin_uses_team_psk() {
        assert!(!use_join_key(None, None, Some("team-1")));
        assert!(!use_join_key(None, None, None));
    }
}
