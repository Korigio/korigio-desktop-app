use super::*;

pub(super) fn to_wire(change: SyncChange) -> ChangeWire {
    ChangeWire {
        id: change.id,
        entity_table: change.entity_table,
        entity_id: change.entity_id,
        op: change.op,
        payload_json: change.payload_json,
        hlc: change.hlc,
        created_at: change.created_at,
    }
}

pub(super) async fn write_msg<W>(
    stream: &mut W,
    key: &[u8; 32],
    msg: &TcpMessage,
) -> Result<(), AppError>
where
    W: AsyncWriteExt + Unpin,
{
    let json = serde_json::to_vec(msg).map_err(|err| AppError::Internal {
        message: format!("encode tcp message: {err}"),
    })?;
    let frame = encode_frame(key, &json)?;
    stream.write_all(&frame).await?;
    Ok(())
}

pub(super) async fn read_msg<R>(stream: &mut R, key: &[u8; 32]) -> Result<TcpMessage, AppError>
where
    R: AsyncReadExt + Unpin,
{
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > crate::sync_net::frame::MAX_PLAINTEXT + 64 {
        return Err(AppError::sync_err("Frame is too large."));
    }
    let mut rest = vec![0u8; len];
    stream.read_exact(&mut rest).await?;
    let mut frame = Vec::with_capacity(4 + len);
    frame.extend_from_slice(&len_buf);
    frame.extend_from_slice(&rest);
    let plain = decode_frame(key, &frame)?;
    serde_json::from_slice(&plain).map_err(|_| AppError::sync_err("Peer sent an invalid message."))
}
