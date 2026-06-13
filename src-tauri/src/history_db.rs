use std::fs;
use std::path::PathBuf;

use rusqlite::{params, Connection};
use serde_json::Value;
use tauri::{AppHandle, Manager};

fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {:?}", e))?;
    fs::create_dir_all(&app_dir).map_err(|e| format!("failed to create app data dir: {:?}", e))?;
    Ok(app_dir.join("sqlite.db"))
}

fn open_conn(app: &AppHandle) -> Result<Connection, String> {
    let path = db_path(app)?;
    Connection::open(path).map_err(|e| format!("failed to open sqlite db: {:?}", e))
}

pub fn init_history_table(app: &AppHandle) -> Result<(), String> {
    let conn = open_conn(app)?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS history(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            dy_name TEXT,
            dy_account TEXT,
            seq TEXT,
            xiaohao_diamond_count INTEGER DEFAULT 0,
            huode_diamond_count INTEGER DEFAULT 0,
            count_num INTEGER DEFAULT 0,
            canyu_count INTEGER DEFAULT 0,
            hongbao_canyu_count INTEGER DEFAULT 0,
            hongbao_huode_diamond_count INTEGER DEFAULT 0,
            hongbao_win_count INTEGER DEFAULT 0,
            win_count INTEGER DEFAULT 0,
            create_at TEXT NOT NULL,
            task_name TEXT,
            task_msg TEXT,
            UNIQUE(seq, create_at)
        );
        "#,
    )
    .map_err(|e| format!("failed to init history table: {:?}", e))?;
    Ok(())
}

fn insert_or_ignore_history(
    conn: &Connection,
    dy_name: Option<&str>,
    dy_account: Option<&str>,
    seq: &str,
    create_at: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT OR IGNORE INTO history(dy_name, dy_account, seq, create_at) VALUES (?, ?, ?, ?)",
        params![dy_name, dy_account, seq, create_at],
    )
    .map_err(|e| format!("failed to insert history: {:?}", e))?;
    Ok(())
}

pub fn process_history_event(app: &AppHandle, payload: &Value) -> Result<(), String> {
    let action = payload["action"].as_str().unwrap_or("");
    let create_at = payload["create_at"].as_str().unwrap_or_else(|| {
        // fallback keeps old behavior if create_at omitted
        ""
    });

    let conn = open_conn(app)?;

    if action == "init_seq" {
        if let Some(windows) = payload["windows"].as_array() {
            let create_at = if create_at.is_empty() {
                chrono::Local::now().format("%Y-%m-%d").to_string()
            } else {
                create_at.to_string()
            };
            for win in windows {
                let seq = win["seq"].as_u64().unwrap_or_default().to_string();
                insert_or_ignore_history(&conn, None, None, &seq, &create_at)?;
            }
        }
        return Ok(());
    }

    let seq = payload["seq"].as_str().map(|s| s.to_string()).unwrap_or_else(|| {
        payload["seq"].as_u64().unwrap_or_default().to_string()
    });
    let create_at = if create_at.is_empty() {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    } else {
        create_at.to_string()
    };

    if action == "insert" || action == "update" {
        insert_or_ignore_history(
            &conn,
            payload["dy_name"].as_str(),
            payload["dy_account"].as_str(),
            &seq,
            &create_at,
        )?;
    }

    if action == "delete" {
        let id = payload["id"].as_i64().unwrap_or_default();
        conn.execute("DELETE FROM history WHERE id = ?", params![id])
            .map_err(|e| format!("failed to delete history: {:?}", e))?;
        return Ok(());
    }

    if action != "update" {
        return Ok(());
    }

    let increment_columns = [
        "xiaohao_diamond_count",
        "huode_diamond_count",
        "canyu_count",
        "win_count",
        "hongbao_canyu_count",
        "hongbao_huode_diamond_count",
        "hongbao_win_count",
    ];

    let mut updates: Vec<String> = Vec::new();
    let mut values: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(obj) = payload.as_object() {
        for (key, value) in obj {
            if key == "seq" || key == "create_at" || key == "action" || key == "windows" || key == "id" {
                continue;
            }

            if increment_columns.contains(&key.as_str()) {
                updates.push(format!("{key} = {key} + ?"));
                values.push(rusqlite::types::Value::Integer(value.as_i64().unwrap_or_default()));
            } else {
                updates.push(format!("{key} = ?"));
                let v = match value {
                    Value::String(s) => rusqlite::types::Value::Text(s.clone()),
                    Value::Number(n) => rusqlite::types::Value::Integer(n.as_i64().unwrap_or_default()),
                    Value::Bool(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
                    _ => continue,
                };
                values.push(v);
            }
        }
    }

    if updates.is_empty() {
        return Ok(());
    }

    let sql = format!("UPDATE history SET {} WHERE seq = ? AND create_at = ?", updates.join(", "));
    values.push(rusqlite::types::Value::Text(seq));
    values.push(rusqlite::types::Value::Text(create_at));

    conn.execute(&sql, rusqlite::params_from_iter(values))
        .map_err(|e| format!("failed to update history: {:?}", e))?;
    Ok(())
}
