use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use rand::prelude::IndexedRandom;
use reqwest::{
    Client,
    header::{CONTENT_TYPE, COOKIE, HeaderMap, HeaderValue, REFERER, USER_AGENT},
};
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::{Mutex, RwLock, mpsc, watch};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

const VERSION: &str = "1.0.4";
const ACT_ASK: &str = "ASK";
const ACT_PARAMS: &str = "PARAMS";
const ACT_STOP_CJ: &str = "STOP_CJ";
const ACT_START_CJ: &str = "START_CJ";
const ACT_RELOAD_PARAMS: &str = "RELOAD_PARAMS";
const ACT_UPDATE_STATUS: &str = "UPDATE_STATUS";
const BASIC_SERVER_URL: &str = "http://127.0.0.1:8080";
const DEFAULT_SERVER_URL: &str = "ws://127.0.0.1:18888";
const INIT_DELAY: u64 = 2;
const MAX_DELAY: u64 = 30;
const HEARTBEAT_INTERVAL_SECS: u64 = 10;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    cj_name: String,
    #[arg(long, default_value = DEFAULT_SERVER_URL)]
    ws_url: String,
}

#[derive(Clone)]
struct AppState {
    cj_name: String,
    ws_url: String,
    cookies: Arc<RwLock<Vec<CookieItem>>>,
    working: Arc<AtomicBool>,
    outbound_tx: Arc<Mutex<Option<mpsc::UnboundedSender<String>>>>,
    worker_manager: Arc<Mutex<WorkerManager>>,
    http_client: Client,
}

impl AppState {
    fn make_message(&self, extra: Value) -> String {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or_default();
        let mut base = json!({
            "event": ACT_PARAMS,
            "cj_name": self.cj_name,
            "ts": ts,
        });
        if let (Some(base_obj), Some(extra_obj)) = (base.as_object_mut(), extra.as_object()) {
            for (k, v) in extra_obj {
                base_obj.insert(k.clone(), v.clone());
            }
        }
        base.to_string()
    }

    async fn send_ws_message(&self, text: String) {
        let guard = self.outbound_tx.lock().await;
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(text);
        }
    }

    async fn update_working_status(&self) {
        let payload = self.make_message(json!({
            "working_status": self.working.load(Ordering::SeqCst),
            "event": ACT_UPDATE_STATUS
        }));
        self.send_ws_message(payload).await;
    }
}

#[derive(Default)]
struct WorkerManager {
    stop_tx: Option<watch::Sender<bool>>,
    handles: Vec<JoinHandle<()>>,
}

impl WorkerManager {
    async fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(true);
        }
        for handle in self.handles.drain(..) {
            let _ = handle.await;
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CookieItem {
    #[serde(default)]
    id: i64,
    #[serde(default)]
    cookie: String,
}

#[derive(Debug, Clone)]
struct WorkerConfig {
    token: String,
    people_limit: i64,
    min_thread_sleep: u64,
    max_thread_sleep: u64,
    min_diamond: i64,
    avg_diamond: f64,
    min_packet_number: i64,
}

#[derive(Debug, Serialize)]
struct UploadPacket {
    room_url: String,
    title: String,
    #[serde(rename = "type")]
    packet_type: String,
    send_time: f64,
    delay_time: i64,
    room_id: String,
    web_rid: String,
    used_times: i64,
    pack_count: i64,
    diamond_count: i64,
    expired_at: f64,
}

#[derive(Debug, Deserialize)]
struct FeedResponse {
    #[serde(default)]
    data: Vec<FeedRoom>,
}

#[derive(Debug, Deserialize)]
struct FeedRoom {
    #[serde(default)]
    data: FeedRoomData,
    #[serde(default)]
    web_rid: String,
}

#[derive(Debug, Default, Deserialize)]
struct FeedRoomData {
    #[serde(default)]
    id_str: String,
    #[serde(default)]
    room_view_stats: RoomViewStats,
}

#[derive(Debug, Default, Deserialize)]
struct RoomViewStats {
    #[serde(default, deserialize_with = "deserialize_string_or_number")]
    display_value: String,
}

fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        Value::Null => Ok(String::new()),
        other => Err(de::Error::custom(format!("expected string or number, got {other}"))),
    }
}

fn seconds_value_to_millis(value: Option<&Value>, default_ms: u64) -> u64 {
    let seconds = value.and_then(Value::as_f64).or_else(|| value.and_then(Value::as_i64).map(|v| v as f64)).unwrap_or(default_ms as f64 / 1000.0);
    (seconds.max(0.0) * 1000.0).round() as u64
}

fn random_sleep_millis(min_ms: u64, max_ms: u64) -> u64 {
    let max_ms = max_ms.max(min_ms);
    if min_ms == max_ms { min_ms } else { rand::random_range(min_ms..=max_ms) }
}

#[derive(Debug, Deserialize)]
struct LuckyBoxResponse {
    #[serde(default)]
    data: Vec<LuckyBoxItem>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct LuckyBoxItem {
    #[serde(default)]
    box_type: i64,
    #[serde(default)]
    business_type: i64,
    #[serde(default)]
    biz_extra: String,
    #[serde(default)]
    diamond_count: i64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    send_time: i64,
    #[serde(default)]
    delay_time: i64,
}

#[derive(Debug, Default, Deserialize)]
struct BizExtra {
    #[serde(default)]
    tags: serde_json::Map<String, Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("starting caiji version {VERSION}");

    let http_client = build_http_client()?;
    let state = AppState {
        cj_name: args.cj_name,
        ws_url: args.ws_url,
        cookies: Arc::new(RwLock::new(Vec::new())),
        working: Arc::new(AtomicBool::new(false)),
        outbound_tx: Arc::new(Mutex::new(None)),
        worker_manager: Arc::new(Mutex::new(WorkerManager::default())),
        http_client,
    };

    tokio::select! {
        result = run_socket_loop(state.clone()) => result,
        _ = tokio::signal::ctrl_c() => {
            shutdown(&state).await;
            Ok(())
        }
    }
}

fn build_http_client() -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36"),
    );
    Client::builder().default_headers(headers).build().context("failed to build http client")
}

async fn run_socket_loop(state: AppState) -> Result<()> {
    let mut current_delay = INIT_DELAY;
    loop {
        if let Err(err) = connect_and_run(state.clone()).await {
            eprintln!("exception for this {err:#}");
        }
        sleep(Duration::from_secs(current_delay)).await;
        current_delay = (current_delay * 2).min(MAX_DELAY);
    }
}

async fn connect_and_run(state: AppState) -> Result<()> {
    println!("connect on run!");
    let ws_url = format!("{}/?cj_name={}", state.ws_url.trim_end_matches('/'), state.cj_name);
    let _ = Url::parse(&ws_url).context("invalid websocket url")?;
    let (stream, _) = connect_async(&ws_url).await.context("websocket connect failed")?;
    println!("on open execute!");

    let (mut write, mut read) = stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    {
        let mut guard = state.outbound_tx.lock().await;
        *guard = Some(tx.clone());
    }

    let writer = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if write.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let heartbeat_state = state.clone();
    let heartbeat = tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(HEARTBEAT_INTERVAL_SECS)).await;
            let payload = heartbeat_state.make_message(json!({
                "event": "ping",
                "cj_name": heartbeat_state.cj_name,
                "worker_status": heartbeat_state.working.load(Ordering::SeqCst)
            }));
            heartbeat_state.send_ws_message(payload).await;
        }
    });

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if let Err(err) = handle_message(&state, &text).await {
                    eprintln!("error except ------- {err:#}");
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(err) => {
                eprintln!("error {err}");
                break;
            }
        }
    }

    println!("连接关闭");
    heartbeat.abort();
    writer.abort();
    {
        let mut guard = state.outbound_tx.lock().await;
        *guard = None;
    }
    Ok(())
}

async fn handle_message(state: &AppState, message: &str) -> Result<()> {
    let msg: Value = serde_json::from_str(message).context("failed to parse websocket message")?;
    let event = msg.get("event").and_then(Value::as_str).unwrap_or_default();

    match event {
        ACT_PARAMS => {
            let cookies: Vec<CookieItem> = serde_json::from_value(msg.get("data").and_then(|v| v.get("cookies")).cloned().unwrap_or_else(|| Value::Array(Vec::new()))).unwrap_or_default();
            println!("Received parameters: ------------- {} cookies", cookies.len());
            let mut guard = state.cookies.write().await;
            *guard = cookies;
        }
        ACT_ASK => {
            let payload = json!({
                "event": ACT_PARAMS,
                "cj_name": state.cj_name,
            })
            .to_string();
            state.send_ws_message(payload).await;
        }
        ACT_UPDATE_STATUS => {
            println!("update status");
            state.update_working_status().await;
        }
        ACT_START_CJ => {
            println!("message for start cj: {msg}");
            let options = msg.get("options").cloned().unwrap_or_else(|| json!({}));
            let config = WorkerConfig {
                token: options.get("token").and_then(Value::as_str).unwrap_or("SPARKS1").to_string(),
                people_limit: options.get("people_limit").and_then(Value::as_i64).unwrap_or(100),
                min_thread_sleep: seconds_value_to_millis(options.get("min_thread_time"), 1000),
                max_thread_sleep: seconds_value_to_millis(options.get("max_thread_time"), 3000),
                min_diamond: options.get("min_diamond").and_then(Value::as_i64).unwrap_or(50),
                avg_diamond: options.get("min_avg_diamond").and_then(Value::as_f64).unwrap_or(4.0),
                min_packet_number: options.get("min_packet_number").and_then(Value::as_i64).unwrap_or(1),
            };
            let num_threads = options.get("thread_number").and_then(Value::as_u64).unwrap_or(8) as usize;
            start_workers(state.clone(), num_threads, config).await;
            state.send_ws_message(json!({"event": "STARTED", "cj_name": state.cj_name, "msg": "started"}).to_string()).await;
        }
        ACT_STOP_CJ => {
            println!("-----------收到停止采集指令，正在停止采集线程-------------");
            stop_workers(state).await;
            state.send_ws_message(json!({"event": "STOPED", "cj_name": state.cj_name, "msg": "stoped"}).to_string()).await;
        }
        ACT_RELOAD_PARAMS => {
            state.send_ws_message(state.make_message(json!({"event": ACT_PARAMS, "cj_name": state.cj_name}))).await;
        }
        "pong" => println!("收到pong响应"),
        _ => {}
    }

    Ok(())
}

async fn start_workers(state: AppState, num_threads: usize, config: WorkerConfig) {
    stop_workers(&state).await;
    state.working.store(true, Ordering::SeqCst);
    let (stop_tx, stop_rx) = watch::channel(false);

    let mut handles = Vec::with_capacity(num_threads);
    for i in 0..num_threads {
        let worker_state = state.clone();
        let worker_config = config.clone();
        let worker_cookies = state.cookies.clone();
        let worker_name = format!("{}-{i}", state.cj_name);
        let rx = stop_rx.clone();
        let handle = tokio::spawn(async move {
            let worker = CaijiWorker::new(worker_name, worker_state.http_client.clone(), worker_config, worker_cookies);
            worker.run(rx).await;
        });
        handles.push(handle);
    }

    let mut manager = state.worker_manager.lock().await;
    manager.stop_tx = Some(stop_tx);
    manager.handles = handles;
}

async fn stop_workers(state: &AppState) {
    state.working.store(false, Ordering::SeqCst);
    let mut manager = state.worker_manager.lock().await;
    manager.stop().await;
}

async fn shutdown(state: &AppState) {
    stop_workers(state).await;
    let mut guard = state.outbound_tx.lock().await;
    *guard = None;
}

struct CaijiWorker {
    name: String,
    client: Client,
    config: WorkerConfig,
    cookies: Arc<RwLock<Vec<CookieItem>>>,
}

impl CaijiWorker {
    fn new(name: String, client: Client, config: WorkerConfig, cookies: Arc<RwLock<Vec<CookieItem>>>) -> Self {
        println!(
            "Initialized CaijiWorker {} with token {}, people_limit {}, min_thread_sleep {}, max_thread_sleep {}, min_diamond {}, avg_diamond {}, min_packet_number {}",
            name, config.token, config.people_limit, config.min_thread_sleep, config.max_thread_sleep, config.min_diamond, config.avg_diamond, config.min_packet_number
        );
        Self { name, client, config, cookies }
    }

    async fn run(self, mut stop_rx: watch::Receiver<bool>) {
        println!("线程 {} 开始采集红包信息...", self.name);
        loop {
            if *stop_rx.borrow() {
                println!("线程 {} 收到停止信号，正在停止采集...", self.name);
                break;
            }

            let maybe_cookie = {
                let cookies = self.cookies.read().await;
                let mut rng = rand::rng();
                cookies.choose(&mut rng).cloned()
            };

            let Some(cookie_item) = maybe_cookie else {
                eprintln!("no cookies available, worker {}", self.name);
                sleep(Duration::from_secs(3)).await;
                continue;
            };
            let sleep_ms = random_sleep_millis(self.config.min_thread_sleep, self.config.max_thread_sleep);
            println!("线程 {} 随机等待 {} 毫秒后继续采集...", self.name, sleep_ms);

            tokio::select! {
                _ = sleep(Duration::from_millis(sleep_ms)) => {}
                changed = stop_rx.changed() => {
                    if changed.is_ok() && *stop_rx.borrow() {
                        println!("线程 {} 收到停止信号，正在停止采集...", self.name);
                        break;
                    }
                }
            }

            if let Err(err) = self.process_cookie(&cookie_item, &mut stop_rx, self.config.min_thread_sleep, self.config.max_thread_sleep).await {
                eprintln!("Error fetching room list: {err:#}");
                if is_room_list_connection_reset(&err) {
                    let removed = self.remove_cookie(&cookie_item).await;
                    if removed {
                        eprintln!("worker {} removed cookie {} from shared pool after connection reset", self.name, cookie_item.id);
                    }
                    continue;
                }
                if cookie_item.id != 0 {
                    let _ = self.update_ck_status(cookie_item.id).await;
                }
            }
        }
    }

    async fn process_cookie(&self, cookie_item: &CookieItem, stop_rx: &mut watch::Receiver<bool>, min_thread_sleep: u64, max_thread_sleep: u64) -> Result<()> {
        let room_list = self.fetch_room_list(&cookie_item.cookie).await?;
        for room in room_list {
            if *stop_rx.borrow() {
                println!("线程 {} 收到停止信号，正在停止采集...", self.name);
                break;
            }

            let room_id = room.data.id_str;
            let web_rid = room.web_rid;
            let display_value = room.data.room_view_stats.display_value.parse::<i64>().unwrap_or(100);
            if display_value > self.config.people_limit {
                println!("Room {} with web_rid {} has {} viewers, skipping red packet logic", room_id, web_rid, display_value);
                continue;
            }

            let red_packet_info = self.fetch_red_packet_info(&room_id, &web_rid, &cookie_item.cookie).await?;
            self.red_packet_logic(&room_id, &web_rid, red_packet_info).await?;
            println!("线程 {} 采集完成一次，等待下一次采集... {} {}", self.name, self.config.max_thread_sleep, self.config.min_thread_sleep);

            let room_sleep_ms = random_sleep_millis(min_thread_sleep, max_thread_sleep);
            tokio::select! {
                _ = sleep(Duration::from_millis(room_sleep_ms)) => {}
                changed = stop_rx.changed() => {
                    if changed.is_ok() && *stop_rx.borrow() {
                        println!("线程 {} 收到停止信号，正在停止采集...", self.name);
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    async fn update_to_server(&self, data: &UploadPacket) -> Result<()> {
        let url = format!("{BASIC_SERVER_URL}/api/addRoom/");
        println!("上传服务器数据 {}", serde_json::to_string(data).unwrap_or_default());
        self.client
            .post(url)
            .header("app-key", &self.config.token)
            .json(data)
            .send()
            .await
            .context("upload to server failed")?
            .error_for_status()
            .context("upload returned error status")?;
        Ok(())
    }

    async fn update_ck_status(&self, ck_id: i64) -> Result<()> {
        let url = format!("{BASIC_SERVER_URL}/caiji/updateCookieStatus");
        self.client
            .post(url)
            .header("app-key", &self.config.token)
            .form(&[("ck_id", ck_id.to_string())])
            .send()
            .await
            .context("update cookie status failed")?
            .error_for_status()
            .context("cookie status returned error status")?;
        Ok(())
    }

    async fn remove_cookie(&self, cookie_item: &CookieItem) -> bool {
        let mut cookies = self.cookies.write().await;
        let before = cookies.len();
        if cookie_item.id != 0 {
            cookies.retain(|item| item.id != cookie_item.id);
        } else {
            cookies.retain(|item| item.cookie != cookie_item.cookie);
        }
        before != cookies.len()
    }

    async fn fetch_room_list(&self, cookie: &str) -> Result<Vec<FeedRoom>> {
        let url = "https://live.douyin.com/webcast/feed/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&enter_from=page_refresh&cookie_enabled=true&screen_width=1920&screen_height=1080&browser_language=zh-CN&browser_platform=Win32&browser_name=Chrome&browser_version=142.0.0.0&channel=channel_pc_web&request_tag_from=web&need_map=1&liveid=1&is_draw=1&inner_from_drawer=0&custom_count=8&action=load_more&action_type=loadmore&enter_source=web_homepage_hot_web_live_card&source_key=web_homepage_hot_web_live_card&msToken=a-or-PT8jAMKz22HtrVQDSSWCOGlZKCU7a1hiuwIDwJ7yA9-XGhds8zVFbXUpFs3n6OsCzWSy8jBvR1sfzCQq6YF_z1KfTzLla5k9q23aw4LDXtUwbGQdR0LofaMXMmqvYAoEKyEnfugENEkQYN-ICO1nQvEVbeAisqh4jX1C_tC13TSBiubQA%3D%3D&a_bogus=mvsVgzSLYZ5bFdMbYcVMe4FUJzV%2FNTuy7lTxWNcuCxOTOqeOMbNsMcbpjxzNgy8l2Rpki9V7qnMAbxxcToUk1lHpKmhkSqGRgsVAVhXo8qiDbzt0LHbkewWzLw0x0csqa55XiIWIhUrogjVAiNdu%2FQA9tKoe5bSBF3O6k2TbO9sX1F6ADZn1PdG2OhJq0D%3D%3D";
        let resp = self
            .client
            .get(url)
            .header(REFERER, "https://live.douyin.com/?from_nav=1")
            .header(COOKIE, cookie)
            .send()
            .await
            .context("fetch room list failed")?
            .error_for_status()
            .context("fetch room list returned error status")?;
        let body = resp.text().await.context("read room list body failed")?;
        let parsed: FeedResponse = serde_json::from_str(&body).context("parse room list json failed")?;
        Ok(parsed.data)
    }

    async fn fetch_red_packet_info(&self, room_id: &str, web_rid: &str, cookie: &str) -> Result<Vec<LuckyBoxItem>> {
        let url = format!(
            "https://live.douyin.com/webcast/luckybox/box/list/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&enter_from=web_live&cookie_enabled=true&screen_width=1680&screen_height=1050&browser_language=zh-CN&browser_platform=MacIntel&browser_name=Chrome&browser_version=142.0.0.0&room_id={room_id}"
        );
        let referer = format!("https://live.douyin.com/{web_rid}");
        let resp = self.client.get(url).header(COOKIE, cookie).header(REFERER, referer).send().await.context("fetch red packet info failed")?;
        let status = resp.status();
        let body = resp.text().await.context("read red packet response body failed")?;
        println!("fetch_red_packet_info room_id={room_id} web_rid={web_rid} status={status} body={body}");
        if !status.is_success() {
            anyhow::bail!("fetch red packet returned error status {status}");
        }
        let parsed: LuckyBoxResponse = serde_json::from_str(&body).context("parse red packet json failed")?;
        Ok(parsed.data)
    }

    async fn red_packet_logic(&self, room_id: &str, web_rid: &str, data: Vec<LuckyBoxItem>) -> Result<()> {
        if data.is_empty() {
            println!("no data for room {} with web_rid {}, skipping...", room_id, web_rid);
            return Ok(());
        }

        let mut all_gift = true;
        let mut packet_list = Vec::new();

        for item in data {
            if item.box_type == 0 {
                all_gift = false;
            }

            if item.box_type == 1 {
                println!("box_type {} is not red packet, 礼物红包 skipping", item.box_type);
                continue;
            }

            let biz_extra: BizExtra = serde_json::from_str(&item.biz_extra).unwrap_or_default();
            let pack_count = biz_extra.tags.get(&item.business_type.to_string()).and_then(Value::as_str).unwrap_or("1").parse::<i64>().unwrap_or(2);
            println!("got packet count is: {:?}", pack_count);

            if pack_count < self.config.min_packet_number {
                println!("pack count {} less {} skip", pack_count, self.config.min_packet_number);
                continue;
            }

            if item.diamond_count < self.config.min_diamond {
                println!("diamond count {} less {} skip", item.diamond_count, self.config.min_diamond);
                continue;
            }

            let avg_diamond = item.diamond_count as f64 / pack_count as f64;
            if avg_diamond < self.config.avg_diamond {
                println!("avg diamond {} less {} skip", avg_diamond, self.config.avg_diamond);
                continue;
            }

            let send_time = item.send_time as f64 / 1000.0;
            let expired_at = send_time + item.delay_time as f64;
            packet_list.push(UploadPacket {
                room_url: format!("https://live.douyin.com/{web_rid}"),
                title: item.title,
                packet_type: "hb".to_string(),
                send_time,
                delay_time: item.delay_time,
                room_id: room_id.to_string(),
                web_rid: web_rid.to_string(),
                used_times: 0,
                pack_count,
                diamond_count: item.diamond_count,
                expired_at,
            });
        }

        if all_gift {
            println!("Room {} with web_rid {} has only gift boxes, skipping red packet logic", room_id, web_rid);
            return Ok(());
        }

        if let Some(pack) = packet_list.into_iter().max_by(|a, b| a.expired_at.total_cmp(&b.expired_at)) {
            self.update_to_server(&pack).await?;
        }

        Ok(())
    }
}

fn is_room_list_connection_reset(err: &anyhow::Error) -> bool {
    let message = format!("{err:#}").to_lowercase();
    message.contains("fetch room list failed")
        && (message.contains("os error 10054") || message.contains("os error 104") || message.contains("connection reset by peer") || message.contains("forcibly closed") || message.contains("远程主机强迫关闭"))
}

#[cfg(test)]
mod tests {
    use super::is_room_list_connection_reset;
    use anyhow::Context;

    #[test]
    fn detects_room_list_connection_reset() {
        let err = Err::<(), _>(anyhow::anyhow!("connection reset (os error 10054)")).context("fetch room list failed").unwrap_err();
        assert!(is_room_list_connection_reset(&err));
    }

    #[test]
    fn ignores_unrelated_connection_reset() {
        let err = Err::<(), _>(anyhow::anyhow!("connection reset (os error 10054)")).context("upload to server failed").unwrap_err();
        assert!(!is_room_list_connection_reset(&err));
    }
}
