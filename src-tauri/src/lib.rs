use std::{collections::HashMap, process::Child};
#[allow(unused)]
use std::{
    error, thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use mac_address::get_mac_address;
use rand::Rng;
use serde_json::json;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;

use crate::bit::MyError;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod bit;
pub mod gather;
pub mod threads;

#[derive(Default)]
pub struct AppState {
    pub is_spider_running: bool,
    pub is_automation_running: bool,
    pub total: i32,
    pub allowed_count: i32,
    pub limit_room: i32, //限制直播间的数量
    pub fd_count: i32,
    pub chrome_driver_child: HashMap<String, Child>,
    pub bit_window_list: HashMap<String, String>,
    pub room_list: Vec<serde_json::Value>,
    pub room_list_used: Vec<serde_json::Value>,
    pub latest_sync_id: u64,
}

static GLOBAL_APP_STATE: OnceLock<Arc<Mutex<AppState>>> = OnceLock::new();
const APP_STORE_NAME: &str = "app.json";
const ACT_UPDATE: &str = "update";
const ACT_NAME: &str = "action";

#[tauri::command]
async fn window_bounds(p: serde_json::Value) -> Result<String, tauri::Error> {
    println!("window bounds called! {:?}", p);
    let r = bit::window_bounds(p).await;
    println!("r of window bounds return {:?} ", r);
    Ok("Success".into())
}

#[tauri::command]
async fn flexable_windows() -> Result<String, tauri::Error> {
    _ = bit::flexable().await.unwrap();
    Ok("Success".into())
}

#[tauri::command] //查看比特浏览器的
async fn fetch_bitbrowser_group() -> Result<Vec<(String, String)>, String> {
    let res = bit::browser_group_list().await;
    match res {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("error of {:?}", e)),
    }
}

#[tauri::command]
async fn sync_server_room(app: AppHandle, tp: &str) -> Result<(), String> {
    let id_synced;
    {
        let state = GLOBAL_APP_STATE.get().unwrap().clone();
        let state_ = state.lock().unwrap();
        id_synced = state_.latest_sync_id;
    }
    let store = app.app_handle().store("app.json").unwrap();
    let token = store
        .get("token")
        .unwrap()
        .as_str()
        .to_owned()
        .unwrap()
        .to_string();

    let res = gather::fetch_server_rooms(&token, tp, id_synced).await;

    if res.is_ok() {
        let t = res.unwrap();
        let data = t["data"].as_array().unwrap();
        {
            let state = GLOBAL_APP_STATE.get().unwrap().clone();
            let mut state_ = state.lock().unwrap();
            for info in data {
                let room_url = info["room_url"].as_str().unwrap();
                let room_id = info["room_id"].as_str().unwrap();
                let web_rid = info["web_rid"].as_str().unwrap();
                let title = info["title"].as_str().unwrap();
                let send_time = info["send_time"].as_u64().unwrap();
                let delay = info["delay"].as_u64().unwrap();
                let expired_at = info["expired_at"].as_u64().unwrap();
                let value = json!({"room_url": room_url,"room_id":room_id ,"type":"hb", "web_rid":web_rid, "title":title, "send_time":send_time, "delay": delay, "expired_at": expired_at});
                state_.room_list.push(value);
                state_.room_list.sort_by(|a, b| {
                    let ac_ = a["expired_at"].as_u64().unwrap();
                    let bc_ = b["expired_at"].as_u64().unwrap();
                    bc_.cmp(&ac_)
                });
            }

            if data.len() > 0 {
                //保存最后同步的ID，
                state_.latest_sync_id = data[0]["id"].as_u64().unwrap();
            }
        }
    }

    Ok(())
}

#[tauri::command]
async fn get_ip() -> Result<serde_json::Value, String> {
    let res = gather::get_ip().await;
    if res.is_err() {
        let err = res.err().unwrap();
        println!("error is: {:?}", err);
        return Err(format!("{:?}", err));
    }
    Ok(res.unwrap())
}

#[tauri::command]
async fn mac_address() -> String {
    let mac = get_mac_address();
    let s = match mac {
        Ok(Some(ma)) => ma.to_string(),
        Ok(None) => "".to_string(),
        Err(_e) => "".to_string(),
    };
    s
}

#[tauri::command]
async fn bit_group_browsers(id: &str) -> Result<Vec<serde_json::Value>, tauri::Error> {
    let res = bit::browser_group_windows(id.into()).await.unwrap();
    Ok(res)
}

#[tauri::command]
async fn browser_windows_open(browser_id: &str) -> Result<(), tauri::Error> {
    let res = bit::browser_open(browser_id).await.unwrap();
    println!("res is: {:?}", res);
    Ok(())
}

#[tauri::command]
async fn restart_window(app: AppHandle, params: String) -> Result<(), String> {
    let options = serde_json::from_str::<serde_json::Value>(&params).expect("parse json error");
    let seqs = options["seqs"].as_array().unwrap();
    let group_ids = options["group_id"].as_array().unwrap();
    let group_id;
    if group_ids.len() > 0 {
        group_id = group_ids[0].as_str().unwrap().to_string();
    } else {
        return Err("选择要执行的分组".to_string());
    }
    let mut wait_for: Vec<serde_json::Value> = vec![];
    let windows = bit::browser_group_windows(group_id).await.unwrap();

    for win in windows {
        let win_seq = win["seq"].as_u64().unwrap();
        for seq in seqs {
            let seq = seq.as_str().unwrap().parse::<u64>().unwrap();
            if seq == win_seq {
                wait_for.push(json!({"id": win["id"].as_str(), "seq":seq}));
            }
        }
    }

    if wait_for.len() <= 0 {
        return Err("请先选择要重启的窗口".to_string());
    }

    {
        let state = GLOBAL_APP_STATE.get().unwrap().clone();
        let mut state_: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        for seq in seqs {
            if state_.bit_window_list.contains_key(&seq.to_string()) {
                state_.bit_window_list.remove(&seq.to_string());
            }
        }

        for seq in seqs {
            let seq_ = seq.as_str().unwrap().parse::<u64>().unwrap();
            if let Some(child) = state_.chrome_driver_child.get_mut(&seq_.to_string()) {
                child.kill().unwrap();
                child.wait().unwrap();
            }
        }
    }
    for w in &wait_for {
        let _ = bit::browser_close(w["id"].as_str().unwrap()).await;
    }
    return automation_run(app, wait_for, options).await;
}

#[tauri::command]
async fn start_automation(app: AppHandle, params: String) -> Result<(), String> {
    let json = serde_json::from_str::<serde_json::Value>(&params).expect("parse json error");
    let group_ids = json["group_id"].as_array().unwrap();
    let mut group_id = String::from("");
    if group_ids.len() > 0 {
        group_id = group_ids[0].as_str().unwrap().to_string();
    }

    {
        let state = GLOBAL_APP_STATE.get().unwrap().clone();
        let mut state_ = state.lock().unwrap();
        state_.is_automation_running = true;
    }

    let windows = bit::browser_group_windows(group_id).await.unwrap();
    return automation_run(app, windows, json).await;
}

#[tauri::command]
async fn re_start_automation(app: AppHandle, params: String) -> Result<(), String> {
    //重启死掉你窗口，或者你自己弄死的窗口
    //重新启动自动化任务逻辑跟之前的逻辑是一样子的
    let json = serde_json::from_str::<serde_json::Value>(&params).expect("parse json error");
    let group_ids = json["group_id"].as_array().unwrap();
    let mut group_id = String::from("");
    if group_ids.len() > 0 {
        group_id = group_ids[0].as_str().unwrap().to_string();
    }
    let windows: Vec<serde_json::Value> = bit::browser_group_windows(group_id).await.unwrap();
    let ids: Vec<String> = windows
        .iter()
        .map(|x| x["id"].as_str().unwrap().to_string())
        .collect();
    println!("ids: xxxxx{:?}", ids);
    let ids_clone = ids.clone();
    let fetch_obj = bit::bitbroswer_pids_alive(ids).await.unwrap();
    let mut alived_windows: Vec<String> = vec![];
    let mut wait_for_open: Vec<String> = vec![];
    let s = fetch_obj["data"].as_object().unwrap();
    //获取整个数据
    for (k, _) in s.into_iter() {
        alived_windows.push(k.to_string());
    }
    for k in ids_clone.into_iter() {
        if !alived_windows.contains(&k) {
            wait_for_open.push(k);
        }
    }

    let mut wait_for: Vec<serde_json::Value> = vec![];

    let state = GLOBAL_APP_STATE.get().unwrap().clone();
    for bid in wait_for_open {
        let res = bit::browser_detail(bid).await.unwrap();
        println!("browser detail res is: {:?}", res);
        let win = res["data"].as_object().unwrap();
        let seq = win["seq"].as_u64().unwrap();
        let browser_id = win["id"].as_str().unwrap().to_string();
        wait_for.push(json!({"seq":seq, "id":browser_id}));
        let mut state_ = state.lock().unwrap();
        if !state_.bit_window_list.contains_key(&browser_id) {
            state_
                .bit_window_list
                .insert(browser_id.clone(), seq.to_string());
        }

        if state_.chrome_driver_child.contains_key(&seq.to_string()) {
            if let Some(child) = state_.chrome_driver_child.get_mut(&seq.to_string()).take() {
                child.kill().unwrap();
                child.wait().unwrap();
            }
        }
    }

    return automation_run(app, wait_for, json).await;
}

async fn automation_run(
    app: AppHandle,
    windows: Vec<serde_json::Value>,
    json: serde_json::Value,
) -> Result<(), String> {
    let join_fans_group = json["join_fans_group"].as_bool().unwrap_or(false);
    let min_time = json["min_time"].as_u64().unwrap();
    let max_time = json["max_time"].as_u64().unwrap();
    let use_time_strict = json["use_time_strict"].as_bool().unwrap_or(false);
    let gather_type = json["tp"].as_str().unwrap().to_string();
    let store = app.app_handle().store("app.json").unwrap();
    let token = store
        .get("token")
        .unwrap()
        .as_str()
        .to_owned()
        .unwrap()
        .to_string();
    app.emit("db", json!({ACT_NAME:"init_seq", "windows": windows}))
        .unwrap();
    let action_map = HashMap::from([("fd", "福袋任务"), ("hb", "红包任务")]);
    let task_name = action_map[gather_type.as_str()].to_string();
    for win in windows {
        let browser_id = win["id"].as_str().unwrap().to_string();
        let seq = win["seq"].as_u64().unwrap().to_string();
        let res = bit::browser_open(browser_id.as_str()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        let success = res["success"].as_bool().unwrap();
        if success == false {
            return Err(format!(
                "打开浏览器失败，{:?}",
                res["msg"].as_str().unwrap()
            ));
        }

        println!("res of this: {:?}", res);
        let dr = res["data"]["driver"].as_str().unwrap().to_string();
        let http = res["data"]["http"].as_str().unwrap().to_string();
        let tp = gather_type.clone();
        let token = token.clone();
        let app = app.clone();
        let browser_id = browser_id.clone();
        let task_name = task_name.clone();
        let json_ = json.clone();
        {
            let state = GLOBAL_APP_STATE.get().unwrap().clone();
            let mut state_ = state.lock().unwrap();
            let bid_ = browser_id.clone();
            let seq_ = seq.clone();
            state_.bit_window_list.insert(bid_, seq_);
        }

        tokio::spawn(async move {
            let driver = bit::start_selenium_chrome(dr.as_str(), http.as_str(), seq.as_str())
                .await
                .unwrap();

            app.emit("db", json!({ACT_NAME:ACT_UPDATE, "task_name":task_name, "task_msg":"等待70秒全部窗口开启", "seq":seq})).unwrap();
            tokio::time::sleep(Duration::from_secs(70)).await; //
            let r = bit::parse_user_info(&driver).await;
            app.emit("db", json!({ACT_NAME:ACT_UPDATE, "task_name":task_name, "task_msg":"检查登陆", "seq":seq})).unwrap();
            if r.is_err() {
                let e = r.unwrap_err().to_string();
                println!("解析人物出错 {:?}", e);
                app.emit("db", json!({ACT_NAME:ACT_UPDATE, "task_name":task_name, "task_msg":"或需要重新陆", "seq":seq})).unwrap();
                bit::browser_close(browser_id.as_str()).await.unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
                driver.quit().await.unwrap();
                return ();
            }
            app.emit("db", json!({ACT_NAME:ACT_UPDATE, "task_name":task_name, "task_msg":"等待80信息解析出来", "seq":seq})).unwrap();
            tokio::time::sleep(Duration::from_secs(70)).await; //等2分钟再开dd
            let rr = r.unwrap();
            let dy_name = rr.0;
            let dy_account: Vec<&str> = rr.1.as_str().split('：').collect();
            let dy_account = dy_account[1].to_string();
            println!("dy name {:?} dy account {:?}", dy_name, dy_account);
            app.emit("db", json!({ACT_NAME:ACT_UPDATE,"task_name": task_name, "task_msg":"更新抖音名称", "dy_name":dy_name, "dy_account":dy_account, "seq":seq})).unwrap();
            let state_clone = GLOBAL_APP_STATE.get().unwrap().clone();
            let mut has_closed_all = false;

            loop {
                let running = {
                    let s = state_clone.lock().unwrap();
                    s.is_automation_running
                };

                if running == false {
                    println!("停止自动化任务执行");
                    bit::browser_close(browser_id.as_str()).await.unwrap(); //好像没有关闭单个的接口
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    driver.quit().await.unwrap();
                    break;
                }

                let dy_name = dy_name.clone();
                let dy_account = dy_account.clone();
                println!("token is: {:?}", token);
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                if !has_closed_all {
                    //不要一直重新关闭
                    let m = bit::close_other_tab(&driver).await;
                    if m.is_err() {
                        let e = m.unwrap_err();
                        println!("err is: {:?}", e);
                    }
                    has_closed_all = true;
                }

                let room_json;

                {
                    let now = SystemTime::now();
                    let now_ts = now
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    let state = GLOBAL_APP_STATE.get().unwrap().clone();
                    let mut state_ = state.lock().unwrap();
                    let s = state_.room_list.pop();
                    if s.is_none() {
                        let s1 = state_.room_list_used.pop();
                        if s1.is_none() {
                            app.emit("db", json!({"action":"update","dy_name":dy_name,  "dy_account":dy_account, "task_name":task_name, "task_msg":"没找到直播间，等待3秒再试", "seq":seq})).unwrap();
                            continue;
                        } else {
                            room_json = s1.unwrap();
                            state_.room_list_used.push(room_json.clone());
                        }
                    } else {
                        room_json = s.unwrap();
                        state_.room_list_used.push(room_json.clone());
                    }

                    state_
                        .room_list_used
                        .retain(|x| x["expired_at"].as_u64().unwrap() < now_ts);

                    state_
                        .room_list
                        .retain(|x| x["expired_at"].as_u64().unwrap() < now_ts);

                    //刪除過期的
                }

                let data = room_json.clone();
                /*
                let data = room_json["data"].as_array().unwrap();
                if data.is_empty() {
                    app.emit("db", json!({"action":"update","dy_name":dy_name,  "dy_account":dy_account, "task_name":task_name, "task_msg":"没找到直播间，等待3秒再试", "seq":seq})).unwrap();
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }
                */

                let title = data["title"].as_str().unwrap();
                app.emit(
                    "db",
                    json!({"action":"update", "task_msg":format!("获取直播间{:?}", title), "task_name":task_name, "seq":seq}),
                )
                .unwrap();

                let send_time = data.get("send_time").unwrap().as_u64().unwrap();
                let delay = data.get("delay").unwrap().as_u64().unwrap();
                let room_url = data.get("room_url").unwrap().as_str().unwrap().to_string();
                let options = json!({
                    "delay": delay,
                    "send_time": send_time,
                    "dy_name": dy_name,
                    "dy_account": dy_account,
                    "join_fans_group": join_fans_group,
                    "max_time": max_time,
                    "min_time": min_time,
                    "use_time_strict": use_time_strict,
                });
                if tp == "hb" {
                    //let app = app.clone();
                    let x = bit::hb_logic(&app, &driver, room_url.as_str(), seq.as_str(), &options)
                        .await;
                    if x.is_err() {
                        let err = x.err().unwrap();
                        let s = match err {
                            MyError::StringError(es) => es,
                            MyError::TauriError => "Tauri Error".to_string(),
                            MyError::WebDriverError(e) => format!("{:?}", e).to_string(),
                            MyError::RequestError(e) => format!("{:?}", e).to_string(),
                            MyError::SerdeError(e) => format!("{:?}", e).to_string(),
                        };
                        app.emit("db", json!({"action":ACT_UPDATE,"task_msg":format!("E: {:?}", s), "seq":seq}))
                            .unwrap();
                    }
                } else if tp == "fd" {
                    let logic =
                        bit::fd_logic(&app, &driver, &room_url, seq.as_str(), &options).await;
                    if logic.is_err() {
                        let err_str = logic.err().unwrap();
                        let s = match err_str {
                            MyError::StringError(es) => es,
                            MyError::TauriError => "Tauri Error".to_string(),
                            MyError::WebDriverError(e) => format!("{:?}", e).to_string(),
                            MyError::RequestError(e) => format!("{:?}", e).to_string(),
                            MyError::SerdeError(e) => format!("{:?}", e).to_string(),
                        };
                        app.emit(
                            "db",
                            json!({"action":ACT_UPDATE, "seq":seq, "task_msg":format!("F: {:?}", s)}),
                        )
                        .unwrap();
                    } else {
                        let s = format!("福袋执行成功 {:?}", logic.ok().unwrap());
                        app.emit("db", json!({"action":ACT_UPDATE, "seq":seq, "task_msg":s}))
                            .unwrap();
                    }
                }
                ()
            }
        });
    }
    Ok(())
}

#[tauri::command]
async fn stop_automation() -> Result<(), tauri::Error> {
    {
        let state = GLOBAL_APP_STATE.get().unwrap().clone();
        let mut state_ = state.lock().unwrap();
        state_.is_automation_running = false;
        for (k, v) in state_.chrome_driver_child.iter_mut() {
            println!("kill key : {:?}", k);
            v.kill().unwrap();
            v.wait().unwrap();
        }
        state_.bit_window_list.clear();
    }
    let _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let _ = bit::browser_closeall().await;
    Ok(())
}

#[tauri::command]
fn spider_with_multiple_threads_stop(app: AppHandle) -> Result<String, tauri::Error> {
    app.emit("log", "success stop multiple threads spider")
        .unwrap();
    {
        let state = GLOBAL_APP_STATE.get().unwrap().clone();
        let mut state_ = state.lock().unwrap();
        state_.is_spider_running = false;
    }
    Ok("Success stop spider".to_string())
}

#[tauri::command]
fn spider_with_multiple_threads(
    app: AppHandle,
    options: serde_json::Value,
    gather_type: &str,
) -> Result<(), String> {
    let threads = options["thread_number"].as_i64().unwrap();
    let people_limit = options["people_limit"].as_u64().unwrap();
    let store = app.app_handle().store(APP_STORE_NAME).unwrap();
    let token = store.get("token").unwrap().as_str().unwrap().to_string();
    let cookies_stack: Vec<String> = match store.get("cookies") {
        Some(x) => x
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x["ck"].to_string())
            .collect(),
        None => return Err("error cookie is empty~!".to_string()),
    };
    let gather_type = gather_type.to_string();
    let length = cookies_stack.len();
    {
        let state = GLOBAL_APP_STATE.get().unwrap().clone();
        let mut state_ = state.lock().unwrap();
        state_.is_spider_running = true;
    }

    for th in 0..threads {
        let app_ = app.clone();
        let _ = app_.emit("log", format!("线程{:?}开启", th)).unwrap();
        let _token = token.clone();
        let _tp = gather_type.clone();
        let _m_cookie_stack = cookies_stack.clone();
        let _options = options.clone();
        thread::spawn(move || {
            let mut rng = rand::rng();
            let rand_number = rng.random_range(3000..8000);
            std::thread::sleep(Duration::from_millis(rand_number));
            app_.emit("log", format!("线程{:?}开始采集", th)).unwrap();
            loop {
                {
                    let state = GLOBAL_APP_STATE.get().unwrap().clone();
                    let state_ = state.lock().unwrap();
                    if state_.is_spider_running == false {
                        app_.emit("log", format!("结束{:?}线程", th)).unwrap();
                        break;
                    }
                }

                let cookie_idx: usize;
                {
                    cookie_idx = rng.random_range(0.._m_cookie_stack.len());
                }

                let rooms = gather::sync_fetch_dy_room(
                    _m_cookie_stack
                        .get(cookie_idx)
                        .unwrap()
                        .as_str()
                        .to_string(),
                );

                if rooms.is_err() {
                    let err = rooms.err().unwrap();
                    app_.emit("log", format!("some error is: {:?}", err))
                        .unwrap();
                    std::thread::sleep(Duration::from_secs(3));
                    continue;
                }

                let rooms = rooms.unwrap();
                let data = rooms["data"].as_array();
                if data.is_none() {
                    app_.emit("log", "no data for room").unwrap();
                    std::thread::sleep(Duration::from_secs(3));
                    continue;
                }

                let mut i = 0;
                let mut room_count = 0;
                for room in data.unwrap() {
                    let web_rid = room["web_rid"].as_str().unwrap();
                    let room_id = room["data"]["id_str"].as_str().unwrap();
                    let title = room["data"]["title"].as_str().unwrap();
                    let owner_nickname = room["data"]["owner"]["nickname"].as_str().unwrap();
                    let user_count = room["data"]["user_count"].as_i64().unwrap();
                    //观看人数
                    let display_value = room["data"]["room_view_stats"]["display_value"]
                        .as_u64()
                        .unwrap();

                    if people_limit < display_value {
                        {
                            let state = GLOBAL_APP_STATE.get().unwrap().clone();
                            let mut state_ = state.lock().unwrap();
                            state_.limit_room += 1;
                            state_.total += 1;
                            app_.emit("spider", json!({"total": state_.total, "allowed_count":state_.allowed_count, "fd_count":state_.fd_count})).unwrap();
                        }
                        continue;
                    }

                    let index = i % length;
                    let cookie__ = _m_cookie_stack.get(index).unwrap().to_string();
                    let mut rng = rand::rng();
                    let rand_number = rng.random_range(800..=2000);
                    let json_data = json!({
                        "room_id": room_id.to_string(),
                        "web_rid":web_rid.to_string(),
                        "title":title.to_string(),
                        "type":_tp,
                        "cookie":cookie__,
                        "token": _token,
                        "sleep":rand_number,
                        "display_value": display_value,
                        "options":_options,
                    });

                    threads::execute_gather(json_data, &app_);
                    i = i + 1;
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    room_count = room_count + 1;

                    {
                        let state = GLOBAL_APP_STATE.get().unwrap().clone();
                        let mut state_ = state.lock().unwrap();
                        state_.total += room_count;
                        app_.emit(
                            "log",
                            format!("本轮采集完成，线程结束，采集房间数: {}", room_count),
                        )
                        .unwrap();
                        app_.emit("spider", json!({"total":state_.total, "allowed_count": state_.allowed_count, "fd_count":state_.fd_count})).unwrap();
                    }
                    let payload = format!(
                        "web rid: {:?} room id: {:?} title: {:?} owner_nick {:?} user count {:?}",
                        web_rid, room_id, title, owner_nickname, user_count
                    );
                    app_.emit("log", payload).unwrap();
                }
            }
        });
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    builder
        .setup(|app| {
            let app_state = Arc::new(Mutex::new(AppState::default()));
            // store in tauri managed state as well as global OnceLock for handlers
            app.manage(app_state.clone());
            let _ = GLOBAL_APP_STATE.set(app_state);
            Ok(())
        })
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            flexable_windows,
            fetch_bitbrowser_group,
            spider_with_multiple_threads_stop,
            spider_with_multiple_threads,
            get_ip,
            restart_window,
            sync_server_room,
            bit_group_browsers,
            browser_windows_open,
            start_automation,
            stop_automation,
            mac_address,
            re_start_automation,
            window_bounds,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
