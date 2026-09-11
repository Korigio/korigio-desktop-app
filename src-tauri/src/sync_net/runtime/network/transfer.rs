use super::session::apply_pushed;
use super::wire::{read_msg, write_msg};
use super::*;

pub(super) async fn send_join_snapshot<W>(
    stream: &mut W,
    key: &[u8; 32],
    db: &Arc<Mutex<crate::db::Db>>,
) -> Result<(), AppError>
where
    W: AsyncWriteExt + Unpin,
{
    let (rows, hashes) = {
        let guard = db.lock().map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
        let (_schema_version, rows) = dump_snapshot(guard.conn())?;
        let hashes = list_blob_hashes(guard.conn())?;
        (rows, hashes)
    };
    write_msg(
        stream,
        key,
        &TcpMessage::SnapshotBegin {
            schema_version: SCHEMA_VERSION,
            counts: rows.len() as i64,
        },
    )
    .await?;
    for row in rows {
        write_msg(
            stream,
            key,
            &TcpMessage::SnapshotRow {
                table: row.table,
                payload: row.payload,
            },
        )
        .await?;
    }
    if !hashes.is_empty() {
        write_msg(stream, key, &TcpMessage::SnapshotBlobList { hashes }).await?;
    }
    write_msg(stream, key, &TcpMessage::SnapshotEnd).await?;
    Ok(())
}

pub(super) async fn wait_for_join_blob_want(
    stream: &mut TcpStream,
    key: &[u8; 32],
    db: &Arc<Mutex<crate::db::Db>>,
) -> Result<(), AppError> {
    tokio::select! {
        msg = read_msg(stream, key) => {
            match msg? {
                TcpMessage::BlobWant { hashes } => {
                    send_blobs(db, stream, key, hashes).await?;
                }
                _ => {}
            }
        }
        _ = tokio::time::sleep(Duration::from_secs(10)) => {}
    }
    Ok(())
}

pub(super) async fn receive_join_snapshot(
    runtime: &SyncRuntime,
    db: &Arc<Mutex<crate::db::Db>>,
    stream: &mut TcpStream,
    key: &[u8; 32],
    peer_id: &str,
) -> Result<(), AppError> {
    let mut grant_fields: Option<(String, String, String)> = None;
    let mut team_name: Option<String> = None;
    let mut snapshot_applied = false;
    let mut snapshot_rows: u64 = 0;
    let mut listed_hashes: Vec<String> = Vec::new();
    let mut wanted: HashSet<String> = HashSet::new();
    let mut snapshot_done = false;

    loop {
        let msg = if snapshot_done {
            tokio::select! {
                msg = read_msg(stream, key) => msg?,
                _ = tokio::time::sleep(Duration::from_secs(15)) => break,
            }
        } else {
            read_msg(stream, key).await?
        };
        match msg {
            TcpMessage::Grant {
                team_id,
                team_psk_hex,
                device_code,
            } => {
                grant_fields = Some((team_id, team_psk_hex, device_code));
            }
            TcpMessage::SnapshotRow { table, payload } => {
                if table == "teams" {
                    if let Some(name) = payload.get("name").and_then(|v| v.as_str()) {
                        if !name.is_empty() {
                            team_name = Some(name.to_string());
                        }
                    }
                }
                {
                    let guard = db.lock().map_err(|_| AppError::Internal {
                        message: "database lock poisoned".into(),
                    })?;
                    apply_snapshot_row(guard.conn(), &table, payload)?;
                }
                snapshot_rows += 1;
                snapshot_applied = true;
            }
            TcpMessage::ChangePush { changes } => {
                for wire in &changes {
                    if wire.entity_table == "teams" {
                        if let Ok(value) =
                            serde_json::from_str::<serde_json::Value>(&wire.payload_json)
                        {
                            if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
                                if !name.is_empty() {
                                    team_name = Some(name.to_string());
                                }
                            }
                        }
                    }
                }
                let (_applied_ids, mutated) = apply_pushed(db, peer_id, changes)?;
                runtime.emit_sync_applied(mutated);
                snapshot_applied = true;
            }
            TcpMessage::SnapshotBlobList { hashes } | TcpMessage::BlobHave { hashes } => {
                listed_hashes = hashes;
            }
            TcpMessage::SnapshotEnd => {
                snapshot_done = true;
                let mut candidates = listed_hashes.clone();
                if candidates.is_empty() {
                    let guard = db.lock().map_err(|_| AppError::Internal {
                        message: "database lock poisoned".into(),
                    })?;
                    candidates = list_blob_hashes(guard.conn())?;
                }
                let missing = missing_blobs(db, candidates)?;
                wanted.extend(missing.iter().cloned());
                write_msg(
                    stream,
                    key,
                    &TcpMessage::BlobWant {
                        hashes: missing.into_iter().take(BLOB_WANT_MAX).collect(),
                    },
                )
                .await?;
                if wanted.is_empty() {
                    break;
                }
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
                if eof {
                    wanted.remove(&hash);
                    if snapshot_done && wanted.is_empty() {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    if let Some((team_id, team_psk_hex, device_code)) = grant_fields {
        if let Ok(mut inner) = runtime.inner.lock() {
            inner.join_grant = Some(JoinGrant {
                team_id,
                team_name: team_name.unwrap_or_default(),
                team_psk_hex,
                device_code,
                snapshot_applied,
            });
        }
    }
    runtime.emit_sync_applied(snapshot_rows);
    Ok(())
}

pub(super) fn missing_blobs(
    db: &Arc<Mutex<crate::db::Db>>,
    hashes: Vec<String>,
) -> Result<Vec<String>, AppError> {
    let guard = db.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })?;
    let mut missing = Vec::new();
    for hash in hashes {
        if !crate::domain::sync::blobs::blob_exists(guard.conn(), &hash)? {
            missing.push(hash);
        }
    }
    Ok(missing)
}

pub(super) async fn send_blobs<W>(
    db: &Arc<Mutex<crate::db::Db>>,
    stream: &mut W,
    key: &[u8; 32],
    hashes: Vec<String>,
) -> Result<(), AppError>
where
    W: AsyncWriteExt + Unpin,
{
    let root = {
        let guard = db.lock().map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
        guard.paths().root.clone()
    };
    for hash in hashes.into_iter().take(BLOB_WANT_MAX) {
        let path = crate::domain::sync::blobs::blob_absolute(&root, &hash);
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        for (i, chunk) in bytes.chunks(BLOB_CHUNK_MAX).enumerate() {
            let offset = (i * BLOB_CHUNK_MAX) as u64;
            let eof = offset as usize + chunk.len() >= bytes.len();
            write_msg(
                stream,
                key,
                &TcpMessage::BlobChunk {
                    hash: hash.clone(),
                    offset,
                    data: hex::encode(chunk),
                    eof,
                },
            )
            .await?;
        }
    }
    Ok(())
}

pub(super) fn store_blob_chunk(
    db: &Arc<Mutex<crate::db::Db>>,
    hash: &str,
    offset: u64,
    data_hex: &str,
    eof: bool,
) -> Result<bool, AppError> {
    let chunk = hex::decode(data_hex).map_err(|_| AppError::sync_err("Blob data was invalid."))?;
    let guard = db.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })?;
    let complete = crate::domain::sync::blobs::accept_blob_chunk(
        &guard.paths().root,
        hash,
        offset,
        &chunk,
        eof,
    )?;
    if let Some(bytes) = complete {
        crate::domain::sync::blobs::finalize_received_blob(
            &guard.paths().root,
            guard.conn(),
            hash,
            &bytes,
        )?;
        return Ok(true);
    }
    Ok(false)
}
