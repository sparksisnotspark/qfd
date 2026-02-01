use std::sync::{mpsc, Arc, Mutex};
use std::{thread};

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

impl Job {
    pub fn new(p: serde_json::Value, app: AppHandle) -> Self {
        Job { payload: p, app: app }
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
        let min_avg_diamond = options["min_avg_diamond"].as_u64().unwrap();
        println!("winning_ratio: {:?} people_limit {:?} join_fans_group {:?} only big packet {:?} min avg diamond {:?}", winning_ratio, people_limit, join_fans_group, only_big_packet, min_avg_diamond);

        if gather_type == "hb" {
            let x = gather::parse_luckbox_info(room_id, web_rid, cookie);
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
                self.app.emit("log", format!("insert hb failure {:?}", x.err().unwrap())).unwrap();
            }
        } else if gather_type == "fd" {
            let x = gather::parse_lottery_info(room_id, web_rid, cookie);
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

                if candidate_num > people_limit {
                    self.app.emit("log", "winning ratio ").unwrap();
                    std::thread::sleep(std::time::Duration::from_millis(600)); 
                    return
                }

                if (lucky_count / candidate_num)  < (winning_ratio  / 100){
                    self.app.emit("log", "winning ratio ").unwrap();
                    std::thread::sleep(std::time::Duration::from_millis(600));
                    return
                }

                println!("prize count {:?} lucky count {:?}, {:?} {:?}", prize_count, lucky_count, delay, candidate_num);
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
                self.app.emit("log", format!("fetch fd failure {:?}", x.err().unwrap())).unwrap();
            }
        }
        {
            let state = GLOBAL_APP_STATE.get().unwrap().clone();
            let state_ = state.lock().unwrap();
            self.app
                .emit("spider", json!({"total": state_.total, "allowed_count":state_.allowed_count, "fd_count":state_.fd_count}))
                .unwrap();
        }

        self.app.emit("log", format!("线程休眠 {:?} 毫秒",sleep )).unwrap();
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

        Worker { _id: id, thread: Some(thread) }
    }
}

//type Job = Box<dyn FnOnce() + Send + 'static>;

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
