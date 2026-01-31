use lazy_regex::regex_is_match;
use rand::Rng;
use reqwest::{header, Client, Error};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, error, os::windows::process::CommandExt, process::Stdio, thread};
use tauri::{AppHandle, Emitter};
use thirtyfour::prelude::*;
use thiserror::Error;
use tokio::time::sleep;
use tokio::time::Duration;
pub const API_URL: &str = "http://127.0.0.1:54345";
pub const QFD_SERVER_URL: &str = "http://193.134.211.160:8787";
//static mut CHILD_PROCESS: Vec<std::process::Child> = vec![];

pub fn monitor() -> Vec<String> {
    Vec::with_capacity(3)
}

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Error From tauri")]
    StringError(String),

    #[error("tauri error for this")]
    TauriError,

    #[error("error from webdriver")]
    WebDriverError(#[from] WebDriverError),

    #[error("error from reqwest")]
    RequestError(#[from] reqwest::Error),

    #[error("error from serde json")]
    SerdeError(#[from] serde_json::Error),
}

pub fn bitbrowser_api_check(path: &str) -> Result<(), Box<dyn error::Error>> {
    let res: () = reqwest::blocking::get(format!("{}{}", API_URL, path))?.json()?;
    println!("res is {:?}", res);
    Ok(())
}

pub fn bitbrowser_api_post(
    path: &str,
    params: HashMap<&str, &str>,
) -> Result<(), Box<dyn error::Error>> {
    let client = reqwest::blocking::Client::new();
    let res: () = client
        .post(format!("{}{}", API_URL, path))
        .json(&params)
        .send()?
        .json()?;
    println!("res is {:?}", res);
    Ok(())
}

pub fn bitbrowser_pids_all(params: &serde_json::Value) -> Result<serde_json::Value, MyError> {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(format!("{}/pids/all", API_URL))
        .json(&params)
        .send()?
        .json::<serde_json::Value>()?;
    println!("res is {:?}", res);
    Ok(res)
}

pub async fn bitbroswer_pids_alive(ids: Vec<String>) -> Result<serde_json::Value, MyError> {
    println!("alive of ids: {:?} ", ids);
    let client = Client::new();
    let res = client
        .post(format!("{}/browser/pids/alive", API_URL))
        .json(&json!({"ids":ids}))
        .send()
        .await;
    let json = res.unwrap().json::<serde_json::Value>().await?;
    Ok(json)
}

pub fn bitbrowser_api_open(browser_id: &str) -> Result<(), Box<dyn error::Error>> {
    let mut params = HashMap::new();
    params.insert("browser_id", browser_id);
    let res = bitbrowser_api_post("/open", params)?;
    println!("Opened URL in browser {:?}", res);
    Ok(())
}

pub async fn browser_group_list() -> Result<Vec<(String, String)>, Box<dyn error::Error>> {
    let client = Client::new();
    let mut params = HashMap::new();
    params.insert("page", 0);
    params.insert("pageSize", 20);
    let res = client
        .post(format!("{}{}", API_URL, "/group/list"))
        .header("Cookie", "")
        .header("referer", "")
        .header("User-Agent", "Mozila")
        .json(&params)
        .send()
        .await?;

    let result = res.json::<serde_json::Value>().await?;
    let mut groups = Vec::new();
    _ = result["data"]["list"]
        .as_array()
        .unwrap()
        .iter()
        .for_each(|item| {
            let group_id = item["id"].as_str().unwrap();
            let group_name = item["groupName"].as_str().unwrap();
            groups.push((group_id.to_string(), group_name.to_string()));
        });
    Ok(groups)
}

//获取根据分组获取窗口列表
pub async fn browser_group_windows(
    group_id: String,
) -> Result<Vec<serde_json::Value>, Box<dyn error::Error>> {
    let client = Client::new();
    let json = serde_json::json!({
        "page": 0,
        "pageSize": 100,
        "groupId": group_id
    });
    let res = client
        .post(format!("{}{}", API_URL, "/browser/list"))
        .json(&json)
        .send()
        .await?;
    let json_data = res.json::<serde_json::Value>().await?;
    let list = json_data["data"]["list"].as_array().unwrap();
    Ok(list.to_vec())
}

pub async fn browser_clear(browser_ids: &str) -> Result<(), Error> {
    let client = Client::new();
    let mut params = HashMap::new();
    params.insert("ids", browser_ids);
    let res = client
        .post(format!("{}{}", API_URL, "/cache/clear"))
        .json(&params)
        .send()
        .await;
    println!("Cleared {:?}", res);
    Ok(())
}

//重新排序窗口
pub async fn flexable() -> Result<(), MyError> {
    let client = Client::new();
    let res = client
        .post(format!("{}{}", API_URL, "/windowbounds/flexable"))
        .send()
        .await?;
    println!("Flexable {:?}", res);
    Ok(())
}

pub async fn browser_detail(browser_id: String) -> Result<serde_json::Value, MyError> {
    let client = Client::new();
    let res = client
        .post(format!("{}{}", API_URL, "/browser/detail"))
        .json(&json!({"id":browser_id}))
        .send()
        .await?;
    let json = res.json::<serde_json::Value>().await?;
    Ok(json)
}

pub async fn window_bounds(options: serde_json::Value) -> Result<(), MyError> {
    let client = Client::new();
    let x_offset = options["x_offset"].as_i64().unwrap();
    let y_offset = options["y_offset"].as_i64().unwrap();
    let json = json!({  "type": "diagonal", "startX": 0, "startY": 0, "width": 1024, "height": 800, "col": 3, "spaceX": 50, "spaceY": 50, "offsetX": x_offset, "offsetY": y_offset,});
    let r = client
        .post(format!("{}{}", API_URL, "/windowbounds"))
        .json(&json)
        .send()
        .await;
    println!("r is: {:?}", r);
    Ok(())
}

//打开浏览器并返回浏览器的相关数据
pub async fn browser_open(browser_id: &str) -> Result<serde_json::Value, Box<dyn error::Error>> {
    let p = json!({"id":browser_id, "ignoreDefaultUrls":true,"args":["--disable-blink-features=AutomationControlled"], "queue": true});
    let client = Client::new();
    let res = client
        .post(format!("{}{}", API_URL, "/browser/open"))
        .timeout(Duration::from_secs(120))
        .json(&p)
        .send()
        .await?;
    let json_value = res.json::<serde_json::Value>().await?;
    Ok(json_value)
}

//停止自动化的时候关闭浏览器
pub async fn browser_closeall() -> Result<(), Error> {
    let client = Client::new();
    _ = client
        .post(format!("{}{}", API_URL, "/browser/close/all"))
        .send()
        .await;
    /*
    unsafe {
        for child in &mut CHILD_PROCESS {
            let _ = child.kill();
        }
        CHILD_PROCESS.clear();
    }
    */
    Ok(())
}

pub async fn browser_close(browser_id: &str) -> Result<(), Error> {
    println!("browser id is: {:?}", browser_id);
    let client = Client::new();
    _ = client
        .post(format!("{}{}", API_URL, "/browser/close"))
        .json(&json!({"id":browser_id}))
        .send()
        .await?;
    Ok(())
}

pub async fn start_selenium_chrome(drive_path: &str, debug_addr: &str) -> Option<WebDriver> {
    println!(
        "driver path is: {:?} driver addr {:?}",
        drive_path, debug_addr
    );
    let mut caps = DesiredCapabilities::chrome();
    caps.set_debugger_address(&debug_addr).unwrap();
    let server_url = chromedrive_start(drive_path.to_string()).unwrap();
    sleep(Duration::from_millis(500)).await;
    let driver = WebDriver::new(server_url, caps).await.unwrap();
    Some(driver)
}

pub fn caclulate_leave_time(start: u64, delay: u64) -> u64 {
    let now = SystemTime::now();
    let now_sec = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let leave_time = start + delay;
    let default_leave_time: u64 = 0;
    if now_sec > leave_time {
        return default_leave_time;
    }
    let mut rng = rand::rng();
    let rand_number = rng.random_range(3..8);
    leave_time - now_sec + rand_number
}

pub async fn check_login_required(driver: &WebDriver) -> Result<bool, MyError> {
    driver.enter_default_frame().await?;
    let exists = driver
        .query(By::Css("button.eK1Z2Sel"))
        .wait(Duration::from_secs(3), Duration::from_millis(30))
        .exists()
        .await?;
    if exists {
        println!("需要登录");
        return Ok(true);
    }
    println!("不需要登录");
    Ok(false)
}

pub async fn check_solid_show(driver: &WebDriver) -> Result<bool, MyError> {
    //检测是否出现滑块认证
    driver.enter_default_frame().await?;
    let exists = driver
        .query(By::Css("div#solid_show"))
        .wait(Duration::from_secs(3), Duration::from_millis(30))
        .exists()
        .await?;
    if exists {
        println!("需要滑块认证");
        return Ok(true);
    }
    println!("不需要滑块认证");
    Ok(false)
}

//搶紅包邏輯
pub async fn hb_logic(
    app: &AppHandle,
    driver: &WebDriver,
    room_url: &str,
    seq: &str,
    options: &serde_json::Value,
) -> Result<(), MyError> {
    driver.goto(room_url).await?;
    println!("options is {:?}", options);
    sleep(Duration::from_secs(1)).await;

    let dy_name = options["dy_name"].as_str().unwrap();
    let dy_account = options["dy_account"].as_str().unwrap();

    let _ = driver.enter_default_frame().await;
    let el = driver
        .query(By::Css("div.redpacket"))
        .wait(Duration::from_secs(3), Duration::from_millis(20))
        .first()
        .await?;

    el.wait_until().clickable().await?;
    driver
        .action_chain()
        .move_to_element_center(&el)
        .click()
        .perform()
        .await?;
    let iframe = driver
        .query(By::Css("div#short_touch_land_redpacket_land > iframe"))
        .first()
        .await?;

    println!("find redpacket iframe");
    iframe.enter_frame().await?;
    let shadow_root = driver
        .query(By::Tag("lynx-view"))
        .wait(Duration::from_secs(3), Duration::from_millis(50))
        .first()
        .await?
        .get_shadow_root()
        .await?;

    let wrap = shadow_root
        .query(By::Css("scroll-view > lynx-wrapper"))
        .wait(Duration::from_secs(3), Duration::from_millis(50))
        .first()
        .await?;

    let hbs = wrap
        .query(By::ClassName("scale_container"))
        .wait(Duration::from_secs(3), Duration::from_millis(20))
        .all_from_selector()
        .await?;
    println!("find 红包按钮? {:?}", hbs);
    let mut hb_canyu_count = 0; //红包参与次数
    let mut hb_win_count = 0; //红包获胜次数
    let mut hb_huode_diamond_count = 0; //红包获得钻石数
    let hbs_clone = hbs.clone();
    let mut all_gift = true;
    let send_time = options["send_time"].as_u64().unwrap();
    let delay = options["delay"].as_u64().unwrap();
    let mut leave_time = caclulate_leave_time(send_time, delay);
    for el in hbs {
        println!("每个点击按钮 {:?}", el);
        let is_gift = el
            .query(By::ClassName("list_container_exp1-left"))
            .wait(Duration::from_secs(2), Duration::from_millis(10))
            .exists()
            .await?;
        println!("is gift {:?}", is_gift);
        if is_gift {
            driver
                .action_chain()
                .move_to_element_center(&el)
                .click()
                .perform()
                .await
                .unwrap();
            sleep(Duration::from_secs(1)).await;
        } else {
            all_gift = false;
            let btn = el
                .query(By::Css(".snatch_button_container.snatch_button_container_stage1.snatch_button_container_stage2"))
                .wait(Duration::from_secs(2), Duration::from_millis(100))
                .first()
                .await;
            if btn.is_err() {
                println!("抢按钮未找到 {:?}", btn.err().unwrap());
                break;
            } else {
                println!("找到抢按钮 ");
            }

            let btn_unwrap = btn.unwrap();

            for _ in 0..2 {
                let c = driver
                    .action_chain()
                    .move_to_element_center(&btn_unwrap)
                    .click()
                    .perform()
                    .await;
                if c.is_ok() {
                    println!("点击抢按钮 ");
                } else {
                    println!("点击抢按钮失败 {:?}", c.err().unwrap());
                }
                sleep(Duration::from_secs(1)).await;
            }

            let timer = el
                .query(By::Css(".count_down_canvas_container_exp"))
                .wait(Duration::from_secs(2), Duration::from_millis(10))
                .first()
                .await;
            if timer.is_err() {
                println!("timer 未找到 {:?}", timer.err().unwrap());
            } else {
                println!("找到 timer ");
                let txt = timer.unwrap().attr("accessibility-label").await;
                if txt.is_err() {
                    println!("timer text error {:?}", txt.err().unwrap());
                } else {
                    let txt = txt.unwrap();
                    println!("timer text is: {:?}", txt);
                    if let Some(c) = txt {
                        let sec = paser_timer(&c);
                        println!("等待 {:?} 秒红包结束", sec);
                        if sec > 0 {
                            let mut rng = rand::rng();
                            let rand_number = rng.random_range(3..10);
                            leave_time = sec as u64 + rand_number;
                            println!("设置等待时间为 {:?}", leave_time);
                        }
                    }
                }
            }

            sleep(Duration::from_secs(1)).await;
            hb_canyu_count += 1;
        }
    }

    if all_gift {
        let _ = app.emit("db", json!({"action":"update","task_msg":"全部礼物红包，下一个", "hongbao_canyu_count":1, "dy_name":dy_name,  "dy_account":dy_account, "seq":seq  }));
        return Ok(());
    }

    let _ = app.emit("db", json!({"action":"update","hongbao_canyu_count": 1, "task_msg":"执行完成抢红包，等待开奖结果", "dy_name":dy_name, "dy_account":dy_account, "seq":seq  }));
    println!("等待 {:?} 秒开奖结果", leave_time);
    sleep(Duration::from_secs(leave_time)).await;

    let i = 0;
    for el in hbs_clone {
        //根据结果解析用户获得的钻石数量
        let result = el
            .query(By::Css(".main_body_container"))
            .wait(Duration::from_secs(1), Duration::from_millis(50))
            .all_from_selector()
            .await;
        if !result.is_err() {
            let res = result.unwrap();
            if res.is_empty() {
                println!("empty of res")
            } else {
                println!("找到结果，解析结果 0 index {:?} {:?}", i, res.len());
                let result = res[1]
                    .query(By::Css(".result_outer"))
                    .wait(Duration::from_secs(3), Duration::from_millis(50))
                    .first()
                    .await;
                if !result.is_ok() {
                    println!("result outer {:?}", i);
                    continue;
                }
                let result_text = result.unwrap().query(By::Css(".result_text")).first().await;
                if !result_text.is_ok() {
                    println!("raw text 未找到！ {:?}", i);
                    continue;
                }
                let rawtext = result_text
                    .unwrap()
                    .query(By::Tag("raw-text"))
                    .wait(Duration::from_secs(3), Duration::from_millis(50))
                    .first()
                    .await;
                if !rawtext.is_ok() {}
                let text = rawtext.unwrap().attr("text").await.unwrap();

                if !text.is_none() {
                    let text = text.unwrap();
                    println!("找到结果，解析结果 1 {:?} {:?}", i, text);
                    if regex_is_match!(r"\d+钻", &text) {
                        hb_win_count += 1;
                        let re = lazy_regex::regex!(r"(\d+)");
                        if let Some(c) = re.captures(&text.as_str()) {
                            let huode = c.get(1).unwrap().as_str().parse::<i32>().unwrap();
                            println!("huode diamonds is: {:?}", huode);
                            hb_huode_diamond_count += huode;
                        }
                    } else {
                        println!("not match \\d+ 没有赢到钻石!");
                    }
                } else {
                    println!("没有找到获得结果");
                }
            }
        }
    }

    sleep(Duration::from_secs(1)).await;
    //----------------------解析开将结果--------------------------
    let user_diamond = parse_user_diamond(driver).await;
    let mut diamond = 0;
    if user_diamond.is_ok() {
        diamond = user_diamond.unwrap();
    }

    app.emit(
        "db",
        json!({"action":"update","task_msg":"开奖完成,准备执行下一个","hongbao_canyu_count": hb_canyu_count, "hongbao_win_count": hb_win_count,"hongbao_huode_diamond_count": hb_huode_diamond_count, "count_num":diamond,  "seq":seq  }),
    ).unwrap();
    //----------------------------------------------------------
    Ok(())
}

pub fn paser_timer(x: &str) -> i64 {
    let re = lazy_regex::regex!(r"(\d+)分(\d+)秒");
    if let Some(c) = re.captures(x) {
        let min = c.get(1).unwrap().as_str().parse::<i64>().unwrap();
        let sec = c.get(2).unwrap().as_str().parse::<i64>().unwrap();
        return min * 60 + sec;
    }

    let re = lazy_regex::regex!(r"(\d+)秒");
    if let Some(c) = re.captures(x) {
        let sec = c.get(1).unwrap().as_str().parse::<i64>().unwrap();
        return sec;
    }
    0
}

pub async fn fd_logic(
    app: &AppHandle,
    driver: &WebDriver,
    room_url: &str,
    seq: &str,
    options: &serde_json::Value,
) -> Result<(), MyError> {
    println!("Executing QFD action...");
    driver.goto(room_url).await?;
    sleep(Duration::from_secs(3)).await;
    let join_fans_group = options["join_fans_group"].as_bool().unwrap_or(false);
    let dy_name = options["dy_name"].as_str().unwrap_or("");
    let dy_account = options["dy_account"].as_str().unwrap_or("");
    let ele = driver
        .query(By::Css("div.dVxrjT_h"))
        .wait(Duration::from_secs(10), Duration::from_millis(50))
        .first()
        .await?;
    ele.wait_until().clickable().await?;

    driver
        .action_chain()
        .move_to_element_center(&ele)
        .click()
        .perform()
        .await?;

    let ele = driver
        .query(By::Css("div.QOARtY3v"))
        .wait(Duration::from_secs(5), Duration::from_millis(50))
        .first()
        .await;
    if ele.is_err() {
        let _ = app.emit(
            "db",
            json!({"seq":seq, "action":"update", "dy_name":dy_name, "dy_account":dy_account, "task_msg":"未找到福袋"}),
        );
        return Err(MyError::StringError("未找到福袋".to_string()));
    }
    app.emit("db", json!({"seq":seq, "action":"update", "canyu_count":1, "dy_name":dy_name, "dy_account": dy_account}))
        .unwrap();
    let btn = ele.unwrap();
    let _ = btn.wait_until().clickable().await?;
    let txt = btn.text().await?;
    let list = vec!["分享直播间".to_string(), "提升亲密度".to_string()];
    if list.contains(&txt) && join_fans_group == false {
        // 需要加粉丝团 或提升亲密度
        println!("加粉丝团");
        return Ok(());
    }

    if regex_is_match!(r"\d+钻", &txt) && join_fans_group == false {
        println!("需要花费钻石");
        return Ok(());
    }

    driver
        .action_chain()
        .move_to_element_center(&btn)
        .click()
        .perform()
        .await?;

    let exists = driver.query(By::Css("div.QOARtY3v")).exists().await?;
    if exists {
        let btn = driver
            .query(By::Css("div.QOARtY3v"))
            .wait(Duration::from_secs(5), Duration::from_millis(100))
            .first()
            .await?;
        btn.wait_until().clickable().await?;
        driver
            .action_chain()
            .move_to_element_center(&btn)
            .click()
            .perform()
            .await?;
    }
    let send_time = options["send_time"].as_u64().unwrap();
    let delay = options["delay"].as_u64().unwrap();

    let leave_time = caclulate_leave_time(send_time, delay);

    let _ = app.emit(
        "db",
        json!({"action":"update","task_name":"福袋任务", "task_msg":format!("等待{:?}秒离开直播间", leave_time), "dy_name":dy_name, "dy_account":dy_account, "seq":seq}),
    );
    println!("wait for leave room {:?}", leave_time);
    sleep(Duration::from_secs(leave_time)).await;
    //---------------------等待结束，查询用户的数据--------------------

    let _ = app.emit(
        "db",
        json!({"action":"update", "task_name":"福袋任务","dy_name":dy_name, "dy_account":dy_account, "task_msg":"解析抢福袋结果", "seq":seq}),
    );
    let result = driver
        .query(By::Css(".KAiGrPfN"))
        .wait(Duration::from_secs(5), Duration::from_millis(50))
        .first()
        .await;

    let mut win_diamond = 0; //收获到的diamond
    let mut win_count = 0; //获服的次数 + 1
    if result.is_ok() {
        let x = result.unwrap().text().await;
        println!("x is: {:?}", x);
        if x.is_ok() {
            let txt = x.unwrap();
            let re = lazy_regex::regex!(r"(\d+)");
            let c = re.captures(&txt.as_str());
            if c.is_some() {
                win_diamond = c.unwrap().get(1).unwrap().as_str().parse::<i32>().unwrap();
                win_count = 1;
            }
        } else {
            println!("解析不到结果数据");
        }
    } else {
        println!("parse fd result error {:?}", result.err().unwrap());
    }

    let user_diamond = parse_user_diamond(driver).await;
    let mut diamond = 0;
    if user_diamond.is_ok() {
        diamond = user_diamond.unwrap();
    } else {
        println!("parase user error {:?}", user_diamond.err().unwrap());
    }

    println!("win count : {:?} {:?}", win_count, win_diamond);
    let _ = app.emit(
        "db",
        json!({"action":"update", "count_num":diamond,"task_msg":"完成福袋任务，准备下一个任务", "huode_diamond_count":win_diamond, "win_count": win_count, "seq":seq}),
    );

    println!("user has diamond is: {:?}", diamond);
    //-------------------------------------------------------------
    Ok(())
}

pub async fn parse_user_diamond(driver: &WebDriver) -> Result<i32, MyError> {
    driver.enter_default_frame().await?;
    let exists = driver
        .query(By::Css(".btn.e6kEVm7K > div"))
        .wait(Duration::from_secs(3), Duration::from_millis(30))
        .exists()
        .await;
    if exists.is_ok() {
        let diamond = driver
            .query(By::Css(".btn.e6kEVm7K > div"))
            .wait(Duration::from_secs(3), Duration::from_millis(20))
            .first()
            .await?;
        println!("diamond is: {:?}", diamond);
        let text = diamond.text().await;
        if text.is_err() {
            println!("文字是错误的 {:?}", text.err().unwrap());
            return Err(MyError::StringError("text出错".to_string()));
        }
        let text = text.unwrap();
        println!("text is {:?}", text);
        let re = lazy_regex::regex!(r"(\d+)");
        if let Some(c) = re.captures(&text.as_str()) {
            let diamond = c.get(1).unwrap().as_str().parse::<i32>().unwrap();
            println!("user diamond is: {:?}", diamond);
            return Ok(diamond);
        }
    } else {
        println!("未找到钻石字段");
    }
    Err(MyError::StringError("未找到这个".to_string()))
}

pub async fn parse_user_info(driver: &WebDriver) -> Result<(String, String), MyError> {
    driver
        .goto("https://www.douyin.com/user/self")
        .await
        .unwrap();
    let _ = tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    let name = driver
        .query(By::Css(".arnSiSbK > span:nth-child(1) > span:nth-child(1) > span:nth-child(1) > span:nth-child(1)"))
        .wait(Duration::from_secs(5), Duration::from_millis(100))
        .first()
        .await?
        .text()
        .await?
        .as_str()
        .to_string();
    let account = driver
        .query(By::Css("span.OcCvtZ2a"))
        .first()
        .await?
        .text()
        .await?
        .as_str()
        .to_string();
    Ok((name, account))
}

pub async fn close_other_tab(
    driver: &WebDriver,
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let cur = driver.window().await.unwrap();
    let handles = driver.windows().await?;
    for i in handles {
        if i != cur {
            driver.switch_to_window(i).await?;
            driver.close_window().await?;
        }
    }
    thread::sleep(Duration::from_secs(3));
    driver.switch_to_window(cur).await?;
    Ok(())
}

//获取直播间福袋的数据
pub async fn get(
    url: String,
    cookie: String,
) -> Result<Vec<String>, Box<dyn error::Error + Send + Sync>> {
    let rooms = vec![];
    let mut header = reqwest::header::HeaderMap::new();
    header.insert(
        reqwest::header::COOKIE,
        header::HeaderValue::from_str(&cookie.as_str()).unwrap(),
    );
    let client = reqwest::Client::builder().default_headers(header).build()?;
    let resp = client.get(url).send().await?;
    let data = resp.text().await?;
    println!("room is {:?}", data);
    Ok(rooms)
}

pub async fn post(url: String, cookie: String, params: serde_json::Value) -> Result<String, Error> {
    let mut header = header::HeaderMap::new();
    header.insert(
        header::COOKIE,
        header::HeaderValue::from_str(&cookie.as_str()).unwrap(),
    );
    let client = reqwest::Client::builder()
        .default_headers(header)
        .build()
        .unwrap();
    let resp = client.post(url).json(&params).send().await?;
    let data = resp.text().await?;
    Ok(data)
}

//从服务器获取
pub async fn fetch_room_info_from_server() {}

pub fn chromedrive_start(chromedrive_path: String) -> Result<String, Box<dyn error::Error>> {
    println!("items {:?}", chromedrive_path);
    //启动chromedriver
    let port = get_available_port();
    let mut command = std::process::Command::new(chromedrive_path);
    let _ = command
        .arg(format!("--port={}", port))
        .arg("--host=127.0.0.1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(0x08000000)
        .spawn()
        .expect("failed to start chromedriver!");
    Ok(format!("http://127.0.0.1:{}", port))
}

pub fn get_available_port() -> u16 {
    std::net::TcpListener::bind("0.0.0.0:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
