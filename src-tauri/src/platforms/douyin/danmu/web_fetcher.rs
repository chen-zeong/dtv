use crate::platforms::common::http_client::HttpClient;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use super::signature; // Assuming signature.rs is in the same directory (src)
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, REFERER, USER_AGENT};

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
    // 新增字段：用于 WebSocket 和签名所需
    pub dy_cookie: Option<String>,
    pub user_unique_id: Option<String>,
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
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 Edg/125.0.0.0".to_string(),
            http_client,
            _ws_stream: None,
            dy_cookie: None,
            user_unique_id: None,
        })
    }

    pub async fn get_ttwid(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ttwid) = &self.ttwid {
            return Ok(ttwid.clone());
        }

        let live_url = "https://live.douyin.com/";
        let response = self.http_client.get_with_cookies(live_url).await
            .map_err(|e| format!("Failed to get response from {}: {}", live_url, e))?;

        let ttwid_val = response
            .cookies()
            .find(|c| c.name() == "ttwid")
            .map(|c| c.value().to_string());

        if let Some(ttwid) = ttwid_val {
            self.ttwid = Some(ttwid.clone());
            // println!("Fetched ttwid: {}", ttwid);
            Ok(ttwid)
        } else {
            Err("ttwid not found in cookies".into())
        }
    }

    // Collect cookies and parse HTML to obtain real room_id and user_unique_id
    pub async fn collect_cookies_and_ids(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("https://live.douyin.com/{}", self.live_id);
        // HEAD to collect initial cookies
        let head_resp = self.http_client.inner
            .head(&url)
            .header("User-Agent", &self.user_agent)
            .header("Referer", "https://live.douyin.com")
            .header("Authority", "live.douyin.com")
            .send()
            .await?;
        let mut dy_cookie = String::new();
        for val in head_resp.headers().get_all("set-cookie").iter() {
            if let Ok(s) = val.to_str() {
                let first = s.split(';').next().unwrap_or("");
                if first.contains("ttwid") || first.contains("__ac_nonce") || first.contains("msToken") || first.contains("s_v_web_id") || first.contains("tt_scid") {
                    dy_cookie.push_str(first);
                    dy_cookie.push(';');
                }
            }
        }
        // GET to complete cookies and fetch HTML
        let get_resp = self.http_client.inner
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .header("Referer", "https://live.douyin.com")
            .send()
            .await?;
        for val in get_resp.headers().get_all("set-cookie").iter() {
            if let Ok(s) = val.to_str() {
                let first = s.split(';').next().unwrap_or("");
                if first.contains("ttwid") || first.contains("__ac_nonce") || first.contains("msToken") || first.contains("s_v_web_id") || first.contains("tt_scid") {
                    if !dy_cookie.contains(first) {
                        dy_cookie.push_str(first);
                        dy_cookie.push(';');
                    }
                }
            }
        }
        let html = get_resp.text().await?;
        // Parse renderData to extract room_id and user_unique_id
        let re = Regex::new(r#"\{\\\"state\\\":\{\\\"appStore.*?\]\\n"#).unwrap();
        let render = match re.find(&html) {
            Some(m) => m.as_str().to_string(),
            None => {
                return Err("Failed to locate renderData in Douyin room HTML".into());
            }
        };
        let json_str = render.trim().replace("\\\"", "\"").replace("\\\\", "\\").replace("]\\n", "");
        let v: serde_json::Value = serde_json::from_str(&json_str)?;
        let state = &v["state"];
        let real_room_id = state["roomStore"]["roomInfo"]["room"]["id_str"].as_str().unwrap_or("").to_string();
        let user_unique_id = state["userStore"]["odin"]["user_unique_id"].as_str().unwrap_or("").to_string();
        if real_room_id.is_empty() || user_unique_id.is_empty() {
            return Err("Failed to parse room_id or user_unique_id from HTML state".into());
        }
        self.room_id = Some(real_room_id);
        self.user_unique_id = Some(user_unique_id);
        self.dy_cookie = Some(dy_cookie);
        Ok(())
    }

    pub async fn get_room_id(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(room_id) = &self.room_id { return Ok(room_id.clone()); }
        self.collect_cookies_and_ids().await?;
        Ok(self.room_id.clone().unwrap())
    }

    pub async fn get_user_unique_id(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(uid) = &self.user_unique_id { return Ok(uid.clone()); }
        self.collect_cookies_and_ids().await?;
        Ok(self.user_unique_id.clone().unwrap())
    }

    pub async fn get_dy_cookie(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(cookie) = &self.dy_cookie { return Ok(cookie.clone()); }
        self.collect_cookies_and_ids().await?;
        Ok(self.dy_cookie.clone().unwrap())
    }

    pub async fn fetch_room_details(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 仅依赖 HTML 解析，获取真实 room_id 和 user_unique_id，并收集 Cookie
        self.collect_cookies_and_ids().await?;
        Ok(())
    }

    pub async fn get_room_status(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Ensure we have room_id and cookies collected
        let room_id_val = self.get_room_id().await?;
        let dy_cookie = self.get_dy_cookie().await?;
        let user_unique_id = self.get_user_unique_id().await?;

        // Parse msToken from collected cookie string (format: "key=value; key2=value2; ...")
        let ms_token = dy_cookie
            .split(';')
            .filter_map(|kv| {
                let kv = kv.trim();
                if kv.starts_with("msToken=") {
                    Some(kv.trim_start_matches("msToken=").to_string())
                } else {
                    None
                }
            })
            .next()
            .unwrap_or_default();

        // Build minimal and consistent URL using new scheme
        let base_url = "https://live.douyin.com/webcast/room/web/enter/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&cookie_enabled=true";
        let url = if ms_token.is_empty() {
            // Fallback: omit msToken if not present (server may read from Cookie)
            format!(
                "{}&room_id={}&user_unique_id={}",
                base_url, room_id_val, user_unique_id
            )
        } else {
            format!(
                "{}&room_id={}&msToken={}&user_unique_id={}",
                base_url, room_id_val, ms_token, user_unique_id
            )
        };

        // Prepare per-request headers: Accept, Accept-Language, Referer, User-Agent, Cookie
        // Update default User-Agent to the one stored in fetcher to avoid override by HttpClient::send_request
        if let Err(e) = self.http_client.insert_header(USER_AGENT, &self.user_agent) {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to set USER_AGENT header: {}", e)).into());
        }

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(ACCEPT, reqwest::header::HeaderValue::from_static("application/json, text/plain, */*"));
        headers.insert(ACCEPT_LANGUAGE, reqwest::header::HeaderValue::from_static("zh-CN,zh;q=0.9"));
        headers.insert(REFERER, reqwest::header::HeaderValue::from_str(&format!("https://live.douyin.com/{}", self.live_id)).unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("https://live.douyin.com")));
        headers.insert(reqwest::header::HeaderName::from_static("cookie"), reqwest::header::HeaderValue::from_str(&dy_cookie).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Invalid Cookie header: {}", e)))?);

        let data: serde_json::Value = match self.http_client.get_json_with_headers(&url, Some(headers)).await {
            Ok(v) => v,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to get room status: {}", e)).into()),
        };

        // Parse and log basic room and owner info
        if let Some(room_data_top) = data.get("data") {
            if let Some(room_info) = room_data_top.get("room") {
                let room_status_val = room_info.get("status").and_then(|s| s.as_i64());
                if let Some(user_data) = room_info.get("owner") {
                    let user_id = user_data.get("id_str").and_then(|s| s.as_str());
                    let nickname = user_data.get("nickname").and_then(|s| s.as_str());

                    if let (Some(status), Some(id), Some(nick)) = (room_status_val, user_id, nickname) {
                        let status_text = if status == 0 { "正在直播" } else { "已结束" };
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

    // fetch_room_details moved earlier to only collect cookies and HTML IDs; old implementation removed.

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

    // Ensure cookies and ids are collected
    fetcher
        .fetch_room_details()
        .await
        .map_err(|e| format!("Failed to collect cookies and ids: {}", e))?;

    let room_id_str = fetcher
        .get_room_id()
        .await
        .map_err(|e| format!("Failed to get room_id: {}", e))?;
    let dy_cookie = fetcher
        .get_dy_cookie()
        .await
        .map_err(|e| format!("Failed to get dy_cookie: {}", e))?;
    let user_unique_id = fetcher
        .get_user_unique_id()
        .await
        .map_err(|e| format!("Failed to get user_unique_id: {}", e))?;

    // Parse msToken from dy_cookie if present
    let ms_token = dy_cookie
        .split(';')
        .filter_map(|kv| {
            let kv = kv.trim();
            if kv.starts_with("msToken=") {
                Some(kv.trim_start_matches("msToken=").to_string())
            } else {
                None
            }
        })
        .next()
        .unwrap_or_default();

    // Construct the URL for the web/enter endpoint with the new unified scheme
    let base_url = "https://live.douyin.com/webcast/room/web/enter/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&cookie_enabled=true";
    let url = if ms_token.is_empty() {
        format!("{}&room_id={}&user_unique_id={}", base_url, room_id_str, user_unique_id)
    } else {
        format!("{}&room_id={}&msToken={}&user_unique_id={}", base_url, room_id_str, ms_token, user_unique_id)
    };

    // Prepare headers per request
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(ACCEPT, reqwest::header::HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert(ACCEPT_LANGUAGE, reqwest::header::HeaderValue::from_static("zh-CN,zh;q=0.9"));
    headers.insert(REFERER, reqwest::header::HeaderValue::from_str(&format!("https://live.douyin.com/{}", live_id)).unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("https://live.douyin.com")));
    headers.insert(reqwest::header::HeaderName::from_static("cookie"), reqwest::header::HeaderValue::from_str(&dy_cookie).map_err(|e| format!("Invalid Cookie header: {}", e))?);

    // Perform request directly with inner client to ensure per-request headers (especially Cookie) are honored
    let resp = fetcher.http_client.inner.get(&url).headers(headers).send().await.map_err(|e| format!("Failed to send request: {}", e))?;
    let status = resp.status();
    if !status.is_success() {
        let err_text = resp.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
        return Err(format!("GET JSON {} failed with status {}: {}", url, status, err_text));
    }
    let data: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Extract required fields
    let (nickname, room_name, avatar_url, status_i32) = {
        if let Some(room_data_top) = data.get("data") {
            if let Some(room_info) = room_data_top.get("room") {
                let status_val = room_info.get("status").and_then(|s| s.as_i64()).unwrap_or(-1);
                let status_i32 = status_val as i32;
                let room_name = room_info.get("title").and_then(|s| s.as_str()).unwrap_or("").to_string();
                let avatar_url = room_info
                    .get("owner")
                    .and_then(|o| o.get("avatar_thumb"))
                    .and_then(|at| at.get("url_list"))
                    .and_then(|ul| ul.get(0))
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                let nickname = room_info
                    .get("owner")
                    .and_then(|o| o.get("nickname"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                (nickname, room_name, avatar_url, status_i32)
            } else { (String::new(), String::new(), String::new(), -1) }
        } else { (String::new(), String::new(), String::new(), -1) }
    };

    Ok(DouyinFollowListRoomInfo {
        room_id_str,
        nickname,
        room_name,
        avatar_url,
        status: status_i32,
    })
}