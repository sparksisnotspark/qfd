use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::SystemTime;

use serde_json::json;
use tauri::{AppHandle, Emitter};

use crate::{gather, GLOBAL_APP_STATE};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

pub struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
pub struct Job {
    payload: serde_json::Value,
    app: AppHandle,
}

pub fn execute_gather(payload: serde_json::Value, app: &AppHandle) {
    let cookie = payload["cookie"].as_str().unwrap();
    let room_id = payload["room_id"].as_str().unwrap();
    let web_rid = payload["web_rid"].as_str().unwrap();
    let gather_type = payload["type"].as_str().unwrap();
    let token = payload["token"].as_str().unwrap();
    let sleep = payload["sleep"].as_i64().unwrap();
    let options = payload["options"].as_object().unwrap();
    let winning_ratio = options["winning_ratio"].as_u64().unwrap();
    let people_limit = options["people_limit"].as_u64().unwrap();
    let join_fans_group = options["join_fans_group"].as_bool().unwrap();
    let only_big_packet = options["only_big_packet"].as_bool().unwrap();
    let min_avg_diamond = options["min_avg_diamond"].as_u64().or(Some(1)).unwrap(); //红包
    let fd_min_avg_diamond = options["fd_min_avg_diamond"].as_u64().or(Some(10)).unwrap();
    let min_time = options["min_time"].as_u64().unwrap(); //时间开始，表示最小等多久
    let max_time = options["max_time"].as_u64().unwrap(); //时间区间，表示最多等多久
    let use_time_strict = options["use_time_strict"].as_bool().unwrap_or(false);
    let display_value = payload["display_value"].as_u64().unwrap();
    let min_packet_number = options["min_packet_number"].as_u64().unwrap_or(1);
    let min_diamond = options["min_diamond"].as_u64().unwrap_or(0);

    if people_limit < display_value {
        {
            let state: Arc<Mutex<crate::AppState>> = GLOBAL_APP_STATE.get().unwrap().clone();
            let mut state_ = state.lock().unwrap();
            state_.limit_room += 1;
            state_.total += 1;
            app.emit("spider", json!({"total": state_.total, "allowed_count":state_.allowed_count, "fd_count":state_.fd_count, "allowed_urls": state_.room_list.len() + state_.room_list_used.len()})).unwrap();
            return;
        }
    }

    //println!( "max time: {:?} use time strict {:?}", max_time, use_time_strict);

    if gather_type == "hb" {
        let x = gather::parse_luckbox_info(room_id, web_rid, cookie, min_avg_diamond, min_diamond);
        if x.is_ok() {
            let info = x.unwrap();
            let room_url = info["room_url"].as_str().unwrap();
            let room_id = info["room_id"].as_str().unwrap();
            let web_rid = info["web_rid"].as_str().unwrap();
            let title = info["title"].as_str().unwrap();
            let send_time = info["send_time"].as_u64().unwrap();
            let delay = info["delay"].as_u64().unwrap();
            let pack_count = info["pack_count"].as_u64().unwrap();

            if pack_count < min_packet_number {
                //最小發包數量小於設定的最小值，則跳過
                app.emit(
                    "log",
                    format!(
                        "红包的发包数量 {:?} 小于设定的最小包数量 {:?}，放弃这个红包",
                        pack_count, min_packet_number
                    ),
                )
                .unwrap();
                let state = GLOBAL_APP_STATE.get().unwrap().clone();
                let mut state_ = state.lock().unwrap();
                state_.limit_room += 1;
                return;
            }

            //let value = json!({"room_url": room_url,"room_id":room_id ,"type":"hb", "web_rid":web_rid, "title":title, "send_time":send_time, "delay": delay});
            let expired_at = send_time + delay;
            //let value = json!({"room_url": room_url,"room_id":room_id ,"type":"hb", "web_rid":web_rid, "title":title, "send_time":send_time, "delay": delay,"used_times": 0, "expired_at": expired_at});
            //let _ = gather::sync_add_one_room(token, value.clone());
            {
                {
                    let state = GLOBAL_APP_STATE.get().unwrap().clone();
                    let state_ = state.lock();
                    if state_.is_ok() {
                        let state_ = state_.unwrap();
                        let _p = state_.room_list.iter().position(|x| x.room_id == room_id);
                        if _p.is_some() {
                            app.emit("log", "已存的红包信息").unwrap();
                            return;
                        }
                    }
                }
                let now = SystemTime::now();
                let now_ts = now
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if expired_at < now_ts {
                    app.emit("log", "红包已经过期了").unwrap();
                    return;
                }

                let wait_for_ts= expired_at - now_ts;
                if wait_for_ts < min_time || wait_for_ts > max_time {
                    app.emit("log", "时间不在区间内").unwrap();
                    return;
                }

                let value_ = crate::RoomUrlList {
                    room_url: room_url.to_string(),
                    room_id: room_id.to_string(),
                    web_rid: web_rid.to_string(),
                    title: title.to_string(),
                    tp: "hb".to_string(),
                    send_time,
                    used_times: 0,
                    delay,
                    expired_at: expired_at,
                };
                {
                    let state = GLOBAL_APP_STATE.get().unwrap().clone();
                    let state_ = state.lock();

                    if state_.is_ok() {
                        let mut state_ = state_.unwrap();
                        state_.allowed_count += 1;
                        state_.room_list.push(value_);
                        state_.room_list.sort_by(|a, b| {
                            let ac_ = a.expired_at;
                            let bc_ = b.expired_at;
                            ac_.cmp(&bc_)
                        });
                        state_.room_list.retain(|x| x.expired_at > now_ts);
                        app.emit("log", "插入红包的信息成功").unwrap();
                    }
                }
            }
        } else {
            app.emit("log", format!("解析红包信息出错  {:?}", x.err().unwrap()))
                .unwrap();
        }
    } else if gather_type == "fd" {
        let x = gather::parse_lottery_info(
            room_id,
            web_rid,
            cookie,
            only_big_packet,
            join_fans_group,
            people_limit,
            winning_ratio,
            fd_min_avg_diamond,
            min_diamond,
        ); //逻辑放到了添加福袋的里面去了
        if x.is_ok() {
            let info = x.unwrap();
            let prize_count = info["prize_count"].as_u64().unwrap();
            let lucky_count = info["lucky_count"].as_u64().unwrap();

            let send_time = info["send_time"].as_u64().unwrap();
            let delay = info["delay"].as_u64().unwrap();
            let room_url = info["room_url"].as_str().unwrap();
            let room_id = info["room_id"].as_str().unwrap();
            let title = info["title"].as_str().unwrap();
            let web_rid = info["web_rid"].as_str().unwrap();

            let now = SystemTime::now();
            let now_ts = now
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let wait_for_secs = send_time + delay - now_ts;
            if wait_for_secs < min_time || wait_for_secs > max_time {
                if wait_for_secs < min_time {
                    app.emit(
                        "log",
                        format!(
                            "福袋即将开始 {:?} 秒, 小于设定的最小时间 {:?} 秒，放弃这个福袋",
                            wait_for_secs, min_time
                        ),
                    )
                    .unwrap();
                } else if wait_for_secs > max_time {
                    app.emit(
                        "log",
                        format!(
                            "福袋要等 {:?} 秒, 超过设定的最大时间 {:?} 秒，放弃这个福袋",
                            wait_for_secs, max_time
                        ),
                    )
                    .unwrap();
                }
                {
                    let state = GLOBAL_APP_STATE.get().unwrap().clone();
                    let state_ = state.lock();
                    if state_.is_ok() {
                        let mut state_ = state_.unwrap();
                        state_.limit_room += 1;
                    }
                }
                return;
            }

            if lucky_count < min_packet_number {
                //最小發包數量小於設定的最小值，則跳過
                app.emit(
                    "log",
                    format!(
                        "福袋的已中奖人数 {:?} 小于设定的最小包数量 {:?}，放弃这个福袋",
                        lucky_count, min_packet_number
                    ),
                )
                .unwrap();
                {
                    let state = GLOBAL_APP_STATE.get().unwrap().clone();
                    let state_ = state.lock();
                    if state_.is_ok() {
                        let mut state_ = state_.unwrap();
                        state_.limit_room += 1;
                    }
                }
                return;
            }

            //println!( "prize count {:?} lucky count {:?}, {:?} {:?}", prize_count, lucky_count, delay, candidate_num);

            //let value = json!({"room_url": room_url,"room_id":room_id ,"type": "fd", "web_rid": web_rid, "title": title, "send_time": send_time, "delay": delay});
            let value = json!({"room_url": room_url,"room_id":room_id ,"type": "fd", "web_rid": web_rid, "title": title, "send_time": send_time, "delay": delay, "used_times":0, "expired_at":send_time + delay});
            let _ = gather::sync_add_one_room(token, value.clone()); //提交服务器
            {
                let state = GLOBAL_APP_STATE.get().unwrap().clone();
                let mut state_ = state.lock().unwrap();

                let _p = state_.room_list.iter().position(|p| p.room_id == room_id);

                if _p.is_some() {
                    app.emit("log", "已存在的直播间!").unwrap();
                    return;
                }

                let value_ = crate::RoomUrlList {
                    room_url: room_url.to_string(),
                    room_id: room_id.to_string(),
                    web_rid: web_rid.to_string(),
                    title: title.to_string(),
                    tp: "fd".to_string(),
                    send_time,
                    used_times: 0,
                    delay,
                    expired_at: send_time + delay,
                };

                state_.fd_count += 1;
                state_.room_list.push(value_);
                state_.room_list.sort_by(|a, b| {
                    let ac_ = a.expired_at;
                    let bc_ = b.expired_at;
                    ac_.cmp(&bc_)
                });

                let now = SystemTime::now();
                let now_ts = now
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                state_.room_list.retain(|x| x.expired_at > now_ts);
                app.emit("log", "插入房间成功!").unwrap();
            }
        } else {
            app.emit("log", format!("福袋解析出错 {:?}", x.err().unwrap()))
                .unwrap();
        }
    }

    app.emit("log", format!("线程休眠 {:?} 毫秒", sleep))
        .unwrap();
    //println!("sleep for this thread {:?}", sleep);
    std::thread::sleep(std::time::Duration::from_millis(sleep as u64));
}

impl Job {
    pub fn new(p: serde_json::Value, app: AppHandle) -> Self {
        Job {
            payload: p,
            app: app,
        }
    }
    fn execute(&self) {
        let cookie = self.payload["cookie"].as_str().unwrap();
        let room_id = self.payload["room_id"].as_str().unwrap();
        let web_rid = self.payload["web_rid"].as_str().unwrap();
        let gather_type = self.payload["type"].as_str().unwrap();
        let token = self.payload["token"].as_str().unwrap();
        let sleep = self.payload["sleep"].as_i64().unwrap();
        let options = self.payload["options"].as_object().unwrap();
        let winning_ratio = options["winning_ratio"].as_u64().unwrap();
        let people_limit = options["people_limit"].as_u64().unwrap();
        let join_fans_group = options["join_fans_group"].as_bool().unwrap();
        let only_big_packet = options["only_big_packet"].as_bool().unwrap();
        let min_avg_diamond = options["min_avg_diamond"].as_u64().or(Some(1)).unwrap(); //红包
        let fd_min_avg_diamond = options["fd_min_avg_diamond"].as_u64().or(Some(10)).unwrap();
        let min_diamond = options["min_diamond"].as_u64().unwrap_or(0);

        println!("winning_ratio: {:?} people_limit {:?} join_fans_group {:?} only big packet {:?} min avg diamond {:?} min diamond {:?}", winning_ratio, people_limit, join_fans_group, only_big_packet, min_avg_diamond, min_diamond);

        if gather_type == "hb" {
            let x = gather::parse_luckbox_info(room_id, web_rid, cookie, min_avg_diamond, min_diamond);
            if x.is_ok() {
                let info = x.unwrap();
                let room_url = info["room_url"].as_str().unwrap();
                let room_id = info["room_id"].as_str().unwrap();
                let web_rid = info["web_rid"].as_str().unwrap();
                let title = info["title"].as_str().unwrap();
                let send_time = info["send_time"].as_u64().unwrap();
                let delay = info["delay"].as_u64().unwrap();
                let value = json!({"room_url": room_url,"room_id":room_id ,"type":"hb", "web_rid":web_rid, "title":title, "send_time":send_time, "delay": delay});
                let x = gather::sync_add_one_room(token, value);
                {
                    let state = GLOBAL_APP_STATE.get().unwrap().clone();
                    let mut state_ = state.lock().unwrap();
                    state_.allowed_count += 1;
                }
                println!("xx is: {:?}", x);
                self.app.emit("log", "插入红包的信息成功").unwrap();
            } else {
                self.app
                    .emit("log", format!("出错  {:?}", x.err().unwrap()))
                    .unwrap();
            }
        } else if gather_type == "fd" {
            let x = gather::parse_lottery_info(
                room_id,
                web_rid,
                cookie,
                only_big_packet,
                join_fans_group,
                people_limit,
                winning_ratio,
                fd_min_avg_diamond,
                min_diamond
            ); //逻辑放到了添加福袋的里面去了
            if x.is_ok() {
                let info = x.unwrap();
                let prize_count = info["prize_count"].as_u64().unwrap();
                let lucky_count = info["lucky_count"].as_u64().unwrap();

                let send_time = info["send_time"].as_u64().unwrap();
                let delay = info["delay"].as_u64().unwrap();
                let room_url = info["room_url"].as_str().unwrap();
                let room_id = info["room_id"].as_str().unwrap();
                let title = info["title"].as_str().unwrap();
                let web_rid = info["web_rid"].as_str().unwrap();
                let candidate_num = info["candidate_num"].as_u64().unwrap();

                println!(
                    "prize count {:?} lucky count {:?}, {:?} {:?}",
                    prize_count, lucky_count, delay, candidate_num
                );
                let value = json!({"room_url": room_url,"room_id":room_id ,"type":"fd", "web_rid":web_rid, "title":title, "send_time":send_time, "delay": delay});
                let x = gather::sync_add_one_room(token, value);
                {
                    let state = GLOBAL_APP_STATE.get().unwrap().clone();
                    let mut state_ = state.lock().unwrap();
                    state_.fd_count += 1;
                }
                println!("xx is: {:?}", x);
                self.app.emit("log", "查询到福袋的信息").unwrap();
            } else {
                self.app
                    .emit("log", format!("福袋解析出错 {:?}", x.err().unwrap()))
                    .unwrap();
            }
        }
        {
            let state = GLOBAL_APP_STATE.get().unwrap().clone();
            let state_ = state.lock().unwrap();
            self.app
                .emit("spider", json!({"total": state_.total, "allowed_count":state_.allowed_count, "fd_count":state_.fd_count, "allowed_urls": state_.room_list.len() + state_.room_list_used.len()}))
                .unwrap();
        }

        self.app
            .emit("log", format!("线程休眠 {:?} 毫秒", sleep))
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(sleep as u64));
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            println!("wait for recive {} got a job; executing.", id);
            match message {
                Ok(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job.execute();
                }
                Err(e) => {
                    println!("线程获取数据出现错误 {:?}", e);
                    break;
                }
            }
        });

        Worker {
            _id: id,
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute(&self, f: Job) {
        println!("execute job!");
        self.sender.as_ref().unwrap().send(f).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
