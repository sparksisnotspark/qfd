use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::SystemTime;

use serde_json::json;
use tauri::{AppHandle, Emitter};

use crate::{
    gather,
    room_list_db::{count_active_room_list, exists_active_room_by_id, insert_room_list},
    GLOBAL_APP_STATE,
};

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
    let min_avg_diamond = options["min_avg_diamond"].as_u64().unwrap_or(1);
    let fd_min_avg_diamond = options["fd_min_avg_diamond"].as_u64().unwrap_or(10);
    let min_time = options["min_time"].as_u64().unwrap();
    let max_time = options["max_time"].as_u64().unwrap();
    let display_value = payload["display_value"].as_u64().unwrap();
    let min_packet_number = options["min_packet_number"].as_u64().unwrap_or(1);
    let min_diamond = options["min_diamond"].as_u64().unwrap_or(0);

    if people_limit < display_value {
        let state: Arc<Mutex<crate::AppState>> = GLOBAL_APP_STATE.get().unwrap().clone();
        let mut state_ = state.lock().unwrap();
        state_.limit_room += 1;
        state_.total += 1;
        let now_ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let allowed_urls = count_active_room_list(app, now_ts).unwrap_or(0);
        let j = json!({"total": state_.total, "allowed_count":state_.allowed_count, "fd_count":state_.fd_count, "allowed_urls": allowed_urls});
        app.emit("spider", j).unwrap();
        return;
    }

    if gather_type == "hb" {
        let x = gather::parse_luckbox_info(room_id, web_rid, cookie, min_avg_diamond, min_diamond);
        if let Ok(info) = x {
            let room_url = info["room_url"].as_str().unwrap();
            let room_id = info["room_id"].as_str().unwrap();
            let web_rid = info["web_rid"].as_str().unwrap();
            let title = info["title"].as_str().unwrap();
            let send_time = info["send_time"].as_u64().unwrap();
            let delay = info["delay"].as_u64().unwrap();
            let pack_count = info["pack_count"].as_u64().unwrap();

            if pack_count < min_packet_number {
                app.emit("log", format!("pack count {:?}  min packet number {:?} not match", pack_count, min_packet_number)).unwrap();
                let state = GLOBAL_APP_STATE.get().unwrap().clone();
                let mut state_ = state.lock().unwrap();
                state_.limit_room += 1;
                return;
            }

            let expired_at = send_time + delay;
            let now_ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            if exists_active_room_by_id(app, room_id, now_ts).unwrap_or(false) {
                app.emit("log", "exists room info").unwrap();
                return;
            }
            if expired_at < now_ts {
                app.emit("log", "room expired").unwrap();
                return;
            }

            let wait_for_ts = expired_at - now_ts;
            if wait_for_ts < min_time || wait_for_ts > max_time {
                app.emit("log", "wait for time out of range").unwrap();
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
                expired_at,
            };

            let state = GLOBAL_APP_STATE.get().unwrap().clone();
            let state_ = state.lock();
            if let Ok(mut state_) = state_ {
                if insert_room_list(app, &value_, now_ts).unwrap_or(false) {
                    state_.allowed_count += 1;
                    app.emit("log", "gather room successfully").unwrap();
                }
            }
        } else {
            app.emit("log", format!("gather room failed: {:?}", x.err().unwrap())).unwrap();
        }
    } else if gather_type == "fd" {
        let x = gather::parse_lottery_info(room_id, web_rid, cookie, only_big_packet, join_fans_group, people_limit, winning_ratio, fd_min_avg_diamond, min_diamond);
        if let Ok(info) = x {
            let lucky_count = info["lucky_count"].as_u64().unwrap();
            let send_time = info["send_time"].as_u64().unwrap();
            let delay = info["delay"].as_u64().unwrap();
            let room_url = info["room_url"].as_str().unwrap();
            let room_id = info["room_id"].as_str().unwrap();
            let title = info["title"].as_str().unwrap();
            let web_rid = info["web_rid"].as_str().unwrap();

            let now_ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let wait_for_secs = send_time + delay - now_ts;
            if wait_for_secs < min_time || wait_for_secs > max_time {
                if wait_for_secs < min_time {
                    app.emit("log", format!("wait for time {:?} is less than min time {:?}", wait_for_secs, min_time)).unwrap();
                } else {
                    app.emit("log", format!("wait for time {:?} is greater than max time {:?}", wait_for_secs, max_time)).unwrap();
                }
                let state = GLOBAL_APP_STATE.get().unwrap().clone();
                let state_ = state.lock();
                if let Ok(mut state_) = state_ {
                    state_.limit_room += 1;
                }
                return;
            }

            if lucky_count < min_packet_number {
                app.emit("log", format!("lucky count {:?} is less than min packet number {:?}", lucky_count, min_packet_number)).unwrap();
                let state = GLOBAL_APP_STATE.get().unwrap().clone();
                let state_ = state.lock();
                if let Ok(mut state_) = state_ {
                    state_.limit_room += 1;
                }
                return;
            }

            let value = json!({"room_url": room_url,"room_id":room_id ,"type": "fd", "web_rid": web_rid, "title": title, "send_time": send_time, "delay": delay, "used_times":0, "expired_at":send_time + delay});
            let _ = gather::sync_add_one_room(token, value.clone());

            let state = GLOBAL_APP_STATE.get().unwrap().clone();
            let mut state_ = state.lock().unwrap();
            if exists_active_room_by_id(app, room_id, now_ts).unwrap_or(false) {
                app.emit("log", "room already exists").unwrap();
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

            if insert_room_list(app, &value_, now_ts).unwrap_or(false) {
                state_.fd_count += 1;
                app.emit("log", "gather room successfully").unwrap();
            }
        } else {
            app.emit("log", format!("gather room failed: {:?}", x.err().unwrap())).unwrap();
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(sleep as u64));
}

impl Job {
    pub fn new(p: serde_json::Value, app: AppHandle) -> Self {
        Job { payload: p, app }
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
        let min_avg_diamond = options["min_avg_diamond"].as_u64().unwrap_or(1);
        let fd_min_avg_diamond = options["fd_min_avg_diamond"].as_u64().unwrap_or(10);
        let min_diamond = options["min_diamond"].as_u64().unwrap_or(0);

        println!("winning_ratio: {:?} people_limit {:?} join_fans_group {:?} only big packet {:?} min avg diamond {:?} min diamond {:?}", winning_ratio, people_limit, join_fans_group, only_big_packet, min_avg_diamond, min_diamond);

        if gather_type == "hb" {
            let x = gather::parse_luckbox_info(room_id, web_rid, cookie, min_avg_diamond, min_diamond);
            if let Ok(info) = x {
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
                self.app.emit("log", "gather room successfully").unwrap();
            } else {
                self.app.emit("log", format!("gather room error {:?}", x.err().unwrap())).unwrap();
            }
        } else if gather_type == "fd" {
            let x = gather::parse_lottery_info(room_id, web_rid, cookie, only_big_packet, join_fans_group, people_limit, winning_ratio, fd_min_avg_diamond, min_diamond);
            if let Ok(info) = x {
                let prize_count = info["prize_count"].as_u64().unwrap();
                let lucky_count = info["lucky_count"].as_u64().unwrap();

                let send_time = info["send_time"].as_u64().unwrap();
                let delay = info["delay"].as_u64().unwrap();
                let room_url = info["room_url"].as_str().unwrap();
                let room_id = info["room_id"].as_str().unwrap();
                let title = info["title"].as_str().unwrap();
                let web_rid = info["web_rid"].as_str().unwrap();
                let candidate_num = info["candidate_num"].as_u64().unwrap();

                println!("prize count {:?} lucky count {:?}, {:?} {:?}", prize_count, lucky_count, delay, candidate_num);
                let value = json!({"room_url": room_url,"room_id":room_id ,"type":"fd", "web_rid":web_rid, "title":title, "send_time":send_time, "delay": delay});
                let x = gather::sync_add_one_room(token, value);
                {
                    let state = GLOBAL_APP_STATE.get().unwrap().clone();
                    let mut state_ = state.lock().unwrap();
                    state_.fd_count += 1;
                }
                println!("xx is: {:?}", x);
                self.app.emit("log", "gather  room successfully").unwrap();
            } else {
                self.app.emit("log", format!("gather room failed: {:?}", x.err().unwrap())).unwrap();
            }
        }
        {
            let state = GLOBAL_APP_STATE.get().unwrap().clone();
            let state_ = state.lock().unwrap();
            let now_ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let allowed_urls = count_active_room_list(&self.app, now_ts).unwrap_or(0);
            let j = json!({"total": state_.total, "allowed_count":state_.allowed_count, "fd_count":state_.fd_count, "allowed_urls": allowed_urls});
            self.app.emit("spider", j).unwrap();
        }

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
                    println!("worker run error {:?}", e);
                    break;
                }
            }
        });

        Worker { _id: id, thread: Some(thread) }
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
        ThreadPool { workers, sender: Some(sender) }
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
