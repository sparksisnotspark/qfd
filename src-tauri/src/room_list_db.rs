use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use rusqlite::{params, Connection};
use tauri::{AppHandle, Manager};

use crate::RoomUrlList;

static LAST_CLEANUP_TS: AtomicU64 = AtomicU64::new(0);
const CLEANUP_INTERVAL_SECS: u64 = 600;
const ROOM_EXPIRING_SOON_BUFFER_SECS: u64 = 40;

fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| format!("failed to resolve app data dir: {:?}", e))?;
    fs::create_dir_all(&app_dir).map_err(|e| format!("failed to create app data dir: {:?}", e))?;
    Ok(app_dir.join("sqlite.db"))
}

fn open_conn(app: &AppHandle) -> Result<Connection, String> {
    let path = db_path(app)?;
    Connection::open(path).map_err(|e| format!("failed to open sqlite db: {:?}", e))
}

pub fn init_room_list_table(app: &AppHandle) -> Result<(), String> {
    let conn = open_conn(app)?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS room_list (
            room_url TEXT NOT NULL,
            room_id TEXT NOT NULL,
            web_rid TEXT NOT NULL,
            title TEXT NOT NULL,
            tp TEXT NOT NULL,
            send_time INTEGER NOT NULL,
            used_times INTEGER NOT NULL DEFAULT 0,
            delay INTEGER NOT NULL,
            expired_at INTEGER NOT NULL,
            UNIQUE(room_id, expired_at)
        );
        CREATE INDEX IF NOT EXISTS idx_room_list_expired_at ON room_list(expired_at);
        "#,
    )
    .map_err(|e| format!("failed to init room_list table: {:?}", e))?;
    Ok(())
}

pub fn cleanup_expired_room_list(app: &AppHandle, now_ts: u64) -> Result<usize, String> {
    let last = LAST_CLEANUP_TS.load(Ordering::Relaxed);
    if last != 0 && now_ts.saturating_sub(last) < CLEANUP_INTERVAL_SECS {
        return Ok(0);
    }
    let conn = open_conn(app)?;
    let deleted = conn.execute("DELETE FROM room_list WHERE expired_at <= ?", params![now_ts]).map_err(|e| format!("failed to cleanup expired room_list: {:?}", e))?;
    LAST_CLEANUP_TS.store(now_ts, Ordering::Relaxed);
    Ok(deleted)
}

pub fn cleanup_room_list(app: &AppHandle, now_ts: u64, can_used_times: u64) -> Result<usize, String> {
    let last = LAST_CLEANUP_TS.load(Ordering::Relaxed);
    if last != 0 && now_ts.saturating_sub(last) < CLEANUP_INTERVAL_SECS {
        return Ok(0);
    }
    let conn = open_conn(app)?;
    let deleted = conn.execute("DELETE FROM room_list WHERE expired_at <= ? OR used_times >= ?", params![now_ts, can_used_times]).map_err(|e| format!("failed to cleanup room_list: {:?}", e))?;
    LAST_CLEANUP_TS.store(now_ts, Ordering::Relaxed);
    Ok(deleted)
}

pub fn get_room_list(app: &AppHandle, now_ts: u64) -> Result<Vec<RoomUrlList>, String> {
    let _ = cleanup_expired_room_list(app, now_ts);
    let conn = open_conn(app)?;
    let mut stmt = conn
        .prepare(
            "SELECT room_url, room_id, web_rid, title, tp, send_time, used_times, delay, expired_at
             FROM room_list
             ORDER BY expired_at ASC",
        )
        .map_err(|e| format!("failed to prepare get_room_list: {:?}", e))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(RoomUrlList {
                room_url: row.get(0)?,
                room_id: row.get(1)?,
                web_rid: row.get(2)?,
                title: row.get(3)?,
                tp: row.get(4)?,
                send_time: row.get(5)?,
                used_times: row.get(6)?,
                delay: row.get(7)?,
                expired_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("failed to query room_list: {:?}", e))?;

    let mut data = Vec::new();
    for row in rows {
        data.push(row.map_err(|e| format!("failed to map room_list row: {:?}", e))?);
    }
    Ok(data)
}

pub fn count_active_room_list(app: &AppHandle, now_ts: u64) -> Result<usize, String> {
    let _ = cleanup_expired_room_list(app, now_ts);
    let conn = open_conn(app)?;
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM room_list WHERE expired_at > ?", params![now_ts], |row| row.get(0)).map_err(|e| format!("failed to count room_list: {:?}", e))?;
    Ok(count as usize)
}

pub fn exists_active_room_by_id(app: &AppHandle, room_id: &str, now_ts: u64) -> Result<bool, String> {
    let _ = cleanup_expired_room_list(app, now_ts);
    let conn = open_conn(app)?;
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM room_list WHERE room_id = ? AND expired_at > ?", params![room_id, now_ts], |row| row.get(0)).map_err(|e| format!("failed to check room_list existence: {:?}", e))?;
    Ok(count > 0)
}

pub fn insert_room_list(app: &AppHandle, item: &RoomUrlList, now_ts: u64) -> Result<bool, String> {
    let _ = cleanup_expired_room_list(app, now_ts);
    let conn = open_conn(app)?;
    let affected = conn
        .execute(
            "INSERT OR IGNORE INTO room_list
            (room_url, room_id, web_rid, title, tp, send_time, used_times, delay, expired_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![item.room_url, item.room_id, item.web_rid, item.title, item.tp, item.send_time, item.used_times, item.delay, item.expired_at],
        )
        .map_err(|e| format!("failed to insert room_list: {:?}", e))?;
    Ok(affected > 0)
}

pub fn pop_next_room(app: &AppHandle, now_ts: u64) -> Result<Option<RoomUrlList>, String> {
    let mut conn = open_conn(app)?;
    let tx = conn.transaction().map_err(|e| format!("failed to open transaction: {:?}", e))?;

    tx.execute("DELETE FROM room_list WHERE expired_at <= ?", params![now_ts]).map_err(|e| format!("failed to cleanup room_list in transaction: {:?}", e))?;

    let picked = {
        let mut stmt = tx
            .prepare(
                "SELECT room_url, room_id, web_rid, title, tp, send_time, used_times, delay, expired_at
                 FROM room_list
                 WHERE expired_at > ?
                 ORDER BY expired_at ASC
                 LIMIT 1",
            )
            .map_err(|e| format!("failed to prepare pop_next_room: {:?}", e))?;

        let mut rows = stmt.query(params![now_ts]).map_err(|e| format!("failed to query next room: {:?}", e))?;

        if let Some(row) = rows.next().map_err(|e| format!("failed to read next room: {:?}", e))? {
            Some(RoomUrlList {
                room_url: row.get(0).map_err(|e| format!("failed to read room_url: {:?}", e))?,
                room_id: row.get(1).map_err(|e| format!("failed to read room_id: {:?}", e))?,
                web_rid: row.get(2).map_err(|e| format!("failed to read web_rid: {:?}", e))?,
                title: row.get(3).map_err(|e| format!("failed to read title: {:?}", e))?,
                tp: row.get(4).map_err(|e| format!("failed to read tp: {:?}", e))?,
                send_time: row.get(5).map_err(|e| format!("failed to read send_time: {:?}", e))?,
                used_times: row.get(6).map_err(|e| format!("failed to read used_times: {:?}", e))?,
                delay: row.get(7).map_err(|e| format!("failed to read delay: {:?}", e))?,
                expired_at: row.get(8).map_err(|e| format!("failed to read expired_at: {:?}", e))?,
            })
        } else {
            None
        }
    };

    if let Some(item) = &picked {
        tx.execute("DELETE FROM room_list WHERE room_id = ? AND expired_at = ?", params![item.room_id, item.expired_at]).map_err(|e| format!("failed to delete popped room: {:?}", e))?;
    }

    tx.commit().map_err(|e| format!("failed to commit pop_next_room: {:?}", e))?;
    Ok(picked)
}

pub fn take_next_room_and_increase_used_times(app: &AppHandle, now_ts: u64, can_used_times: u64) -> Result<Option<RoomUrlList>, String> {
    let mut conn = open_conn(app)?;
    let tx = conn.transaction().map_err(|e| format!("failed to open transaction: {:?}", e))?;

    let _ = cleanup_expired_room_list(app, now_ts);
    let min_expired_at = now_ts.saturating_add(ROOM_EXPIRING_SOON_BUFFER_SECS);

    let picked = {
        let mut stmt = tx
            .prepare(
                "SELECT room_url, room_id, web_rid, title, tp, send_time, used_times, delay, expired_at
                 FROM room_list
                 WHERE expired_at > ? AND used_times < ?
                 ORDER BY used_times ASC, expired_at ASC
                 LIMIT 1",
            )
            .map_err(|e| format!("failed to prepare take_next_room: {:?}", e))?;

        let mut rows = stmt.query(params![min_expired_at, can_used_times]).map_err(|e| format!("failed to query next room: {:?}", e))?;

        if let Some(row) = rows.next().map_err(|e| format!("failed to read next room: {:?}", e))? {
            Some(RoomUrlList {
                room_url: row.get(0).map_err(|e| format!("failed to read room_url: {:?}", e))?,
                room_id: row.get(1).map_err(|e| format!("failed to read room_id: {:?}", e))?,
                web_rid: row.get(2).map_err(|e| format!("failed to read web_rid: {:?}", e))?,
                title: row.get(3).map_err(|e| format!("failed to read title: {:?}", e))?,
                tp: row.get(4).map_err(|e| format!("failed to read tp: {:?}", e))?,
                send_time: row.get(5).map_err(|e| format!("failed to read send_time: {:?}", e))?,
                used_times: row.get(6).map_err(|e| format!("failed to read used_times: {:?}", e))?,
                delay: row.get(7).map_err(|e| format!("failed to read delay: {:?}", e))?,
                expired_at: row.get(8).map_err(|e| format!("failed to read expired_at: {:?}", e))?,
            })
        } else {
            None
        }
    };

    if let Some(mut item) = picked {
        let next_used_times = item.used_times + 1;
        tx.execute("UPDATE room_list SET used_times = ? WHERE room_id = ? AND expired_at = ?", params![next_used_times, item.room_id, item.expired_at]).map_err(|e| format!("failed to update used_times: {:?}", e))?;
        item.used_times = next_used_times;
        tx.commit().map_err(|e| format!("failed to commit take_next_room: {:?}", e))?;
        Ok(Some(item))
    } else {
        tx.commit().map_err(|e| format!("failed to commit empty take_next_room: {:?}", e))?;
        Ok(None)
    }
}
