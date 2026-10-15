use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use axum::{Json, Router};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::mysql::{MySqlPoolOptions, MySqlRow};
use sqlx::{MySqlPool, Row};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::time::interval;
use tracing::{error, info, warn};

const GLOBAL_ID: u64 = 1;
const GET_PARAMS: &str = "PARAMS";
const ACT_ASK: &str = "ASK";
const ACT_STOP_CJ: &str = "STOP_CJ";
const ACT_START_CJ: &str = "START_CJ";
const ACT_STARTED: &str = "STARTED";
const ACT_STOPED: &str = "STOPED";
const ACT_RELOAD_PARAMS: &str = "RELOAD_PARAMS";
const SYNC_CAIJI_STATUS: &str = "SYNC_CAIJI_STATUS";
const PING: &str = "ping";
const PONG: &str = "pong";
const EVENT_NAME: &str = "event";
const DEFAULT_HTTP_LISTEN: &str = "127.0.0.1:28888";
const DEFAULT_WS_LISTEN: &str = "0.0.0.0:18888";
const KEEP_ALIVE_TIMEOUT_SECS: u64 = 60;
const ACTIVITY_TIMEOUT_SECS: u64 = 55;

#[derive(Clone)]
struct AppState {
    db: MySqlPool,
    clients: Arc<RwLock<HashMap<String, ClientHandle>>>,
}

#[derive(Clone)]
struct ClientHandle {
    socket_id: String,
    gather_name: String,
    worker_status: Arc<RwLock<bool>>,
    client_not_send_ping_count: Arc<Mutex<u32>>,
    sender: mpsc::UnboundedSender<WsCommand>,
}

#[derive(Debug)]
enum WsCommand {
    Text(String),
    Pong(Vec<u8>),
    Close,
}

#[derive(Debug, Deserialize)]
struct ActionQuery {
    action: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WsConnectQuery {
    cj_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ControlBody {
    #[serde(default)]
    cj_name: Vec<String>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Debug, Serialize)]
struct ApiReply<'a> {
    code: i32,
    msg: &'a str,
}

#[derive(Debug, Serialize)]
struct GatherPayload {
    id: u64,
    cj_name: String,
    cj_count: i64,
    up_count: i64,
    cj_remark: Option<String>,
    cj_status: i32,
    created_at: Option<String>,
    updated_at: Option<String>,
    cookies: Vec<CookiePayload>,
}

#[derive(Debug, Serialize)]
struct CookiePayload {
    id: u64,
    cookie: String,
    dy_name: Option<String>,
    in_used: i32,
    can_use: i32,
    ck_remark: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::from_path(find_env_path()).ok();
    init_tracing();

    let db_url = database_url()?;
    let db = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .context("failed to connect mysql")?;

    let state = AppState {
        db,
        clients: Arc::new(RwLock::new(HashMap::new())),
    };

    let http_addr: SocketAddr = env::var("CAIJI_HTTP_LISTEN")
        .unwrap_or_else(|_| DEFAULT_HTTP_LISTEN.to_string())
        .parse()
        .context("invalid CAIJI_HTTP_LISTEN")?;
    let ws_addr: SocketAddr = env::var("CAIJI_WS_LISTEN")
        .unwrap_or_else(|_| DEFAULT_WS_LISTEN.to_string())
        .parse()
        .context("invalid CAIJI_WS_LISTEN")?;

    let http_app = Router::new()
        .route("/", any(api_handler))
        .with_state(state.clone());
    let ws_app = Router::new()
        .route("/", get(ws_handler))
        .with_state(state.clone());

    let heartbeat_state = state.clone();
    tokio::spawn(async move {
        heartbeat_loop(heartbeat_state).await;
    });

    let http_listener = TcpListener::bind(http_addr).await?;
    let ws_listener = TcpListener::bind(ws_addr).await?;

    info!("caiji rust http listening on {}", http_addr);
    info!("caiji rust websocket listening on {}", ws_addr);

    let http_server = axum::serve(http_listener, http_app);
    let ws_server = axum::serve(ws_listener, ws_app);

    tokio::try_join!(http_server, ws_server)?;
    Ok(())
}

async fn api_handler(
    State(state): State<AppState>,
    Query(query): Query<ActionQuery>,
    request: Request,
) -> Response {
    match api_handler_inner(state, query, request).await {
        Ok(response) => response,
        Err(err) => {
            error!("api handler error: {err:#}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code": 500, "msg": "internal error"})),
            )
                .into_response()
        }
    }
}

async fn api_handler_inner(
    state: AppState,
    query: ActionQuery,
    request: Request,
) -> Result<Response> {
    let action = query.action.unwrap_or_default();
    if action == "caijiList" {
        let clients = state.clients.read().await;
        let list: Vec<String> = clients.keys().cloned().collect();
        return Ok((StatusCode::OK, Json(json!(list))).into_response());
    }

    let body = extract_control_body(request).await?;
    match action.as_str() {
        ACT_START_CJ => {
            push_event_to_clients(&state, &body.cj_name, ACT_START_CJ, body.extra.clone()).await;
            Ok(not_found_style_success().into_response())
        }
        ACT_RELOAD_PARAMS => {
            push_event_to_clients(&state, &body.cj_name, ACT_RELOAD_PARAMS, body.extra.clone()).await;
            Ok(not_found_style_success().into_response())
        }
        ACT_STOP_CJ => {
            push_event_to_clients(&state, &body.cj_name, ACT_STOP_CJ, body.extra.clone()).await;
            Ok((StatusCode::OK, Json(json!(ApiReply { code: 0, msg: "success" }))).into_response())
        }
        SYNC_CAIJI_STATUS => {
            sync_caiji_status(&state, &body.cj_name).await?;
            Ok((StatusCode::OK, Json(json!(ApiReply { code: 0, msg: "success" }))).into_response())
        }
        _ => Ok((StatusCode::OK, Json(json!(ApiReply { code: 0, msg: "success" }))).into_response()),
    }
}

async fn ws_handler(
    State(state): State<AppState>,
    Query(query): Query<WsConnectQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(state, query, socket))
}

async fn handle_socket(state: AppState, query: WsConnectQuery, socket: WebSocket) {
    let gather_name = query.cj_name.unwrap_or_default();
    if gather_name.is_empty() {
        return;
    }

    let socket_id = {
        let clients = state.clients.read().await;
        format!("{}.{}", GLOBAL_ID, clients.len() + 1)
    };

    let (sender, receiver) = mpsc::unbounded_channel::<WsCommand>();
    let worker_status = Arc::new(RwLock::new(false));
    let client_not_send_ping_count = Arc::new(Mutex::new(0_u32));

    let handle = ClientHandle {
        socket_id: socket_id.clone(),
        gather_name: gather_name.clone(),
        worker_status: worker_status.clone(),
        client_not_send_ping_count: client_not_send_ping_count.clone(),
        sender,
    };

    {
        let mut clients = state.clients.write().await;
        clients.insert(gather_name.clone(), handle.clone());
    }

    if let Err(err) = update_caiji_status(&state.db, &gather_name, 1).await {
        error!("failed to mark {gather_name} online: {err:#}");
    }

    let ask_message = json!({
        EVENT_NAME: ACT_ASK,
        "socket_id": socket_id,
        "activity_timeout": ACTIVITY_TIMEOUT_SECS
    })
    .to_string();

    let (write, read) = socket.split();
    let writer_state = state.clone();
    let writer_name = gather_name.clone();
    let writer = tokio::spawn(async move {
        if let Err(err) = socket_writer(write, receiver).await {
            warn!("socket writer closed for {writer_name}: {err:#}");
        }
        remove_client(&writer_state, &writer_name).await;
    });

    if handle.sender.send(WsCommand::Text(ask_message)).is_err() {
        remove_client(&state, &gather_name).await;
        return;
    }

    if let Err(err) = socket_reader(state.clone(), handle, read).await {
        if is_expected_disconnect(&err) {
            info!("socket disconnected for {gather_name}");
        } else {
            warn!("socket reader closed for {gather_name}: {err:#}");
        }
    }

    writer.abort();
    remove_client(&state, &gather_name).await;
}

async fn socket_writer(
    mut write: SplitSink<WebSocket, Message>,
    mut receiver: mpsc::UnboundedReceiver<WsCommand>,
) -> Result<()> {
    while let Some(command) = receiver.recv().await {
        match command {
            WsCommand::Text(message) => write.send(Message::Text(message.into())).await?,
            WsCommand::Pong(payload) => write.send(Message::Pong(payload.into())).await?,
            WsCommand::Close => {
                write.send(Message::Close(None)).await?;
                break;
            }
        }
    }
    Ok(())
}

async fn socket_reader(
    state: AppState,
    client: ClientHandle,
    mut read: SplitStream<WebSocket>,
) -> Result<()> {
    while let Some(message) = read.next().await {
        match message? {
            Message::Text(text) => handle_client_message(&state, &client, &text).await?,
            Message::Binary(bytes) => {
                if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                    handle_client_message(&state, &client, &text).await?;
                }
            }
            Message::Close(_) => break,
            Message::Ping(payload) => {
                let _ = client.sender.send(WsCommand::Pong(payload.to_vec()));
            }
            Message::Pong(_) => {}
        }
    }
    Ok(())
}

async fn handle_client_message(state: &AppState, client: &ClientHandle, text: &str) -> Result<()> {
    {
        let mut count = client.client_not_send_ping_count.lock().await;
        *count = 0;
    }

    let payload: Value = serde_json::from_str(text).unwrap_or(Value::Null);
    let event = payload
        .get(EVENT_NAME)
        .and_then(Value::as_str)
        .unwrap_or_default();

    match event {
        PING => {
            let status = payload
                .get("worker_status")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let now_status = *client.worker_status.read().await;
            if status != now_status {
                sync_worker_status(state, client, status).await?;
            }
            let _ = client
                .sender
                .send(WsCommand::Text(
                    json!({ EVENT_NAME: PONG, "data": [] }).to_string(),
                ));
        }
        GET_PARAMS => {
            fetch_gather_options(state, client).await?;
        }
        ACT_STARTED => {
            update_caiji_status(&state.db, &client.gather_name, 2).await?;
        }
        ACT_STOPED => {
            update_caiji_status(&state.db, &client.gather_name, 1).await?;
        }
        _ => {}
    }

    Ok(())
}

async fn fetch_gather_options(state: &AppState, client: &ClientHandle) -> Result<()> {
    let gather = load_gather_payload(&state.db, &client.gather_name).await?;
    let message = json!({
        "cj_name": client.gather_name,
        "socket_id": client.socket_id,
        EVENT_NAME: GET_PARAMS,
        "data": gather
    })
    .to_string();
    let _ = client.sender.send(WsCommand::Text(message));
    Ok(())
}

async fn push_event_to_clients(
    state: &AppState,
    caiji_list: &[String],
    event: &str,
    options: HashMap<String, Value>,
) {
    let clients = state.clients.read().await;
    for cj in caiji_list {
        if let Some(client) = clients.get(cj) {
            let message = json!({
                EVENT_NAME: event,
                "options": options
            })
            .to_string();
            let _ = client.sender.send(WsCommand::Text(message));
        }
    }
}

async fn sync_caiji_status(state: &AppState, caiji_list: &[String]) -> Result<()> {
    let clients = state.clients.read().await;
    for cj in caiji_list {
        if !clients.contains_key(cj) {
            update_caiji_status(&state.db, cj, 0).await?;
        }
    }
    Ok(())
}

async fn sync_worker_status(state: &AppState, client: &ClientHandle, status: bool) -> Result<()> {
    {
        let mut worker_status = client.worker_status.write().await;
        *worker_status = status;
    }
    let update_status = if status { 2 } else { 1 };
    update_caiji_status(&state.db, &client.gather_name, update_status).await
}

async fn heartbeat_loop(state: AppState) {
    let mut ticker = interval(Duration::from_secs(KEEP_ALIVE_TIMEOUT_SECS / 2));
    loop {
        ticker.tick().await;

        let snapshot = {
            let clients = state.clients.read().await;
            clients.values().cloned().collect::<Vec<_>>()
        };

        for client in snapshot {
            let mut count = client.client_not_send_ping_count.lock().await;
            if *count > 1 {
                let _ = client.sender.send(WsCommand::Close);
                drop(count);
                remove_client(&state, &client.gather_name).await;
                continue;
            }
            *count += 1;
        }
    }
}

async fn remove_client(state: &AppState, gather_name: &str) {
    let removed = {
        let mut clients = state.clients.write().await;
        clients.remove(gather_name)
    };

    if removed.is_some() {
        if let Err(err) = update_caiji_status(&state.db, gather_name, 0).await {
            error!("failed to mark {gather_name} offline: {err:#}");
        }
    }
}

async fn update_caiji_status(db: &MySqlPool, gather_name: &str, status: i32) -> Result<()> {
    sqlx::query("UPDATE dy_caiji SET cj_status = ? WHERE cj_name = ?")
        .bind(status)
        .bind(gather_name)
        .execute(db)
        .await?;
    Ok(())
}

async fn load_gather_payload(db: &MySqlPool, gather_name: &str) -> Result<Option<GatherPayload>> {
    let row = sqlx::query(
        "SELECT id, cj_name, cj_count, up_count, cj_remark, cj_status, created_at, updated_at
         FROM dy_caiji WHERE cj_name = ? LIMIT 1",
    )
    .bind(gather_name)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let cj_id = get_u64(&row, "id")?;
    let cookies = load_cookies(db, cj_id).await?;
    Ok(Some(GatherPayload {
        id: cj_id,
        cj_name: row.try_get("cj_name")?,
        cj_count: get_i64(&row, "cj_count")?,
        up_count: get_i64(&row, "up_count")?,
        cj_remark: row.try_get("cj_remark")?,
        cj_status: row.try_get("cj_status")?,
        created_at: opt_string(&row, "created_at")?,
        updated_at: opt_string(&row, "updated_at")?,
        cookies,
    }))
}

async fn load_cookies(db: &MySqlPool, cj_id: u64) -> Result<Vec<CookiePayload>> {
    let rows = sqlx::query(
        "SELECT c.id, c.cookie, c.dy_name, c.in_used, c.can_use, c.ck_remark, c.created_at, c.updated_at
         FROM dy_cookies c
         INNER JOIN dy_cj_ck rel ON rel.ck_id = c.id
         WHERE rel.cj_id = ?",
    )
    .bind(cj_id)
    .fetch_all(db)
    .await?;

    let mut cookies = Vec::with_capacity(rows.len());
    for row in rows {
        cookies.push(CookiePayload {
            id: get_u64(&row, "id")?,
            cookie: row.try_get("cookie")?,
            dy_name: row.try_get("dy_name")?,
            in_used: row.try_get("in_used")?,
            can_use: row.try_get("can_use")?,
            ck_remark: row.try_get("ck_remark")?,
            created_at: opt_string(&row, "created_at")?,
            updated_at: opt_string(&row, "updated_at")?,
        });
    }
    Ok(cookies)
}

async fn extract_control_body(request: Request) -> Result<ControlBody> {
    let bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .context("failed to read request body")?;
    if bytes.is_empty() {
        return Ok(ControlBody {
            cj_name: Vec::new(),
            extra: HashMap::new(),
        });
    }
    Ok(serde_json::from_slice(&bytes).context("invalid control body json")?)
}

fn database_url() -> Result<String> {
    let host = env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
    let database = env::var("DB_NAME").unwrap_or_else(|_| "dy_server".to_string());
    let username = env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
    let password = env::var("DB_PASS").unwrap_or_else(|_| "root".to_string());
    let ssl_mode = env::var("DB_SSL_MODE").unwrap_or_else(|_| "DISABLED".to_string());
    Ok(format!(
        "mysql://{username}:{password}@{host}:{port}/{database}?charset=utf8mb4&ssl-mode={ssl_mode}"
    ))
}

fn find_env_path() -> String {
    env::current_dir()
        .ok()
        .and_then(|cwd| cwd.join(".env").to_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| ".env".to_string())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();
}

fn not_found_style_success() -> Json<Value> {
    Json(json!({"code": 500, "msg": "采集器不存在"}))
}

fn is_expected_disconnect(err: &anyhow::Error) -> bool {
    let message = format!("{err:#}");
    message.contains("Connection reset by peer")
        || message.contains("Broken pipe")
        || message.contains("os error 104")
        || message.contains("os error 32")
}

fn get_i64(row: &MySqlRow, column: &str) -> Result<i64> {
    if let Ok(value) = row.try_get::<i64, _>(column) {
        return Ok(value);
    }
    if let Ok(value) = row.try_get::<u64, _>(column) {
        return i64::try_from(value).context("u64 value does not fit into i64");
    }
    if let Ok(value) = row.try_get::<u32, _>(column) {
        return Ok(i64::from(value));
    }
    Ok(i64::from(row.try_get::<i32, _>(column)?))
}

fn get_u64(row: &MySqlRow, column: &str) -> Result<u64> {
    if let Ok(value) = row.try_get::<u64, _>(column) {
        return Ok(value);
    }
    if let Ok(value) = row.try_get::<u32, _>(column) {
        return Ok(u64::from(value));
    }
    if let Ok(value) = row.try_get::<i64, _>(column) {
        return u64::try_from(value).context("i64 value is negative and cannot convert to u64");
    }
    let value = row.try_get::<i32, _>(column)?;
    u64::try_from(value).context("i32 value is negative and cannot convert to u64")
}

fn opt_string(row: &MySqlRow, column: &str) -> Result<Option<String>> {
    if let Ok(value) = row.try_get::<Option<String>, _>(column) {
        return Ok(value);
    }
    let value = row
        .try_get::<Option<sqlx::types::chrono::NaiveDateTime>, _>(column)?
        .map(|v| v.format("%Y-%m-%d %H:%M:%S").to_string());
    Ok(value)
}
