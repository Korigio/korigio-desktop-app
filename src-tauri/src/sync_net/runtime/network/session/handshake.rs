use super::*;

pub(in crate::sync_net::runtime::network) async fn drive_session(
    runtime: SyncRuntime,
    db: Arc<Mutex<crate::db::Db>>,
    mut stream: TcpStream,
    expected_peer: Option<String>,
    join_pin: Option<String>,
) -> Result<(), AppError> {
    let me = snapshot_identity(&db).ok_or_else(|| AppError::sync_err("Identity is missing."))?;
    let pending_join = runtime
        .inner
        .lock()
        .ok()
        .and_then(|inner| inner.join_pin.clone());
    let is_join = use_join_key(
        join_pin.as_deref(),
        pending_join.as_deref(),
        me.team_id.as_deref(),
    );
    let (key, psk_bytes) = session_key(&me, join_pin.as_deref(), pending_join.as_deref(), is_join)?;

    let mut my_nonce = [0u8; 32];
    use rand::RngCore;
    rand::thread_rng().fill_bytes(&mut my_nonce);
    write_msg(
        &mut stream,
        &key,
        &TcpMessage::Hello {
            proto: PROTO.into(),
            device_id: me.device_id.clone(),
            nonce: hex::encode(my_nonce),
        },
    )
    .await?;
    let peer_id = authenticate_peer(
        &db,
        &mut stream,
        &key,
        &psk_bytes,
        &me.device_id,
        &my_nonce,
        expected_peer.as_deref(),
    )
    .await?;

    runtime.mark_online(&peer_id);
    if let Ok(mut inner) = runtime.inner.lock() {
        inner.connected.insert(peer_id.clone());
        inner.error_message = None;
    }

    let result = if is_join {
        drive_join(
            &runtime,
            &db,
            &mut stream,
            &key,
            &me,
            &peer_id,
            join_pin.or(pending_join),
        )
        .await
    } else {
        gossip_loop(&runtime, &db, stream, &key, &me, &peer_id).await
    };

    if let Ok(mut inner) = runtime.inner.lock() {
        inner.connected.remove(&peer_id);
        inner.snapshot_rows.remove(&peer_id);
    }
    result
}

fn session_key(
    me: &NetIdentity,
    join_pin: Option<&str>,
    pending_join: Option<&str>,
    is_join: bool,
) -> Result<([u8; 32], Vec<u8>), AppError> {
    if is_join {
        let pin = join_pin
            .or(pending_join)
            .ok_or_else(|| AppError::sync_err("Join PIN is missing."))?;
        return Ok((join_key_from_pin(pin)?, pin.as_bytes().to_vec()));
    }
    let psk_hex = me
        .team_psk
        .as_deref()
        .ok_or_else(|| AppError::sync_err("Team key is missing."))?;
    let raw = hex::decode(psk_hex).map_err(|_| AppError::sync_err("Team key is invalid."))?;
    Ok((team_key_from_psk_hex(psk_hex)?, raw))
}

async fn authenticate_peer(
    db: &Arc<Mutex<crate::db::Db>>,
    stream: &mut TcpStream,
    key: &[u8; 32],
    psk_bytes: &[u8],
    local_device_id: &str,
    local_nonce: &[u8; 32],
    expected_peer: Option<&str>,
) -> Result<String, AppError> {
    let hello = read_msg(stream, key).await?;
    let (peer_id, peer_nonce_hex) = match hello {
        TcpMessage::Hello {
            proto,
            device_id,
            nonce,
        } if proto == PROTO => (device_id, nonce),
        _ => return Err(AppError::sync_err("Peer handshake was rejected.")),
    };
    if expected_peer.is_some_and(|expected| expected != peer_id) {
        return Err(AppError::sync_err("Peer identity did not match."));
    }
    if device_is_removed(db, &peer_id) {
        return Err(AppError::sync_err("This device was removed from the team."));
    }
    let peer_nonce = hex::decode(peer_nonce_hex)
        .map_err(|_| AppError::sync_err("Peer handshake was rejected."))?;
    write_msg(
        stream,
        key,
        &TcpMessage::Auth {
            mac: auth_mac(psk_bytes, &peer_nonce, local_device_id),
        },
    )
    .await?;
    let TcpMessage::Auth { mac } = read_msg(stream, key).await? else {
        return Err(AppError::sync_err("Peer handshake was rejected."));
    };
    if mac != auth_mac(psk_bytes, local_nonce, &peer_id) {
        return Err(AppError::sync_err("Peer handshake was rejected."));
    }
    Ok(peer_id)
}

async fn drive_join(
    runtime: &SyncRuntime,
    db: &Arc<Mutex<crate::db::Db>>,
    stream: &mut TcpStream,
    key: &[u8; 32],
    me: &NetIdentity,
    peer_id: &str,
    join_pin: Option<String>,
) -> Result<(), AppError> {
    if me.team_id.is_some() {
        if let Some(pin) = join_pin {
            let grant = {
                let guard = db.lock().map_err(|_| AppError::Internal {
                    message: "database lock poisoned".into(),
                })?;
                crate::domain::team::issue_join_grant(guard.conn(), &pin, peer_id, "PC")?
            };
            write_msg(
                stream,
                key,
                &TcpMessage::Grant {
                    team_id: grant.team_id,
                    team_psk_hex: grant.team_psk_hex,
                    device_code: grant.device_code,
                },
            )
            .await?;
            send_join_snapshot(stream, key, db).await?;
            wait_for_join_blob_want(stream, key, db).await?;
        }
    } else {
        receive_join_snapshot(runtime, db, stream, key, peer_id).await?;
    }
    Ok(())
}

/// Use the join/PIN key only for a real join session — never leftover PIN after this PC is in a team.
pub(in crate::sync_net::runtime::network) fn use_join_key(
    join_pin: Option<&str>,
    pending_join: Option<&str>,
    team_id: Option<&str>,
) -> bool {
    join_pin.is_some() || pending_join.is_some() && team_id.is_none()
}
