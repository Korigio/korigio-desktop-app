use super::session::{drive_session, remember_peer_addr, should_dial_on_hello};
use super::*;

pub(crate) async fn lan_loop(runtime: SyncRuntime, db: Arc<Mutex<crate::db::Db>>) {
    let udp = match UdpSocket::bind(("0.0.0.0", UDP_PORT)).await {
        Ok(s) => s,
        Err(_) => {
            if let Ok(mut inner) = runtime.inner.lock() {
                inner.error_message = Some("Could not bind the team discovery port.".into());
                inner.state = "error".into();
            }
            idle_poll(runtime, db).await;
            return;
        }
    };
    let _ = udp.set_broadcast(true);
    let tcp = match TcpListener::bind(("0.0.0.0", TCP_PORT)).await {
        Ok(s) => s,
        Err(_) => {
            if let Ok(mut inner) = runtime.inner.lock() {
                inner.error_message = Some("Could not bind the team sync port.".into());
                inner.state = "error".into();
            }
            idle_poll(runtime, db).await;
            return;
        }
    };

    let mut buf = [0u8; UDP_MAX_BYTES];
    let sleep = tokio::time::sleep(Duration::ZERO);
    tokio::pin!(sleep);
    loop {
        tokio::select! {
            _ = &mut sleep => {
                expire_online(&runtime);
                broadcast_beacon(&udp, &runtime, &db).await;
                sleep.as_mut().reset(tokio::time::Instant::now() + runtime.interval());
            }
            _ = runtime.notify.notified() => {
                expire_online(&runtime);
                broadcast_beacon(&udp, &runtime, &db).await;
                sleep.as_mut().reset(tokio::time::Instant::now() + runtime.interval());
            }
            Ok((len, src)) = udp.recv_from(&mut buf) => {
                handle_udp(&runtime, &db, &buf[..len], src);
            }
            Ok((stream, _)) = tcp.accept() => {
                let runtime = runtime.clone();
                let db = db.clone();
                tokio::spawn(async move {
                    let _ = handle_incoming(runtime, db, stream).await;
                });
            }
        }
    }
}

async fn idle_poll(runtime: SyncRuntime, db: Arc<Mutex<crate::db::Db>>) {
    loop {
        tokio::select! {
            _ = tokio::time::sleep(runtime.interval()) => {}
            _ = runtime.notify.notified() => {}
        }
        let snapshot = snapshot_identity(&db);
        if snapshot.and_then(|s| s.team_id).is_none() {
            if let Ok(mut inner) = runtime.inner.lock() {
                inner.state = "idle".into();
                inner.online.clear();
            }
        }
    }
}

fn expire_online(runtime: &SyncRuntime) {
    let cutoff = match now_utc_rfc3339() {
        Ok(now) => now,
        Err(_) => return,
    };
    if let Ok(mut inner) = runtime.inner.lock() {
        let stale: Vec<String> = inner
            .last_seen
            .iter()
            .filter(|(_, seen)| {
                match time::OffsetDateTime::parse(
                    seen,
                    &time::format_description::well_known::Rfc3339,
                ) {
                    Ok(ts) => {
                        (time::OffsetDateTime::now_utc() - ts).whole_seconds()
                            > runtime.online_window_secs() as i64
                    }
                    Err(_) => true,
                }
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in stale {
            inner.online.remove(&id);
            inner.last_seen.remove(&id);
        }
        let _ = cutoff;
    }
}

async fn broadcast_beacon(udp: &UdpSocket, runtime: &SyncRuntime, db: &Arc<Mutex<crate::db::Db>>) {
    let identity = snapshot_identity(db);
    let (join_team_id, join_pin) = {
        let Ok(inner) = runtime.inner.lock() else {
            return;
        };
        (inner.join_team_id.clone(), inner.join_pin.clone())
    };

    let beacon = if let (Some(team_id), Some(pin)) = (join_team_id, join_pin) {
        let Some(identity) = identity else { return };
        UdpBeacon {
            v: 2,
            kind: "join".into(),
            team_id,
            team_name: String::new(),
            device_id: identity.device_id,
            tcp_port: TCP_PORT,
            pin_hash: Some(hash_team_pin(&pin)),
        }
    } else if let Some(identity) = identity {
        let Some(team_id) = identity.team_id else {
            if let Ok(mut inner) = runtime.inner.lock() {
                inner.state = "idle".into();
                inner.online.clear();
            }
            return;
        };
        if let Ok(mut inner) = runtime.inner.lock() {
            if inner.state == "idle" {
                inner.state = "offline".into();
            }
        }
        UdpBeacon {
            v: 2,
            kind: "hello".into(),
            team_id,
            team_name: identity.team_name.unwrap_or_default(),
            device_id: identity.device_id,
            tcp_port: TCP_PORT,
            pin_hash: None,
        }
    } else {
        return;
    };

    let Ok(bytes) = serde_json::to_vec(&beacon) else {
        return;
    };
    if bytes.len() > UDP_MAX_BYTES {
        return;
    }
    let _ = udp.send_to(&bytes, ("255.255.255.255", UDP_PORT)).await;
    let peer_addrs: Vec<SocketAddr> = {
        let Ok(inner) = runtime.inner.lock() else {
            return;
        };
        inner.peer_addrs.values().copied().collect()
    };
    for addr in peer_addrs {
        let dest = SocketAddr::new(addr.ip(), UDP_PORT);
        let _ = udp.send_to(&bytes, dest).await;
    }
}

fn handle_udp(
    runtime: &SyncRuntime,
    db: &Arc<Mutex<crate::db::Db>>,
    bytes: &[u8],
    src: SocketAddr,
) {
    let Ok(beacon) = serde_json::from_slice::<UdpBeacon>(bytes) else {
        return;
    };
    if beacon.v != 2 {
        return;
    }
    let Some(me) = snapshot_identity(db) else {
        return;
    };
    if beacon.device_id == me.device_id {
        return;
    }
    if device_is_removed(db, &beacon.device_id) {
        return;
    }
    remember_peer_addr(runtime, &beacon.device_id, src);

    if beacon.kind == "hello" && me.team_id.is_none() {
        runtime.remember_nearby(beacon.team_id, beacon.team_name);
        return;
    }

    if beacon.kind == "join" {
        let Some(hash) = beacon.pin_hash.clone() else {
            return;
        };
        let Some(my_team) = me.team_id.as_deref() else {
            return;
        };
        if beacon.team_id != my_team {
            return;
        }
        let Some(pin) = me.team_pin.clone() else {
            return;
        };
        if hash_team_pin(&pin) != hash {
            return;
        }
        spawn_dial(
            runtime.clone(),
            db.clone(),
            src.ip().to_string(),
            beacon.tcp_port,
            beacon.device_id,
            Some(pin),
        );
        return;
    }

    let Some(team_id) = me.team_id.as_deref() else {
        return;
    };
    if beacon.kind != "hello" || beacon.team_id != team_id {
        return;
    }
    if should_dial_on_hello(&me.device_id, &beacon.device_id) {
        spawn_dial(
            runtime.clone(),
            db.clone(),
            src.ip().to_string(),
            beacon.tcp_port,
            beacon.device_id,
            None,
        );
    }
}

pub(super) fn device_is_removed(db: &Arc<Mutex<crate::db::Db>>, device_id: &str) -> bool {
    let Ok(guard) = db.lock() else {
        return false;
    };
    guard
        .conn()
        .query_row(
            "SELECT removed_at FROM team_devices WHERE id = ?1",
            [device_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .ok()
        .flatten()
        .is_some()
}

fn spawn_dial(
    runtime: SyncRuntime,
    db: Arc<Mutex<crate::db::Db>>,
    ip: String,
    port: u16,
    peer_id: String,
    join_pin: Option<String>,
) {
    {
        let Ok(mut inner) = runtime.inner.lock() else {
            return;
        };
        if !inner.connected.insert(peer_id.clone()) {
            return;
        }
    }
    tokio::spawn(async move {
        let addr = format!("{ip}:{port}");
        match tokio::time::timeout(DIAL_TIMEOUT, TcpStream::connect(&addr)).await {
            Ok(Ok(stream)) => {
                let _ = drive_session(runtime.clone(), db, stream, Some(peer_id.clone()), join_pin)
                    .await;
            }
            Ok(Err(_)) | Err(_) => {
                if let Ok(mut inner) = runtime.inner.lock() {
                    if !inner.online.contains(&peer_id) {
                        inner.error_message = Some(CONNECT_FAIL_MSG.into());
                    }
                }
            }
        }
        if let Ok(mut inner) = runtime.inner.lock() {
            inner.connected.remove(&peer_id);
        }
    });
}

pub(super) async fn handle_incoming(
    runtime: SyncRuntime,
    db: Arc<Mutex<crate::db::Db>>,
    stream: TcpStream,
) -> Result<(), AppError> {
    drive_session(runtime, db, stream, None, None).await
}
