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
}

//static mut SPIDER_STOP: bool = false;
//static mut AUTOMATION_STOP: bool = false;

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
fn sync_start_spider(
    app: AppHandle,
    options: serde_json::Value,
    gather_type: &str,
) -> Result<String, String> {
    let threads = options["thread_number"].as_i64().unwrap();
    println!("what is this params is: {:?}", threads);
    let _ = app.emit("notify", "start spider success!"); //直接发射，不需要获得结果
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
    println!("gather type is: {:?}", gather_type);
    let length = cookies_stack.len();
    {
        let state = GLOBAL_APP_STATE.get().unwrap().clone();
        let mut state_ = state.lock().unwrap();
        state_.is_spider_running = true;
    }

    println!("start spider threads number is go here?");
    thread::spawn(move || {
        let pool = threads::ThreadPool::new(threads as usize);
        let gather_type_ = gather_type.clone();
        let token_ = token.clone();
        let m_cookies_stack = cookies_stack.clone();
        let cookie_ = m_cookies_stack[0].as_str();
        loop {
            {
                let state = GLOBAL_APP_STATE.get().unwrap().clone();
                let state_ = state.lock().unwrap();
                if state_.is_spider_running == false {
                    println!("停止线程执行");
                    break;
                }
            }
            app.emit("log", "Go... spider").unwrap();

            let rooms: Result<serde_json::Value, bit::MyError> =
                gather::sync_fetch_dy_room(cookie_.to_string());
            if rooms.is_err() {
                let err = rooms.err().unwrap();
                println!("获取服务器房间数据失败 {:?}", err);
                app.emit("log", format!("获取房间数据失败{:?}", err))
                    .unwrap();
                std::thread::sleep(std::time::Duration::from_secs(3));
                continue;
            }
            let rooms = rooms.unwrap();
            let data = rooms["data"].as_array();
            if data.is_none() {
                println!("没有获取到数据");
                app.emit("log", "no data for room").unwrap();
                std::thread::sleep(std::time::Duration::from_secs(3));
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
                let index = i % length;
                let cookie__ = m_cookies_stack.get(index).unwrap().to_string();
                let mut rng = rand::rng();
                let rand_number = rng.random_range(900..=2000);
                let json_data = json!({
                    "room_id": room_id.to_string(),
                    "web_rid":web_rid.to_string(),
                    "title":title.to_string(),"type":gather_type_, "cookie":cookie__, "token": token_, "sleep":rand_number,
                    "options":options
                });
                pool.execute(threads::Job::new(json_data, app.clone()));
                let payload = format!(
                    "web rid: {:?} room id: {:?} title: {:?} owner_nick {:?} user count {:?}",
                    web_rid, room_id, title, owner_nickname, user_count
                );
                app.emit("log", payload).unwrap();
                i = i + 1;
                std::thread::sleep(std::time::Duration::from_millis(100));
                room_count += 1;
            }
            {
                let state = GLOBAL_APP_STATE.get().unwrap().clone();
                let mut state_ = state.lock().unwrap();
                state_.total += room_count;
                if state_.is_spider_running == false {
                    println!("停止线程执行");
                    break;
                }
                println!("------start main emit-----");
                app.emit(
                    "log",
                    format!("本轮采集完成，线程结束,采集房间数：{}", room_count),
                )
                .unwrap();
                app.emit("spider", json!({"total": state_.total, "allowed_count":state_.allowed_count,"fd_count":state_.fd_count }))
                    .unwrap();
            }
            std::thread::sleep(std::time::Duration::from_millis(800));
        }
    });
    Ok("Success".to_string())
}

#[tauri::command]
async fn sync_stop_spider(app: AppHandle) -> Result<String, String> {
    let _ = app.emit("notify", "采集程序停止成功");
    {
        let state = GLOBAL_APP_STATE.get().unwrap().clone();
        let mut state_ = state.lock().unwrap();
        state_.is_spider_running = false;
    }
    Ok("stop success!".to_string())
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
async fn test_login_required(params: String) -> Result<bool, String> {
    let json = serde_json::from_str::<serde_json::Value>(&params).expect("parse json error");
    let group_ids = json["group_id"].as_array().unwrap();
    let mut group_id = String::from("");
    if group_ids.len() > 0 {
        group_id = group_ids[0].as_str().unwrap().to_string();
    }
    let windows = bit::browser_group_windows(group_id).await.unwrap();
    println!("windows is: {:?}", windows);
    Ok(true)
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
    //重新启动自动化任务逻辑跟之前的逻辑是一样子的
    let json = serde_json::from_str::<serde_json::Value>(&params).expect("parse json error");
    let group_ids = json["group_id"].as_array().unwrap();
    let mut group_id = String::from("");
    if group_ids.len() > 0 {
        group_id = group_ids[0].as_str().unwrap().to_string();
    }
    let windows: Vec<serde_json::Value> = bit::browser_group_windows(group_id).await.unwrap();
    return automation_run(app, windows, json).await;
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

                let room_json = gather::async_fetch_room(&token, &tp, &json_).await;
                if room_json.is_err() {
                    let s = "获取直播间信息超时,等待5秒再试";
                    println!("{:?}", s);
                    app.emit("db", json!({"action":"update", "task_msg":s, "seq":seq}))
                        .unwrap();
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }

                let room_json = room_json.unwrap();
                let data = room_json["data"].as_array().unwrap();
                if data.is_empty() {
                    app.emit("db", json!({"action":"update","dy_name":dy_name,  "dy_account":dy_account, "task_name":task_name, "task_msg":"没找到直播间，等待3秒再试", "seq":seq})).unwrap();
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }

                let title = data[0]["title"].as_str().unwrap();
                app.emit(
                    "db",
                    json!({"action":"update", "task_msg":format!("获取直播间{:?}", title), "task_name":task_name, "seq":seq}),
                )
                .unwrap();

                let send_time = data[0].get("send_time").unwrap().as_u64().unwrap();
                let delay = data[0].get("delay").unwrap().as_u64().unwrap();
                //let leave_time = bit::caclulate_leave_time(send_time, delay);  //不知道live time
                //要干嘛
                let room_url = data[0]
                    .get("room_url")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();
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
            let rand_number = rng.random_range(3000..5000);
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

                let rooms = gather::sync_fetch_dy_room(_m_cookie_stack[0].as_str().to_string());
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
                    let rand_number = rng.random_range(1000..=3000);
                    let json_data = json!({
                        "room_id": room_id.to_string(),
                        "web_rid":web_rid.to_string(),
                        "title":title.to_string(),
                        "type":_tp, "cookie":cookie__, "token": _token, "sleep":rand_number,
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

/*
#[tauri::command]
async fn check_token_expired(app:AppHandle, _token: &str) -> Result<(), String> {
    //模拟检查token是否过期
    let store = app.app_handle().store("app.json").unwrap();
    let token = store
        .get("token")
        .unwrap()
        .as_str()
        .to_owned()
        .unwrap()
        .to_string();
    println!("token is: {:?}", token);
    let result = gather::check_token_expired(&token).await;
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("token expired check error {:?}", e)),
    }
}
    */

#[tauri::command]
async fn stop_test() -> Result<(), tauri::Error> {
    let mut s: Vec<(i32, String)> = vec![
        (231, "sparks".to_string()),
        (321, "json".to_string()),
        (32, "safsd".to_string()),
        (111, "do somethings".to_string()),
    ];
    s.sort_by(|a, b| a.0.cmp(&b.0));
    let x = s.pop().unwrap();
    println!("sparks is s: {:?} {:?}", s, x);
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
            stop_test,
            flexable_windows,
            sync_start_spider,
            sync_stop_spider,
            fetch_bitbrowser_group,
            spider_with_multiple_threads_stop,
            spider_with_multiple_threads,
            get_ip,
            bit_group_browsers,
            browser_windows_open,
            start_automation,
            stop_automation,
            mac_address,
            test_login_required,
            re_start_automation,
            window_bounds,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
