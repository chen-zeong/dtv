use crate::platforms::common::FollowHttpClient;
use md5;
use md5::{Digest, Md5};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, REFERER, USER_AGENT};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::command;
use tauri::State;

// WBI mixin key mapping table (same as Python implementation)
const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

fn get_mixin_key(origin: &str) -> String {
    let mut out = String::new();
    for &idx in MIXIN_KEY_ENC_TAB.iter() {
        if let Some(ch) = origin.chars().nth(idx) {
            out.push(ch);
        }
    }
    out.chars().take(32).collect()
}

async fn get_wbi_keys(
    client: &reqwest::Client,
    headers: &HeaderMap,
) -> Result<(String, String), String> {
    let url = "https://api.bilibili.com/x/web-interface/nav";
    let resp = client
        .get(url)
        .headers(headers.clone())
        .send()
        .await
        .map_err(|e| format!("Failed to get WBI keys: {}", e))?;
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read WBI keys text: {}", e))?;
    let json: Value = serde_json::from_str(&text)
        .map_err(|e| format!("Failed to parse WBI keys JSON: {} | {}", e, text))?;
    let wbi_img = json["data"]["wbi_img"].clone();
    let img_url = wbi_img["img_url"].as_str().unwrap_or("");
    let sub_url = wbi_img["sub_url"].as_str().unwrap_or("");

    let img_key = if let Some(pos) = img_url.rfind('/') {
        let fname = &img_url[pos + 1..];
        fname.split('.').next().unwrap_or("").to_string()
    } else {
        String::new()
    };
    let sub_key = if let Some(pos) = sub_url.rfind('/') {
        let fname = &sub_url[pos + 1..];
        fname.split('.').next().unwrap_or("").to_string()
    } else {
        String::new()
    };

    if img_key.is_empty() || sub_key.is_empty() {
        return Err("WBI keys not found".to_string());
    }
    Ok((img_key, sub_key))
}

fn sanitize_value(value: &str) -> String {
    // Remove characters in the banned set: !'()*
    let banned: [char; 5] = ['!', '\'', '(', ')', '*'];
    value.chars().filter(|c| !banned.contains(c)).collect()
}

fn build_wbi_sign(room_id: &str, img_key: &str, sub_key: &str) -> (String, String) {
    let mixin_key = get_mixin_key(&format!("{}{}", img_key, sub_key));
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let wts = now.to_string();

    // Build parameter map with room_id and wts
    let mut qp: HashMap<String, String> = HashMap::new();
    qp.insert("room_id".to_string(), room_id.to_string());
    qp.insert("wts".to_string(), wts.clone());

    // Sort keys and sanitize values
    let mut keys: Vec<String> = qp.keys().cloned().collect();
    keys.sort();
    let mut query_parts: Vec<String> = Vec::new();
    for k in keys.iter() {
        let v = sanitize_value(qp.get(k).map(|s| s.as_str()).unwrap_or(""));
        query_parts.push(format!("{}={}", k, urlencoding::encode(&v)));
    }
    let query = query_parts.join("&");
    let mut hasher = Md5::new();
    hasher.update(format!("{}{}", query, mixin_key).as_bytes());
    let w_rid = format!("{:x}", hasher.finalize());
    (wts, w_rid)
}

#[command]
pub async fn fetch_bilibili_streamer_info(
    payload: crate::platforms::common::GetStreamUrlPayload,
    cookie: Option<String>,
    follow_http: State<'_, FollowHttpClient>,
) -> Result<crate::platforms::common::LiveStreamInfo, String> {
    let room_id = payload.args.room_id_str.clone();
    if room_id.trim().is_empty() {
        return Ok(crate::platforms::common::LiveStreamInfo {
            title: None,
            anchor_name: None,
            avatar: None,
            stream_url: None,
            status: None,
            error_message: Some("房间ID未提供".to_string()),
            upstream_url: None,
            available_streams: None,
            normalized_room_id: None,
            web_rid: None,
        });
    }

    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

    // Build headers (include optional cookie)
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(ua).unwrap());
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://live.bilibili.com/"),
    );
    if let Some(c) = cookie.as_ref() {
        if !c.is_empty() {
            headers.insert(
                COOKIE,
                HeaderValue::from_str(c).unwrap_or(HeaderValue::from_static("")),
            );
        }
    }

    let client = &follow_http.0.inner;

    // Get WBI keys and build sign
    let (img_key, sub_key) = get_wbi_keys(client, &headers).await?;
    let (wts, w_rid) = build_wbi_sign(&room_id, &img_key, &sub_key);

    // Call getInfoByRoom API with signed params
    let base = "https://api.live.bilibili.com/xlive/web-room/v1/index/getInfoByRoom";
    let params = vec![("room_id", room_id.clone()), ("wts", wts), ("w_rid", w_rid)];

    let resp = client
        .get(base)
        .headers(headers.clone())
        .query(&params)
        .send()
        .await
        .map_err(|e| format!("Room info request failed: {}", e))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Read text failed: {}", e))?;
    if !status.is_success() {
        return Ok(crate::platforms::common::LiveStreamInfo {
            title: None,
            anchor_name: None,
            avatar: None,
            stream_url: None,
            status: None,
            error_message: Some(format!("Room info status: {} body: {}", status, text)),
            upstream_url: None,
            available_streams: None,
            normalized_room_id: None,
            web_rid: None,
        });
    }
    let j: Value = serde_json::from_str(&text)
        .map_err(|e| format!("Room info JSON parse failed: {} | body: {}", e, text))?;
    let data = j["data"].clone();

    let base_info = data["anchor_info"]["base_info"].clone();
    let room_info = data["room_info"].clone();

    let title = room_info["title"].as_str().map(|s| s.to_string());
    let anchor_name = base_info["uname"].as_str().map(|s| s.to_string());
    let avatar = base_info["face"].as_str().map(|s| s.to_string());
    let live_status = room_info["live_status"].as_i64().unwrap_or(0) as i32;

    Ok(crate::platforms::common::LiveStreamInfo {
        title,
        anchor_name,
        avatar,
        stream_url: None,
        status: Some(live_status),
        error_message: None,
        upstream_url: None,
        available_streams: None,
        normalized_room_id: None,
        web_rid: None,
    })
}
