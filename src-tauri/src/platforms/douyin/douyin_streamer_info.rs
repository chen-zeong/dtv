use crate::platforms::common::http_client::HttpClient;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, COOKIE, REFERER, USER_AGENT};
use serde_json::Value;
use regex::Regex;
use tauri::command;



#[command]
pub async fn fetch_douyin_streamer_info(
    payload: crate::platforms::common::GetStreamUrlPayload, // Using the same payload structure for simplicity
) -> Result<crate::platforms::common::LiveStreamInfo, String> {
    let room_id_str = payload.args.room_id_str;
    println!("[Douyin Info RS] Received room_id_str: '{}'", room_id_str);

    if room_id_str.is_empty() {
        return Ok(crate::platforms::common::LiveStreamInfo {
            title: None,
            anchor_name: None,
            avatar: None,
            stream_url: None,
            status: None,
            error_message: Some("Room ID cannot be empty.".to_string()),
            upstream_url: None,
            available_streams: None,
            normalized_room_id: None,
        });
    }

    // 直连 HTTP 客户端，绕过所有代理（与 detail 保持一致）
    let mut http_client = HttpClient::new_direct_connection()
        .map_err(|e| format!("Failed to create direct connection HttpClient: {}", e))?;

    // 保证 ttwid 存在（与 detail 保持一致）
    ensure_ttwid(&mut http_client).await.ok();

    // 根据长度判断是 webRid 还是 roomId（与 detail 一致）
    if room_id_str.len() <= 16 {
        // HTML 解析路径：直接从 state 中返回主播信息，不再调用 reflow
        match fetch_room_detail_by_web_rid_html(&http_client, &room_id_str).await {
            Ok(state_json) => {
                // 提取所需字段
                let room_info = state_json.get("roomStore").and_then(|v| v.get("roomInfo"));
                let (room, anchor) = match room_info {
                    Some(ri) => (ri.get("room"), ri.get("anchor")),
                    None => (None, None),
                };
                let room = match room { Some(r) => r, None => {
                    return Ok(crate::platforms::common::LiveStreamInfo {
                        title: None,
                        anchor_name: None,
                        avatar: None,
                        stream_url: None,
                        status: None,
                        error_message: Some("未能从 HTML state 中解析到房间详情".to_string()),
                        upstream_url: None,
                        available_streams: None,
                        normalized_room_id: None,
                    });
                }};
                println!("room: {:?}", room);
                let status = room.get("status").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let title = room.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
                let owner = room.get("owner");
                let anchor_name = if status == 2 {
                    owner.and_then(|o| o.get("nickname")).and_then(|v| v.as_str()).map(|s| s.to_string())
                } else {
                    anchor.and_then(|a| a.get("nickname")).and_then(|v| v.as_str()).map(|s| s.to_string())
                };
                let avatar = if status == 2 {
                    owner.and_then(|o| o.get("avatar_thumb")).and_then(|a| a.get("url_list")).and_then(|ul| ul.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string())
                } else {
                    anchor.and_then(|a| a.get("avatar_thumb")).and_then(|a| a.get("url_list")).and_then(|ul| ul.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string())
                };
                // 规范化后的房间ID：从 HTML state 中提取 room.id_str
                let normalized_room_id = room.get("id_str").and_then(|v| v.as_str()).map(|s| s.to_string());

                Ok(crate::platforms::common::LiveStreamInfo {
                    title,
                    anchor_name,
                    avatar,
                    stream_url: None,
                    status: Some(status),
                    error_message: None,
                    upstream_url: None,
                    available_streams: None,
                    normalized_room_id,
                })
            }
            Err(e) => {
                return Ok(crate::platforms::common::LiveStreamInfo {
                    title: None,
                    anchor_name: None,
                    avatar: None,
                    stream_url: None,
                    status: None,
                    error_message: Some(format!("HTML 解析失败: {}", e)),
                    upstream_url: None,
                    available_streams: None,
                    normalized_room_id: None,
                });
            }
        }
    } else {
        // 通过 reflow info 接口（与 detail 保持一致的 Cookie/Headers/URL 参数）
        match fetch_room_detail_by_room_id(&http_client, &room_id_str).await {
            Ok(json) => {
                let room = match json.get("data").and_then(|d| d.get("room")) {
                    Some(r) => r,
                    None => {
                        return Ok(crate::platforms::common::LiveStreamInfo {
                            title: None,
                            anchor_name: None,
                            avatar: None,
                            stream_url: None,
                            status: None,
                            error_message: Some("未能从 reflow info 中解析到房间详情".to_string()),
                            upstream_url: None,
                            available_streams: None,
                            normalized_room_id: None,
                        });
                    }
                };
                let owner = room.get("owner").cloned().unwrap_or(Value::Null);
                let status = room.get("status").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let title = room.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
                let anchor_name = owner.get("nickname").and_then(|v| v.as_str()).map(|s| s.to_string());
                let avatar = owner
                    .get("avatar_thumb")
                    .and_then(|a| a.get("url_list"))
                    .and_then(|ul| ul.get(0))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                Ok(crate::platforms::common::LiveStreamInfo {
                    title,
                    anchor_name,
                    avatar,
                    stream_url: None,
                    status: Some(status),
                    error_message: None,
                    upstream_url: None,
                    available_streams: None,
                    normalized_room_id: Some(room_id_str.clone()),
                })
            }
            Err(e) => {
                return Ok(crate::platforms::common::LiveStreamInfo {
                    title: None,
                    anchor_name: None,
                    avatar: None,
                    stream_url: None,
                    status: None,
                    error_message: Some(format!("Reflow 接口请求失败: {}", e)),
                    upstream_url: None,
                    available_streams: None,
                    normalized_room_id: None,
                });
            }
        }
    }
}

// 与 douyin_streamer_detail.rs 保持一致的 ttwid 获取逻辑
async fn ensure_ttwid(http_client: &mut HttpClient) -> Result<(), String> {
    let live_url = "https://live.douyin.com/";
    let response = http_client
        .get_with_cookies(live_url)
        .await
        .map_err(|e| format!("获取 {} 响应失败: {}", live_url, e))?;

    if let Some(ttwid_cookie) = response
        .cookies()
        .find(|c| c.name() == "ttwid")
        .map(|c| c.value().to_string())
    {
        let cookie_header_val = format!("ttwid={};", ttwid_cookie);
        http_client
            .insert_header(COOKIE, &cookie_header_val)
            .map_err(|e| format!("设置 ttwid cookie 失败: {}", e))?;
    }
    Ok(())
}

// 与 douyin_streamer_detail.rs 中 fetch_room_detail_by_room_id 相同的请求结构（headers/cookies）
async fn fetch_room_detail_by_room_id(http_client: &HttpClient, room_id: &str) -> Result<Value, String> {
    let url = "https://webcast.amemv.com/webcast/room/reflow/info/";
    let params = vec![
        ("type_id", "0"),
        ("live_id", "1"),
        ("room_id", room_id),
        ("sec_user_id", ""),
        ("version_code", "99.99.99"),
        ("app_id", "6383"),
    ];
    let mut query = String::new();
    for (i, (k, v)) in params.iter().enumerate() {
        if i > 0 { query.push('&'); }
        query.push_str(&format!("{}={}", k, v));
    }
    let full_url = format!("{}?{}", url, query);

    let mut headers = HeaderMap::new();
    headers.insert(REFERER, HeaderValue::from_static(DouyinSitePyDefaults::REFERER));
    headers.insert(USER_AGENT, HeaderValue::from_static(DouyinSitePyDefaults::ua()));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("zh-CN,zh;q=0.9"));

    http_client
        .get_json_with_headers(&full_url, Some(headers))
        .await
        .map_err(|e| format!("请求 reflow info 失败: {}", e))
}

// 与 douyin_streamer_detail.rs 中 fetch_room_detail_by_web_rid_html 相同的请求结构（headers）
async fn fetch_room_detail_by_web_rid_html(http_client: &HttpClient, web_rid: &str) -> Result<Value, String> {
    let room_url = format!("https://live.douyin.com/{}", web_rid);
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(DouyinSitePyDefaults::ua()).unwrap());
    headers.insert(REFERER, HeaderValue::from_static(DouyinSitePyDefaults::REFERER));

    let text = http_client
        .get_text_with_headers(&room_url, Some(headers))
        .await
        .map_err(|e| format!("获取房间页面失败: {}", e))?;

    let re = Regex::new(r#"\{\\\"state\\\":\{\\\"appStore.*?\]\\n"#)
        .map_err(|e| format!("构建正则失败: {}", e))?;
    let m = re
        .find(&text)
        .ok_or_else(|| "未能在 HTML 中解析到 Douyin state 数据".to_string())?;
    let raw = m.as_str().trim();
    let s = raw.replace("\\\"", "\"").replace("\\\\", "\\").replace("]\\n", "");
    let data: Value = serde_json::from_str(&s).map_err(|e| format!("解析 state JSON 失败: {}", e))?;
    Ok(data["state"].clone())
}

// 与 detail.rs 保持一致的 UA/Referer
struct DouyinSitePyDefaults;
impl DouyinSitePyDefaults {
    const REFERER: &'static str = "https://live.douyin.com";
    fn ua() -> &'static str {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 Edg/125.0.0.0"
    }
}
