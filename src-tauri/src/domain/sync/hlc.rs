//! Hybrid logical clock: (wall_ms, counter, origin_device_id).

use rusqlite::{params, Connection};
use time::OffsetDateTime;

use crate::domain::identity;
use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hlc {
    pub wall: i64,
    pub counter: i64,
    pub origin_device_id: String,
}

impl Hlc {
    pub fn new(wall: i64, counter: i64, origin_device_id: impl Into<String>) -> Self {
        Self {
            wall,
            counter,
            origin_device_id: origin_device_id.into(),
        }
    }
}

/// Greater wall, then counter, then lexicographically greater device_id.
pub fn hlc_greater(a: &Hlc, b: &Hlc) -> bool {
    a.wall > b.wall
        || (a.wall == b.wall && a.counter > b.counter)
        || (a.wall == b.wall && a.counter == b.counter && a.origin_device_id > b.origin_device_id)
}

pub fn hlc_equal(a: &Hlc, b: &Hlc) -> bool {
    a.wall == b.wall && a.counter == b.counter && a.origin_device_id == b.origin_device_id
}

pub fn now_ms() -> Result<i64, AppError> {
    let now = OffsetDateTime::now_utc();
    let millis =
        i64::try_from(now.unix_timestamp_nanos() / 1_000_000).map_err(|_| AppError::Internal {
            message: "clock overflow".into(),
        })?;
    Ok(millis)
}

/// Advance this PC's HLC and persist it on `local_identity`.
pub fn tick_hlc(conn: &Connection) -> Result<(i64, i64, String), AppError> {
    let identity = identity::require_local_identity(conn)?;
    let now = now_ms()?;
    let wall = identity.hlc_wall_ms.max(now);
    let counter = if wall == identity.hlc_wall_ms {
        identity.hlc_counter + 1
    } else {
        0
    };
    conn.execute(
        "UPDATE local_identity SET hlc_wall_ms = ?1, hlc_counter = ?2 WHERE id = 1",
        params![wall, counter],
    )?;
    Ok((wall, counter, identity.device_id))
}

pub fn tick_hlc_value(conn: &Connection) -> Result<Hlc, AppError> {
    let (wall, counter, origin_device_id) = tick_hlc(conn)?;
    Ok(Hlc {
        wall,
        counter,
        origin_device_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greater_prefers_wall_then_counter_then_device() {
        let low = Hlc::new(10, 5, "aaa");
        let high_wall = Hlc::new(11, 0, "aaa");
        let high_counter = Hlc::new(10, 6, "aaa");
        let high_device = Hlc::new(10, 5, "zzz");
        assert!(hlc_greater(&high_wall, &low));
        assert!(!hlc_greater(&low, &high_wall));
        assert!(hlc_greater(&high_counter, &low));
        assert!(hlc_greater(&high_device, &low));
        assert!(!hlc_greater(&low, &high_device));
        assert!(!hlc_greater(&low, &low));
        assert!(hlc_equal(&low, &Hlc::new(10, 5, "aaa")));
    }
}
