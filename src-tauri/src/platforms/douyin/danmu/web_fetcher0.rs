use crate::platforms::common::http_client::HttpClient;
use regex::Regex;
use reqwest::header::{HeaderName, ACCEPT, ACCEPT_LANGUAGE, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use super::signature; // Assuming signature.rs is in the same directory (src)

// New struct for frontend
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DouyinFollowListRoomInfo {
    pub room_id_str: String,
    pub nickname: String,
    pub room_name: String, // Title of the room
    pub avatar_url: String,
    pub status: i32, // 0 for live, other values indicate not live or error
}

pub struct DouyinLiveWebFetcher {
    pub live_id: String,
    pub ttwid: Option<String>,
    pub room_id: Option<String>,
    pub user_agent: String,
    pub http_client: HttpClient,
    pub(crate) _ws_stream: Option<Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
}

impl DouyinLiveWebFetcher {
    pub fn new(live_id: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // 使用直连HTTP客户端，绕过所有代理设置
        let http_client = HttpClient::new_direct_connection()
            .map_err(|e| format!("Failed to create direct connection HttpClient: {}", e))?;

        Ok(DouyinLiveWebFetcher {
            live_id: live_id.to_string(),
            ttwid: None,
            room_id: None,
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            http_client,
            _ws_stream: None,
        })
    }

    pub async fn get_ttwid(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ttwid) = &self.ttwid {
            return Ok(ttwid.clone());
        }

        // 使用固定的 ttwid 值（来自fetch_douyin_data_new.js）
        let fixed_ttwid = "1%7CMzira2CT1P-CLey42gr0QsEGL_Wmq3Yg5PQF2X412hY%7C1677897397%7C0df7a1da2a44ccac7dda848d236c8d5276d3eae070dfb3fe6df6e86fbd896d93".to_string();
        self.ttwid = Some(fixed_ttwid.clone());
        Ok(fixed_ttwid)
    }

    pub async fn get_room_id(
        &mut self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(room_id) = &self.room_id {
            return Ok(room_id.clone());
        }

        let ttwid = self.get_ttwid().await?;
        // 使用固定的 msToken 值（来自fetch_douyin_data_new.js）
        let ms_token = "djIQSLNfdq3BLVY9-hIFbpJVQs238wUtsl1_Zvc2-rkmUSUy44JUt-L_jMcpo--kcwpK8Sc4C7fUvX-QL-mrqE1RM0E65tIZ8Rz4UoVXrzbCAhvwNKSX0TG8r1KNdI3K9dbBvI3Lb6W62nr7LStyw-41pkfZkFW2Vfi9zqnnLDSM-NMhCJTrxQ%3D%3D";
        let ac_nonce = "0123407cc00a9e438deb4";
        let cookie_header = format!(
            "ttwid={}; msToken={}; __ac_nonce={}",
            ttwid, ms_token, ac_nonce
        );

        let url = format!("https://live.douyin.com/{}", self.live_id);
        
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::COOKIE,
            reqwest::header::HeaderValue::from_str(&cookie_header)
                .map_err(|e| format!("Failed to create cookie header: {}", e))?
        );
        headers.insert(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse().unwrap());
        headers.insert(ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9,en;q=0.8".parse().unwrap());
        headers.insert(HeaderName::from_static("priority"), "u=0, i".parse().unwrap());
        headers.insert(HeaderName::from_static("sec-ch-ua"), "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"".parse().unwrap());
        headers.insert(HeaderName::from_static("sec-ch-ua-mobile"), "?0".parse().unwrap());
        headers.insert(HeaderName::from_static("sec-ch-ua-platform"), "\"macOS\"".parse().unwrap());
        headers.insert(HeaderName::from_static("sec-fetch-dest"), "document".parse().unwrap());
        headers.insert(HeaderName::from_static("sec-fetch-mode"), "navigate".parse().unwrap());
        headers.insert(HeaderName::from_static("sec-fetch-site"), "none".parse().unwrap());
        headers.insert(HeaderName::from_static("sec-fetch-user"), "?1".parse().unwrap());
        headers.insert(HeaderName::from_static("upgrade-insecure-requests"), "1".parse().unwrap());
        headers.insert(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".parse().unwrap());
        
        let text = self.http_client.get_text_with_headers(&url, Some(headers)).await
            .map_err(|e| {
                // 获取调试信息
                let debug_headers = self.http_client.get_debug_headers();
                let debug_cookies = self.http_client.get_debug_cookies(&url);
                
                println!("[Douyin Danmu WebFetcher] get_room_page API request failed.");
                println!("URL: {}", url);
                println!("Headers:\n{}", debug_headers);
                println!("Cookies: {}", debug_cookies);
                println!("Error: {}", e);
                
                format!("Failed to get room page: {}", e)
            })?;
        println!(
            "HTML Response (first 500 chars): {}",
            &text[..std::cmp::min(500, text.len())]
        );

        let re = Regex::new(r#"\\"roomId\\":\\"(\d+)\\""#).unwrap();

        if let Some(caps) = re.captures(&text) {
            if let Some(room_id_match) = caps.get(1) {
                let room_id_val = room_id_match.as_str().to_string();
                self.room_id = Some(room_id_val.clone());
                println!("Fetched room_id: {}", room_id_val);
                return Ok(room_id_val);
            }
        }
        Err("roomId not found in response".into())
    }

    pub async fn get_room_status(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let room_id_val = self.get_room_id().await?;
        let ttwid_val = self.get_ttwid().await?;
        // 使用固定的 msToken 值（来自fetch_douyin_data_new.js）
        let ms_token = "djIQSLNfdq3BLVY9-hIFbpJVQs238wUtsl1_Zvc2-rkmUSUy44JUt-L_jMcpo--kcwpK8Sc4C7fUvX-QL-mrqE1RM0E65tIZ8Rz4UoVXrzbCAhvwNKSX0TG8r1KNdI3K9dbBvI3Lb6W62nr7LStyw-41pkfZkFW2Vfi9zqnnLDSM-NMhCJTrxQ%3D%3D";
        let ac_nonce = "0123407cc00a9e438deb4";
        let cookie_header = format!(
            "ttwid={}; msToken={}; __ac_nonce={}",
            ttwid_val, ms_token, ac_nonce
        );

        let url = format!(
            "https://live.douyin.com/webcast/room/web/enter/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&enter_from=web_homepage_follow&cookie_enabled=true&screen_width=1920&screen_height=1080&browser_language=zh-CN&browser_platform=MacIntel&browser_name=Chrome&browser_version=140.0.0.0&web_rid={}&room_id_str={}&enter_source=&is_need_double_stream=false&insert_task_id=&live_reason=&msToken=djIQSLNfdq3BLVY9-hIFbpJVQs238wUtsl1_Zvc2-rkmUSUy44JUt-L_jMcpo--kcwpK8Sc4C7fUvX-QL-mrqE1RM0E65tIZ8Rz4UoVXrzbCAhvwNKSX0TG8r1KNdI3K9dbBvI3Lb6W62nr7LStyw-41pkfZkFW2Vfi9zqnnLDSM-NMhCJTrxQ%3D%3D&a_bogus=EJ0fkF67Dx%2FfPdKGuObyCHlU2lxMNB8yQZixWCluCNzJOXUTjuP7gcbZboqs4doR3bpsiHIHTx0lYEncTdUs1ZrkumkfSmzyJzACVgsL8qwsGFJQgHfZeukFqwBN0Rsqa%2FcIE1g78sBK2d5W9HAQldBaC5Pa5QmDWHqydM9bj9WbDAyPu3rROMEWiEwPBQ2-rf%3D%3D",
            self.live_id, // web_rid should be self.live_id (which is the original ID from URL, e.g. username or a numerical ID)
            room_id_val // room_id_str is the actual numerical room ID
        );

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::COOKIE,
            reqwest::header::HeaderValue::from_str(&cookie_header)
                .map_err(|e| format!("Failed to create cookie header: {}", e))?
        );
        headers.insert(ACCEPT, "application/json, text/plain, */*".parse().unwrap());
        headers.insert(ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9,en;q=0.8".parse().unwrap());
        headers.insert(HeaderName::from_static("priority"), "u=1, i".parse().unwrap());
        headers.insert(REFERER, "https://live.douyin.com/".parse().unwrap());
        headers.insert(HeaderName::from_static("sec-ch-ua"), "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"".parse().unwrap());
        headers.insert(HeaderName::from_static("sec-ch-ua-mobile"), "?0".parse().unwrap());
        headers.insert(HeaderName::from_static("sec-ch-ua-platform"), "\"macOS\"".parse().unwrap());
        headers.insert(HeaderName::from_static("sec-fetch-dest"), "empty".parse().unwrap());
        headers.insert(HeaderName::from_static("sec-fetch-mode"), "cors".parse().unwrap());
        headers.insert(HeaderName::from_static("sec-fetch-site"), "same-origin".parse().unwrap());
        headers.insert(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".parse().unwrap());
        
        // 先获取文本响应来调试
        let response_text = self.http_client.get_text_with_headers(&url, Some(headers.clone())).await
            .map_err(|e| {
                // 获取调试信息
                let debug_headers = self.http_client.get_debug_headers();
                let debug_cookies = self.http_client.get_debug_cookies(&url);
                
                println!("[Douyin Danmu WebFetcher] get_room_status API request failed.");
                println!("URL: {}", url);
                println!("Headers:\n{}", debug_headers);
                println!("Cookies: {}", debug_cookies);
                println!("Error: {}", e);
                
                format!("Failed to get room status: {}", e)
            })?;
        
        println!("[DEBUG] Response text (first 1000 chars): {}", &response_text[..std::cmp::min(1000, response_text.len())]);
        
        // 尝试解析JSON
        let data: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| {
                println!("[ERROR] JSON parsing failed: {}", e);
                println!("[ERROR] Full response text: {}", response_text);
                format!("Failed to parse JSON response: {}", e)
            })?;

        // This part is mostly for printing/debugging in the original code
        if let Some(room_data_top) = data.get("data") {
            // Douyin API often has a nested "data" field for room details
            if let Some(room_info) = room_data_top.get("room") {
                // Actual room details are often in a "room" sub-object
                let room_status_val = room_info.get("status").and_then(|s| s.as_i64());
                // User data is often in an "owner" sub-object of "room"
                if let Some(user_data) = room_info.get("owner") {
                    let user_id = user_data.get("id_str").and_then(|s| s.as_str());
                    let nickname = user_data.get("nickname").and_then(|s| s.as_str());

                    if let (Some(status), Some(id), Some(nick)) =
                        (room_status_val, user_id, nickname)
                    {
                        let status_text = if status == 0 {
                            "正在直播"
                        } else {
                            "已结束"
                        };
                        println!("【{}】[{}]直播间：{}.", nick, id, status_text);
                    } else {
                        println!("【X】无法解析直播间信息的部分字段 (status, id, nick)");
                    }
                } else {
                    println!("【X】未找到用户信息 (owner data in room_data.room)");
                }
            } else {
                println!("【X】未找到房间信息 (room object in room_data_top)");
            }
        } else {
            println!("【X】未找到顶层房间数据 (data object in response)");
        }
        Ok(())
    }

    pub async fn fetch_room_details(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.get_ttwid().await?;
        self.get_room_id().await?;
        self.get_room_status().await?; // Optional: for debugging or if status is needed before connection
        Ok(())
    }

    // pub async fn connect_websocket_placeholder(&mut self, _room_id_param: &str, _ttwid_param: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    //     println!("Connect_websocket logic will be moved elsewhere.");
    //     Ok(())
    // }
}

// New Tauri command
#[tauri::command]
pub async fn fetch_douyin_room_info(live_id: String) -> Result<DouyinFollowListRoomInfo, String> {
    println!(
        "[fetch_douyin_room_info] Fetching details for live_id: {}",
        live_id
    );
    let mut fetcher = DouyinLiveWebFetcher::new(&live_id)
        .map_err(|e| format!("Failed to create DouyinLiveWebFetcher: {}", e))?;

    let ttwid = fetcher
        .get_ttwid()
        .await
        .map_err(|e| format!("Failed to get ttwid: {}", e))?;
    let room_id_str = fetcher
        .get_room_id()
        .await
        .map_err(|e| format!("Failed to get room_id: {}", e))?;

    // Construct the URL for the web/enter endpoint
    let url = format!(
        "https://live.douyin.com/webcast/room/web/enter/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&enter_from=web_homepage_follow&cookie_enabled=true&screen_width=1920&screen_height=1080&browser_language=zh-CN&browser_platform=MacIntel&browser_name=Chrome&browser_version=140.0.0.0&web_rid={}&room_id_str={}&enter_source=&is_need_double_stream=false&insert_task_id=&live_reason=&msToken=djIQSLNfdq3BLVY9-hIFbpJVQs238wUtsl1_Zvc2-rkmUSUy44JUt-L_jMcpo--kcwpK8Sc4C7fUvX-QL-mrqE1RM0E65tIZ8Rz4UoVXrzbCAhvwNKSX0TG8r1KNdI3K9dbBvI3Lb6W62nr7LStyw-41pkfZkFW2Vfi9zqnnLDSM-NMhCJTrxQ%3D%3D&a_bogus=EJ0fkF67Dx%2FfPdKGuObyCHlU2lxMNB8yQZixWCluCNzJOXUTjuP7gcbZboqs4doR3bpsiHIHTx0lYEncTdUs1ZrkumkfSmzyJzACVgsL8qwsGFJQgHfZeukFqwBN0Rsqa%2FcIE1g78sBK2d5W9HAQldBaC5Pa5QmDWHqydM9bj9WbDAyPu3rROMEWiEwPBQ2-rf%3D%3D",
        live_id, // web_rid (this should be the original live_id from input)
        room_id_str // room_id_str (the numerical one we fetched)
    );

    // 使用固定的 msToken 值（来自fetch_douyin_data_new.js）
    let ms_token = "djIQSLNfdq3BLVY9-hIFbpJVQs238wUtsl1_Zvc2-rkmUSUy44JUt-L_jMcpo--kcwpK8Sc4C7fUvX-QL-mrqE1RM0E65tIZ8Rz4UoVXrzbCAhvwNKSX0TG8r1KNdI3K9dbBvI3Lb6W62nr7LStyw-41pkfZkFW2Vfi9zqnnLDSM-NMhCJTrxQ%3D%3D";
    let ac_nonce = "0123407cc00a9e438deb4";
    let cookie_header = format!(
        "ttwid={}; msToken={}; __ac_nonce={}",
        ttwid, ms_token, ac_nonce
    );

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::COOKIE,
        reqwest::header::HeaderValue::from_str(&cookie_header)
            .map_err(|e| format!("Failed to create cookie header: {}", e))?
    );
    headers.insert(ACCEPT, "application/json, text/plain, */*".parse().unwrap());
    headers.insert(ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9,en;q=0.8".parse().unwrap());
    headers.insert(HeaderName::from_static("priority"), "u=1, i".parse().unwrap());
    headers.insert(REFERER, "https://live.douyin.com/".parse().unwrap());
    headers.insert(HeaderName::from_static("sec-ch-ua"), "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"".parse().unwrap());
    headers.insert(HeaderName::from_static("sec-ch-ua-mobile"), "?0".parse().unwrap());
    headers.insert(HeaderName::from_static("sec-ch-ua-platform"), "\"macOS\"".parse().unwrap());
    headers.insert(HeaderName::from_static("sec-fetch-dest"), "empty".parse().unwrap());
    headers.insert(HeaderName::from_static("sec-fetch-mode"), "cors".parse().unwrap());
    headers.insert(HeaderName::from_static("sec-fetch-site"), "same-origin".parse().unwrap());
    headers.insert(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".parse().unwrap());
    
    let data: serde_json::Value = fetcher.http_client.get_json_with_headers(&url, Some(headers)).await
        .map_err(|e| format!("Failed to get room info from web/enter: {}", e))?;

    // Parse data based on typical Douyin API structure
    let room_data_top = data
        .get("data")
        .ok_or_else(|| "Missing 'data' field in response".to_string())?;
    let room_info = room_data_top
        .get("room")
        .ok_or_else(|| "Missing 'room' field in data".to_string())?;
    let owner_info = room_info
        .get("owner")
        .ok_or_else(|| "Missing 'owner' field in room data".to_string())?;

    let nickname = owner_info
        .get("nickname")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let avatar_url = owner_info
        .get("avatar_thumb")
        .and_then(|v| v.get("url_list"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.get(0))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let room_name = room_info
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let status_val = room_info
        .get("status")
        .and_then(|v| v.as_i64())
        .unwrap_or(4) as i32; // Default to 4 (not live) if not found or parsing fails

    if nickname.is_empty() && room_name.is_empty() {
        return Err(format!("Failed to extract critical info (nickname/room_name empty). Parsed status: {}. Room ID: {}", status_val, room_id_str));
    }

    Ok(DouyinFollowListRoomInfo {
        room_id_str: room_id_str.clone(), // Use the fetched numerical room_id_str
        nickname,
        room_name,
        avatar_url,
        status: status_val, // status 0 means live
    })
}
