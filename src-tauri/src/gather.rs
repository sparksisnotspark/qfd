use crate::bit::MyError;
#[allow(unused)]
use reqwest::Client;
use serde_json::json;
use std::{collections::HashMap, time::Duration};

//const API_URL: &str = "http://127.0.0.1:8787";
const API_URL: &str = "http://193.134.211.160:8787";
const DEFAULT_TIMEOUT: u64 = 60;

pub async fn async_fetch_room(
    app_key: &str,
    tp: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, MyError> {
    let url: String = format!("{}/api/fetchRoom", API_URL);
    let client = Client::new();
    let json = json!({"type":tp, "app_key":app_key, "params":params});
    let resp = client
        .get(url)
        .json(&json)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .header("app-key", app_key)
        .header("Content-Type", "application/json")
        .send()
        .await?;
    let json = resp.json::<serde_json::Value>().await?;
    Ok(json)
}

pub async fn fetch_server_rooms(
    app_key: &str,
    tp: &str,
    id: u64,
) -> Result<serde_json::Value, MyError> {
    let url: String = format!("{}/api/serverRooms", API_URL);
    let client = Client::new();
    let json = json!({"tp":tp, "app_key":app_key, "id":id});
    let resp = client
        .get(url)
        .json(&json)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .header("app-key", app_key)
        .header("Content-Type", "application/json")
        .send()
        .await?;
    let json = resp.json::<serde_json::Value>().await?;
    Ok(json)
}

pub async fn get_ip() -> Result<serde_json::Value, MyError> {
    let client = Client::new();
    let resp = client
        .get("http://ip-api.com/json")
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()
        .await?
        .json()
        .await?;
    Ok(resp)
}

pub async fn check_token_expired(app_key: &str) -> Result<serde_json::Value, MyError> {
    let url: String = format!("{}/api/checkToken", API_URL);
    let client = Client::new();
    let resp = client
        .get(url)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .header("Content-Type", "application/json")
        .header("app-key", app_key)
        .send()
        .await?;
    let json = resp.json::<serde_json::Value>().await?;
    Ok(json)
}

pub fn sync_fetch_room(app_key: &str, tp: &str) -> Result<serde_json::Value, MyError> {
    let url: String = format!("{}/api/fetchRoom", API_URL);
    let client = reqwest::blocking::Client::new();
    let json = json!({"type":tp, "app_key":app_key});
    let resp = client
        .get(url)
        .json(&json)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .header("app-key", app_key)
        .header("Content-Type", "application/json")
        .send()?;
    let json = resp.json::<serde_json::Value>()?;
    Ok(json)
}

pub async fn fetch_dy_room(cookie_str: &str) -> Result<serde_json::Value, MyError> {
    let url = "https://live.douyin.com/webcast/feed/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&enter_from=page_refresh&cookie_enabled=true&screen_width=1920&screen_height=1080&browser_language=zh-CN&browser_platform=Win32&browser_name=Chrome&browser_version=142.0.0.0&channel=channel_pc_web&request_tag_from=web&need_map=1&liveid=1&is_draw=1&inner_from_drawer=0&custom_count=8&action=load_more&action_type=loadmore&enter_source=web_homepage_hot_web_live_card&source_key=web_homepage_hot_web_live_card&msToken=a-or-PT8jAMKz22HtrVQDSSWCOGlZKCU7a1hiuwIDwJ7yA9-XGhds8zVFbXUpFs3n6OsCzWSy8jBvR1sfzCQq6YF_z1KfTzLla5k9q23aw4LDXtUwbGQdR0LofaMXMmqvYAoEKyEnfugENEkQYN-ICO1nQvEVbeAisqh4jX1C_tC13TSBiubQA%3D%3D&a_bogus=mvsVgzSLYZ5bFdMbYcVMe4FUJzV%2FNTuy7lTxWNcuCxOTOqeOMbNsMcbpjxzNgy8l2Rpki9V7qnMAbxxcToUk1lHpKmhkSqGRgsVAVhXo8qiDbzt0LHbkewWzLw0x0csqa55XiIWIhUrogjVAiNdu%2FQA9tKoe5bSBF3O6k2TbO9sX1F6ADZn1PdG2OhJq0D%3D%3D";
    let client = Client::new();
    let resp = client
        .get(url)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .header(reqwest::header::USER_AGENT, "MMozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::COOKIE, cookie_str)
        .header(reqwest::header::ACCEPT, "application/json, text/javascript, */*; q=0.01")
        .header(reqwest::header::REFERER, "https://live.douyin.com/?from_nav=1")
        .send()
        .await?;
    let json = resp.json().await?;
    Ok(json)
}

pub fn sync_fetch_dy_room(cookie: String) -> Result<serde_json::Value, MyError> {
    let url = "https://live-hj.douyin.com/webcast/feed/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&enter_from=page_refresh&cookie_enabled=true&screen_width=1920&screen_height=1080&browser_language=zh-CN&browser_platform=Win32&browser_name=Chrome&browser_version=142.0.0.0&channel=channel_pc_web&request_tag_from=web&need_map=1&liveid=1&is_draw=1&inner_from_drawer=0&custom_count=8&action=load_more&action_type=loadmore&enter_source=web_homepage_hot_web_live_card&source_key=web_homepage_hot_web_live_card&msToken=EFV4C2zDfEjt_EEQ5n24H9U_xA4mG6q4EeWWe5ng_l9wPM8XmJo9aK2_W8UunFbIT_EMk8nLCmagvH1sqcI3dUksx0Tv5rJXM053FPcjCqdXmBsl8J2ASPMMy3SpJs7pkIiiDWp04vqFHHGbN3yOh06RNz1jYWkJ7CGJiJzc3CvHs3kaf1mWSPE%3D&a_bogus=xfs5h7UjmomnFVKb8CftCVaUcSoMrsSy3lT2SaluSxPIc7UcxuP0Qxa0roFi%2FQMdwup0ioV7AnU%2FYExcz9XT1MHpomhvSqwfqTQ9VSXoMHJ6Tak29ZW8ebbxKJsGWSsYO5Avi5wXI0FwI2O3hrn%2FAVVaH5zOQORDSNFbd28ca9ANVSfHnoQHeQvWaXy%3D";
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(url)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::ACCEPT, "application/json, text/javascript, */*; q=0.01")
        .header(reqwest::header::REFERER, "https://live.douyin.com/?from_nav=1")
        .header(reqwest::header::COOKIE, cookie)
        .send()?;
    let resp_str = resp.text()?;
    let json = serde_json::from_str::<serde_json::Value>(&resp_str)?;
    Ok(json)
}

pub async fn user_login_system(
    username: &str,
    password: &str,
    token: &str,
) -> Result<serde_json::Value, MyError> {
    let url = format!("{}/login", API_URL);
    let client = Client::new();
    let mut params = HashMap::new();
    params.insert("username", username);
    params.insert("password", password);
    params.insert("token", token);
    let resp = client
        .post(url)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .form(&params)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await
        .unwrap();
    Ok(resp)
}

pub fn parse_luckbox_info(
    room_id: &str,
    web_rid: &str,
    cookie: &str,
    min_diamond: u64,
) -> Result<serde_json::Value, MyError> {
    let url = format!("https://live.douyin.com/webcast/luckybox/box/list/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&enter_from=web_live&cookie_enabled=true&screen_width=1680&screen_height=1050&browser_language=zh-CN&browser_platform=MacIntel&browser_name=Chrome&browser_version=142.0.0.0&room_id={}", room_id);
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(url)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36")
        .header(
            reqwest::header::REFERER,
            format!("https://live.douyin.com/{}", web_rid),
        )
        .timeout(Duration::from_secs(30))
        .header(reqwest::header::COOKIE, cookie)
        .send()?;
    let json = resp.json::<serde_json::Value>()?;
    let data = json["data"].as_array();
    if data.is_none() {
        return Err(MyError::StringError("无数据".to_string()));
    }
    let data = data.unwrap();
    let mut packet_list: Vec<serde_json::Value> = vec![];
    let mut all_gift = true;
    for i in data {
        let box_type = i["box_type"].as_i64().unwrap();
        if box_type == 0 {
            all_gift = false;
        } else { //如果不是钻石的，就跳到下一个
            continue;
        }
        let business_type = i["business_type"].as_i64().unwrap();
        //println!("bussiness type is: {:?}", business_type);
        let pack_count = i["biz_extra"].as_str().unwrap();
        let extra_ = serde_json::from_str::<serde_json::Value>(pack_count).unwrap();

        let pack_count = extra_["tags"][business_type.to_string().as_str()]
            .as_str()
            .unwrap();
        let diamond_count = i["diamond_count"].as_u64().unwrap();

        let pack_count_u64 = pack_count.parse::<u64>().unwrap();

        let avg_diamond = (diamond_count as f64) / (pack_count_u64 as f64);

        if avg_diamond.round() < min_diamond as f64 {
            //println!( "avg diamond {} is less than min diamond {}", avg_diamond, min_diamond);
            continue;
        }

        let title = i["title"].as_str().unwrap();
        let send_time = i["send_time"].as_i64().unwrap() / 1000;
        let delay_time = i["delay_time"].as_i64().unwrap();
        let room_url = format!("https://live.douyin.com/{}", web_rid);

        //println!( "box type is: {:?} pack count {:?} diamond_count {:?} send time {:?} delay time {:?} room url {:?}", box_type, pack_count, diamond_count, send_time, delay_time, room_url);

        let json_data = json!({"room_url":room_url, "room_id":room_id, "web_rid":web_rid, "pack_count":pack_count, "diamond_count": diamond_count, "title":title, "send_time":send_time, "delay":delay_time});
        packet_list.push(json_data);
    }

    if all_gift && data.len() > 0 {
        return Err(MyError::StringError("都是礼物红包，下一个".to_string()));
    }

    if !packet_list.is_empty() {
        packet_list.sort_by(|a, b| {
            let ac_ = a["send_time"].as_i64().unwrap();
            let bc_ = b["send_time"].as_i64().unwrap();
            bc_.cmp(&ac_)
        });
        let out = packet_list.pop().unwrap();
        return Ok(out);
    }

    if data.len() == 0 {
        return Err(MyError::StringError("没有找到红包信息".to_string()));
    }

    Err(MyError::StringError("每个包小于最小的红包".to_string()))
}

pub fn parse_lottery_info(
    room_id: &str,
    web_rid: &str,
    cookie: &str,
    only_big_packet: bool,
    join_fans_group: bool,
    people_limit: u64,
    winning_ratio: u64,
    min_avg_diamond: u64,
) -> Result<serde_json::Value, MyError> {
    let url = format!("https://live.douyin.com/webcast/lottery/melon/lottery_info/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&enter_from=web_live&cookie_enabled=true&screen_width=1920&screen_height=1080&browser_language=zh-CN&browser_platform=Win32&browser_name=Chrome&browser_version=141.0.0.0&room_id={}&query_from=1&msToken=RAyuluDXqzOK2qujtiZCq9HHm1pbAM_Da8umCUHWr4V55TUEhlWs2EgcgliOavYKkeEuRqCWRxd7plgF0vt0oMjA3pofShhVSMihz7o6X19vhADQM6qtYn_yBYaiFbPoNGvXUHekTF4I2dKvMT8YaYdwlopoRtKtrVwv_vuc2pd9tVNtT1xWgw%3D%3D&a_bogus=mX0fgwWJEZR5ed%2FSmO4LeA2U4uEMrT8yfMixbc5u7PKhOhFP7WN%2FBrcxaxoV%2F%2FVKnup0hoV7pDzAbEnc0xXT1l9pLmhDSFG6yG2c9tso8ZH4GsJ2XNf8CJSEFiPYUAGYm5AniMW51sMeIdOWIHCzAB3HF%2FvxRRjdMH-JVMYjE9u40C8jio%2FIaV72Ph0qPf%3D%3D", room_id);
    let client = reqwest::blocking::Client::new();
    let agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36";
    let resp = client
        .get(url)
        .header(
            reqwest::header::REFERER,
            format!("https://live.douyin.com/{}", web_rid),
        )
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .header(reqwest::header::USER_AGENT, agent)
        .header(reqwest::header::COOKIE, cookie)
        .send()?;

    let json = resp.json::<serde_json::Value>()?;
    if json["data"].as_object().is_some() {
        let info = json["data"].as_object().unwrap();
        if info.contains_key("lottery_info") {
            //println!("info is: {:?}", info);
            let prize_count = info["lottery_info"]["prize_count"].as_i64().unwrap();
            let lucky_count = info["lottery_info"]["lucky_count"].as_i64().unwrap();
            let conditions = info["lottery_info"]["conditions"].as_array().unwrap();
            let count_down = info["lottery_info"]["count_down"].as_i64().unwrap();
            let start_time = info["lottery_info"]["start_time"].as_i64().unwrap();
            let draw_time = info["lottery_info"]["draw_time"].as_i64().unwrap();
            let candidate_num = info["lottery_info"]["candidate_num"].as_i64().unwrap();
            let room_url = format!("https://live.douyin.com/{}", web_rid);
            let mut s: Vec<String> = vec![];
            let mut need_diamond = 0;
            conditions.iter().for_each(|f| {
                s.push(f["description"].as_str().unwrap().to_string());
                let nd = f["need_diamond_count"].as_i64().unwrap();
                if nd > 0 {
                    need_diamond = 1;
                }
            });

            //println!( "only big package {:?} people limit {:?}", only_big_packet, people_limit);

            if need_diamond > 0 && join_fans_group == false {
                println!("需要加粉丝团, 粉丝团没有打勾");
                return Err(MyError::StringError("需要加粉丝团".to_string()));
            }

            let per_pack = prize_count as f64 / lucky_count as f64;

            if per_pack < min_avg_diamond as f64 {
                // 大于等于表示OK
                println!("福袋金额太小 {:?} {:?}", per_pack, min_avg_diamond);
                return Err(MyError::StringError(
                    format!(
                        "福袋金额太小 per pack {:?} lucky count: {:?}",
                        per_pack, lucky_count
                    )
                    .to_string(),
                ));
            }

            let rate = lucky_count as f64 / candidate_num as f64;

            if rate * 100.0 < winning_ratio as f64 {
                println!("中奖率问题 {:?} {:?}", rate * 100.0, winning_ratio);
                return Err(MyError::StringError(
                    format!(
                        "中奖率太低 lucky count{:?}  抢的人{:?}",
                        lucky_count, candidate_num
                    )
                    .to_string(),
                ));
            }

            let title = s.join(",");
            println!(
                "{:?} {:?} {:?} {:?} {:?}",
                prize_count, lucky_count, draw_time, start_time, candidate_num
            );
            let ret_json = json!({
                "prize_count": prize_count,
                "lucky_count": lucky_count, 
                "need_diamond": need_diamond, 
                "title":title,
                "delay": count_down, 
                "send_time":start_time, 
                "draw_time":draw_time,
                "candidate_num": candidate_num,
                "room_url": room_url, 
                "room_id": room_id, 
                "web_rid": web_rid});
            //println!("采集符合条件的数据成功 {:?}", ret_json);
            return Ok(ret_json);
        } else {
            //println!("did not contains lottery info");
        }
    }
    Err(MyError::StringError("没有福袋的数据".to_string()))
}

pub async fn upload_server(params: serde_json::Value, token: &str) {
    let url = "{}/updateRoom";
    let client = Client::new();
    let _ = client
        .post(url)
        .header("app-key", token)
        .json(&params)
        .send()
        .await;
}

pub async fn add_one_room(token: &str, params: serde_json::Value) -> Result<(), MyError> {
    let url = format!("{}/api/addRoom", API_URL);
    println!("token is: {:?}", token);
    let client = Client::new();
    let r = client
        .post(url)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .header("app-key", token)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36")
        .json(&params)
        .send()
        .await;
    if r.is_err() {
        println!("error of this is: {:?}", r);
    }
    Ok(())
}

pub fn sync_add_one_room(token: &str, params: serde_json::Value) -> Result<(), MyError> {
    let url = format!("{}/api/addRoom", API_URL);
    println!("token is: {:?}", token);
    let client = reqwest::blocking::Client::new();
    let r = client
        .post(url)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .header("app-key", token)
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&params)
        .send()?;
    println!("return add room result is: {:?}", r);
    Ok(())
}
