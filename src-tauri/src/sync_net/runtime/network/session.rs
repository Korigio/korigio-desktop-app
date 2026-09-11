use super::discovery::device_is_removed;
use super::transfer::{
    missing_blobs, receive_join_snapshot, send_blobs, send_join_snapshot, store_blob_chunk,
    wait_for_join_blob_want,
};
use super::wire::{read_msg, to_wire, write_msg};
use super::*;

mod handshake;

pub(super) use handshake::{drive_session, use_join_key};

/// Both sides always dial on team hello so a one-way firewall cannot stall gossip.
pub(super) fn should_dial_on_hello(_local_device_id: &str, _peer_device_id: &str) -> bool {
    true
}

pub(super) fn is_default_empty_cursor(hlc: &Hlc) -> bool {
    hlc.wall == 0 && hlc.counter == 0
}

pub(super) fn remember_peer_addr(runtime: &SyncRuntime, device_id: &str, src: SocketAddr) {
    if let Ok(mut inner) = runtime.inner.lock() {
        inner.peer_addrs.insert(device_id.to_string(), src);
    }
}

async fn maybe_send_catchup_snapshot(
    runtime: &SyncRuntime,
    db: &Arc<Mutex<crate::db::Db>>,
    writer: &TcpWriter,
    key: &[u8; 32],
    peer_id: &str,
    rx: &mut tokio::sync::mpsc::Receiver<Result<TcpMessage, AppError>>,
) -> Result<(), AppError> {
    let empty = {
        let guard = db.lock().map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
        is_default_empty_cursor(&get_peer_cursor(guard.conn(), peer_id)?)
    };
    if !empty {
        return Ok(());
    }
    if let Ok(mut inner) = runtime.inner.lock() {
        inner.catching_up = true;
    }
    let (rows, hashes) = {
        let guard = db.lock().map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
        let (_schema_version, rows) = dump_snapshot(guard.conn())?;
        let hashes = list_blob_hashes(guard.conn())?;
        (rows, hashes)
    };
    drain_gossip_inbox(runtime, db, writer, key, peer_id, rx).await?;
    write_locked(
        writer,
        key,
        &TcpMessage::SnapshotBegin {
            schema_version: SCHEMA_VERSION,
            counts: rows.len() as i64,
        },
    )
    .await?;
    for row in rows {
        drain_gossip_inbox(runtime, db, writer, key, peer_id, rx).await?;
        write_locked(
            writer,
            key,
            &TcpMessage::SnapshotRow {
                table: row.table,
                payload: row.payload,
            },
        )
        .await?;
    }
    if !hashes.is_empty() {
        drain_gossip_inbox(runtime, db, writer, key, peer_id, rx).await?;
        write_locked(writer, key, &TcpMessage::SnapshotBlobList { hashes }).await?;
    }
    drain_gossip_inbox(runtime, db, writer, key, peer_id, rx).await?;
    write_locked(writer, key, &TcpMessage::SnapshotEnd).await
}

async fn drain_gossip_inbox(
    runtime: &SyncRuntime,
    db: &Arc<Mutex<crate::db::Db>>,
    writer: &TcpWriter,
    key: &[u8; 32],
    peer_id: &str,
    rx: &mut tokio::sync::mpsc::Receiver<Result<TcpMessage, AppError>>,
) -> Result<(), AppError> {
    loop {
        match rx.try_recv() {
            Ok(msg) => {
                handle_tcp_message(runtime, db, writer, key, peer_id, msg?).await?;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => return Ok(()),
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                return Err(AppError::sync_err("Peer connection closed."));
            }
        }
    }
}

type TcpWriter = Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>;

async fn gossip_loop(
    runtime: &SyncRuntime,
    db: &Arc<Mutex<crate::db::Db>>,
    stream: TcpStream,
    key: &[u8; 32],
    me: &NetIdentity,
    peer_id: &str,
) -> Result<(), AppError> {
    let (read_half, write_half) = stream.into_split();
    let writer: TcpWriter = Arc::new(tokio::sync::Mutex::new(write_half));
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<TcpMessage, AppError>>(16);
    let reader_key = *key;
    let reader = tokio::spawn(async move {
        let mut read_half = read_half;
        loop {
            match read_msg(&mut read_half, &reader_key).await {
                Ok(msg) => {
                    if tx.send(Ok(msg)).await.is_err() {
                        break;
                    }
                }
                Err(err) => {
                    let _ = tx.send(Err(err)).await;
                    break;
                }
            }
        }
    });

    let result = async {
        send_heartbeat_and_pull(db, &writer, key, me, peer_id).await?;
        maybe_send_catchup_snapshot(runtime, db, &writer, key, peer_id, &mut rx).await?;
        let sleep = tokio::time::sleep(runtime.interval());
        tokio::pin!(sleep);
        loop {
            tokio::select! {
                _ = &mut sleep => {
                    send_heartbeat_and_pull(db, &writer, key, me, peer_id).await?;
                    sleep.as_mut().reset(tokio::time::Instant::now() + runtime.interval());
                }
                _ = runtime.notify.notified() => {
                    send_heartbeat_and_pull(db, &writer, key, me, peer_id).await?;
                    sleep.as_mut().reset(tokio::time::Instant::now() + runtime.interval());
                }
                _ = crate::domain::sync::local_change_notified() => {
                    send_heartbeat_and_pull(db, &writer, key, me, peer_id).await?;
                    sleep.as_mut().reset(tokio::time::Instant::now() + runtime.interval());
                }
                msg = rx.recv() => {
                    let Some(msg) = msg else {
                        return Err(AppError::sync_err("Peer connection closed."));
                    };
                    handle_tcp_message(runtime, db, &writer, key, peer_id, msg?).await?;
                }
            }
        }
    }
    .await;
    reader.abort();
    result
}

async fn send_heartbeat_and_pull(
    db: &Arc<Mutex<crate::db::Db>>,
    writer: &TcpWriter,
    key: &[u8; 32],
    me: &NetIdentity,
    peer_id: &str,
) -> Result<(), AppError> {
    let hlc_max = {
        let guard = db.lock().map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
        identity::require_local_identity(guard.conn())
            .map(|id| Hlc::new(id.hlc_wall_ms, id.hlc_counter, id.device_id))?
    };
    write_locked(
        writer,
        key,
        &TcpMessage::Heartbeat {
            staff_id: me.staff_id.clone(),
            device_name: me.device_name.clone(),
            hlc_max,
        },
    )
    .await?;
    let after = {
        let guard = db.lock().map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
        get_peer_cursor(guard.conn(), peer_id)?
    };
    write_locked(writer, key, &TcpMessage::ChangePull { after }).await?;
    Ok(())
}

async fn write_locked(
    writer: &TcpWriter,
    key: &[u8; 32],
    msg: &TcpMessage,
) -> Result<(), AppError> {
    let mut write_half = writer.lock().await;
    write_msg(&mut *write_half, key, msg).await
}

async fn handle_tcp_message(
    runtime: &SyncRuntime,
    db: &Arc<Mutex<crate::db::Db>>,
    writer: &TcpWriter,
    key: &[u8; 32],
    peer_id: &str,
    msg: TcpMessage,
) -> Result<(), AppError> {
    match msg {
        TcpMessage::Heartbeat {
            staff_id,
            device_name: _,
            hlc_max: _,
        } => {
            runtime.mark_online(peer_id);
            let now = now_utc_rfc3339()?;
            let guard = db.lock().map_err(|_| AppError::Internal {
                message: "database lock poisoned".into(),
            })?;
            guard.conn().execute(
                "INSERT INTO presence (id, device_id, staff_id, last_seen_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(device_id) DO UPDATE SET
                    staff_id = excluded.staff_id,
                    last_seen_at = excluded.last_seen_at",
                rusqlite::params![crate::domain::ids::new_entity_id(), peer_id, staff_id, now],
            )?;
        }
        TcpMessage::ChangePull { after } => {
            let changes = {
                let guard = db.lock().map_err(|_| AppError::Internal {
                    message: "database lock poisoned".into(),
                })?;
                changes_after(guard.conn(), &after, CHANGE_PUSH_MAX as i64)?
            };
            if !changes.is_empty() {
                write_locked(
                    writer,
                    key,
                    &TcpMessage::ChangePush {
                        changes: changes.into_iter().map(to_wire).collect(),
                    },
                )
                .await?;
            }
        }
        TcpMessage::ChangePush { changes } => {
            let (applied_ids, mutated) = apply_pushed(db, peer_id, changes)?;
            runtime.emit_sync_applied(mutated);
            write_locked(writer, key, &TcpMessage::ChangeAck { applied_ids }).await?;
            if let Ok(mut inner) = runtime.inner.lock() {
                inner.last_synced_at = now_utc_rfc3339().ok();
                inner.catching_up = false;
                inner.state = "synced".into();
            }
        }
        TcpMessage::SnapshotRow { table, payload } => {
            {
                let guard = db.lock().map_err(|_| AppError::Internal {
                    message: "database lock poisoned".into(),
                })?;
                apply_snapshot_row(guard.conn(), &table, payload)?;
            }
            if let Ok(mut inner) = runtime.inner.lock() {
                *inner.snapshot_rows.entry(peer_id.to_string()).or_insert(0) += 1;
            }
        }
        TcpMessage::SnapshotEnd => {
            let applied = if let Ok(mut inner) = runtime.inner.lock() {
                inner.last_synced_at = now_utc_rfc3339().ok();
                inner.catching_up = false;
                inner.state = "synced".into();
                inner.snapshot_rows.remove(peer_id).unwrap_or(1).max(1)
            } else {
                1
            };
            runtime.emit_sync_applied(applied);
        }
        TcpMessage::SnapshotBegin { .. } | TcpMessage::Grant { .. } => {}
        TcpMessage::BlobHave { hashes } | TcpMessage::SnapshotBlobList { hashes } => {
            let missing = missing_blobs(db, hashes)?;
            if !missing.is_empty() {
                write_locked(
                    writer,
                    key,
                    &TcpMessage::BlobWant {
                        hashes: missing.into_iter().take(BLOB_WANT_MAX).collect(),
                    },
                )
                .await?;
            }
        }
        TcpMessage::BlobWant { hashes } => {
            let mut write_half = writer.lock().await;
            send_blobs(db, &mut *write_half, key, hashes).await?;
        }
        TcpMessage::BlobChunk {
            hash,
            offset,
            data,
            eof,
        } => {
            if store_blob_chunk(db, &hash, offset, &data, eof)? {
                runtime.emit_sync_applied(1);
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn apply_pushed(
    db: &Arc<Mutex<crate::db::Db>>,
    peer_id: &str,
    changes: Vec<ChangeWire>,
) -> Result<(Vec<String>, u64), AppError> {
    let guard = db.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })?;
    let mut applied = Vec::new();
    let mut mutated = 0u64;
    let mut max_hlc: Option<Hlc> = None;
    for wire in changes {
        let change = SyncChange {
            id: wire.id.clone(),
            entity_table: wire.entity_table,
            entity_id: wire.entity_id,
            op: wire.op,
            payload_json: wire.payload_json,
            hlc: wire.hlc.clone(),
            created_at: wire.created_at,
        };
        let outcome = crate::domain::sync::apply_remote_change(guard.conn(), &change)?;
        applied.push(wire.id);
        if outcome == ApplyOutcome::Applied {
            mutated += 1;
        }
        if max_hlc
            .as_ref()
            .map(|h| crate::domain::sync::hlc_greater(&wire.hlc, h))
            .unwrap_or(true)
        {
            max_hlc = Some(wire.hlc);
        }
    }
    if let Some(hlc) = max_hlc {
        set_peer_cursor(guard.conn(), peer_id, &hlc)?;
    }
    Ok((applied, mutated))
}
