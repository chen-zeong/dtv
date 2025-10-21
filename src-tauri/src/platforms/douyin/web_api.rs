use crate::platforms::common::http_client::HttpClient;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, COOKIE, REFERER, USER_AGENT};
use serde_json::{self, Map, Value};

const DEFAULT_COOKIE: &str = "ttwid=1%7CB1qls3GdnZhUov9o2NxOMxxYS2ff6OSvEWbv0ytbES4%7C1680522049%7C280d802d6d478e3e78d0c807f7c487e7ffec0ae4e5fdd6a0fe74c3c6af149511";
const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:115.0) Gecko/20100101 Firefox/115.0";

#[derive(Debug, Clone)]
pub struct DouyinRoomData {
    pub room: Value,
}

fn build_common_headers(cookies: Option<&str>) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("zh-CN,zh;q=0.8"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://live.douyin.com/"),
    );
    let cookie_val = cookies.unwrap_or(DEFAULT_COOKIE);
    headers.insert(
        COOKIE,
        HeaderValue::from_str(cookie_val)
            .map_err(|e| format!("Invalid cookie header value: {}", e))?,
    );
    Ok(headers)
}

fn parse_state_json(html: &str) -> Result<Value, String> {
    let re_primary = Regex::new(r#"(\{\\"state\\":.*?)]\\n"]\)"#)
        .map_err(|e| format!("Failed to compile state regex: {}", e))?;
    let re_fallback = Regex::new(r#"(\{\\"common\\":.*?)]\\n"]\)</script><div hidden"#)
        .map_err(|e| format!("Failed to compile fallback regex: {}", e))?;

    let matched = re_primary
        .captures(html)
        .or_else(|| re_fallback.captures(html))
        .ok_or_else(|| "Cannot locate roomStore JSON".to_string())?;

    let raw = matched
        .get(1)
        .ok_or_else(|| "State capture missing group".to_string())?
        .as_str();

    let candidates = [
        raw.replace("\\\"", "\"")
            .replace("\\\\", "\\")
            .replace("]\\n", "")
            .replace("u0026", "&"),
        raw.replace("\\", "").replace("u0026", "&"),
    ];

    let mut last_err = None;
    for cleaned in candidates {
        match serde_json::from_str::<Value>(&cleaned) {
            Ok(data) => return Ok(data),
            Err(err) => last_err = Some(err),
        }
    }

    if let Some(fallback) = build_state_from_room_store(&raw) {
        return Ok(fallback);
    }

    Err(last_err
        .map(|e| format!("Failed to parse state JSON: {}", e))
        .unwrap_or_else(|| "Failed to parse state JSON: unknown error".to_string()))
}

fn build_state_from_room_store(raw: &str) -> Option<Value> {
    let simple = raw.replace("\\", "").replace("u0026", "&");

    // Room store substring sits before linkmicStore, same as python extraction.
    let room_store_re = Regex::new(r#""roomStore":(.*?),"linkmicStore""#).ok()?;
    let room_store_caps = room_store_re.captures(&simple)?;
    let room_store_raw = room_store_caps.get(1)?.as_str();

    let anchor_name = Regex::new(r#""nickname":"(.*?)","avatar_thumb""#)
        .ok()
        .and_then(|re| {
            re.captures(room_store_raw)
                .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        });

    let trimmed = if let Some(idx) = room_store_raw.find(r#","has_commerce_goods""#) {
        let mut prefix = room_store_raw[..idx].to_string();
        prefix.push_str("}}}");
        prefix
    } else {
        room_store_raw.to_string()
    };

    let mut room_store_value: Value = serde_json::from_str(&trimmed).ok()?;

    if let Some(name) = anchor_name {
        if let Some(room_info) = room_store_value.get_mut("roomInfo") {
            if let Some(room) = room_info.get_mut("room") {
                if let Some(obj) = room.as_object_mut() {
                    obj.entry("anchor_name".to_string())
                        .or_insert(Value::String(name));
                }
            }
        }
    }

    let mut state_map = Map::new();
    state_map.insert("roomStore".to_string(), room_store_value);

    let mut root_map = Map::new();
    root_map.insert("state".to_string(), Value::Object(state_map));
    Some(Value::Object(root_map))
}

fn extract_room_from_state(mut state_data: Value) -> Result<Value, String> {
    let state_obj = state_data
        .get_mut("state")
        .ok_or_else(|| "Missing state key".to_string())?;
    let room_store = state_obj
        .get_mut("roomStore")
        .ok_or_else(|| "Missing roomStore in state".to_string())?;
    let room_info = room_store
        .get_mut("roomInfo")
        .ok_or_else(|| "Missing roomInfo in roomStore".to_string())?;
    let room = room_info
        .get_mut("room")
        .cloned()
        .ok_or_else(|| "Missing room data in roomInfo".to_string())?;

    let anchor_name = room_info
        .get("anchor")
        .and_then(|v| v.get("nickname"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut room_mut = room;
    if let Some(name) = anchor_name {
        if let Some(obj) = room_mut.as_object_mut() {
            obj.insert("anchor_name".to_string(), Value::String(name));
        }
    }
    Ok(room_mut)
}

fn augment_origin_streams(room: &mut Value, html: &str) -> Result<(), String> {
    let stream_orientation = room
        .get("stream_url")
        .and_then(|v| v.get("stream_orientation"))
        .and_then(|v| v.as_i64())
        .unwrap_or_default();

    let script_re = Regex::new(r#""(\{\\"common\\":.*?)"]\)</script><script nonce="#)
        .map_err(|e| format!("Failed to compile origin regex: {}", e))?;

    let candidates: Vec<String> = script_re
        .captures_iter(html)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .collect();

    let mut origin: Option<Value> = None;
    if !candidates.is_empty() {
        let idx = if stream_orientation == 1 { 0 } else { 1 };
        let candidate = candidates
            .get(idx as usize)
            .or_else(|| candidates.first())
            .cloned();
        if let Some(c) = candidate {
            let s = c
                .replace("\\", "")
                .replace("\"{", "{")
                .replace("}\"", "}")
                .replace("u0026", "&");
            if let Ok(value) = serde_json::from_str::<Value>(&s) {
                origin = value
                    .get("data")
                    .and_then(|d| d.get("origin"))
                    .and_then(|o| o.get("main"))
                    .cloned();
            }
        }
    }

    if origin.is_none() {
        let fallback = html.replace("\\", "").replace("u0026", "&");
        let re = Regex::new(r#""origin":\{"main":(.*?),"dash""#)
            .map_err(|e| format!("Failed to compile origin fallback regex: {}", e))?;
        if let Some(caps) = re.captures(&fallback) {
            if let Some(m) = caps.get(1) {
                let json_str = format!("{}{}", m.as_str(), "}");
                if let Ok(value) = serde_json::from_str::<Value>(&json_str) {
                    origin = Some(value);
                }
            }
        }
    }

    if let Some(origin_val) = origin {
        let codec = origin_val
            .get("sdk_params")
            .and_then(|p| p.get("VCodec"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let origin_hls = origin_val
            .get("hls")
            .and_then(|v| v.as_str())
            .map(|s| format!("{}&codec={}", s, codec));
        let origin_flv = origin_val
            .get("flv")
            .and_then(|v| v.as_str())
            .map(|s| format!("{}&codec={}", s, codec));

        if let Some(stream_url) = room.get_mut("stream_url") {
            if let Some(hls_origin) = origin_hls {
                match stream_url.get_mut("hls_pull_url_map") {
                    Some(map_val) if map_val.is_object() => {
                        if let Some(map) = map_val.as_object_mut() {
                            let existing = map.clone();
                            map.clear();
                            map.insert("ORIGIN".to_string(), Value::String(hls_origin.clone()));
                            map.extend(existing);
                        }
                    }
                    _ => {
                        let mut new_map = serde_json::Map::new();
                        new_map.insert("ORIGIN".to_string(), Value::String(hls_origin));
                        stream_url.as_object_mut().map(|obj| {
                            obj.insert("hls_pull_url_map".to_string(), Value::Object(new_map))
                        });
                    }
                }
            }

            if let Some(flv_origin) = origin_flv {
                match stream_url.get_mut("flv_pull_url") {
                    Some(map_val) if map_val.is_object() => {
                        if let Some(map) = map_val.as_object_mut() {
                            let existing = map.clone();
                            map.clear();
                            map.insert("ORIGIN".to_string(), Value::String(flv_origin.clone()));
                            map.extend(existing);
                        }
                    }
                    _ => {
                        let mut new_map = serde_json::Map::new();
                        new_map.insert("ORIGIN".to_string(), Value::String(flv_origin));
                        stream_url.as_object_mut().map(|obj| {
                            obj.insert("flv_pull_url".to_string(), Value::Object(new_map))
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

async fn fetch_room_from_html(
    http_client: &HttpClient,
    web_id: &str,
    cookies: Option<&str>,
) -> Result<DouyinRoomData, String> {
    let url = if web_id.starts_with("http") {
        web_id.to_string()
    } else {
        format!("https://live.douyin.com/{}", web_id)
    };
    let headers = build_common_headers(cookies)?;
    let html = http_client
        .get_text_with_headers(&url, Some(headers))
        .await
        .map_err(|e| format!("Failed to fetch Douyin room html: {}", e))?;

    let state_json = parse_state_json(&html)?;
    let mut room = extract_room_from_state(state_json)?;
    augment_origin_streams(&mut room, &html)?;
    Ok(DouyinRoomData { room })
}

async fn fetch_room_from_api(
    http_client: &HttpClient,
    web_id: &str,
    cookies: Option<&str>,
) -> Result<DouyinRoomData, String> {
    let headers = build_common_headers(cookies)?;
    let params = vec![
        ("aid", "6383"),
        ("app_name", "douyin_web"),
        ("live_id", "1"),
        ("device_platform", "web"),
        ("language", "zh-CN"),
        ("cookie_enabled", "true"),
        ("screen_width", "1920"),
        ("screen_height", "1080"),
        ("browser_language", "zh-CN"),
        ("browser_platform", "Win32"),
        ("browser_name", "Chrome"),
        ("browser_version", "120.0.0.0"),
        ("web_rid", web_id),
        ("msToken", ""),
        ("a_bogus", ""),
    ];
    let query = serde_urlencoded::to_string(&params)
        .map_err(|e| format!("Failed to encode Douyin enter params: {}", e))?;
    let api = format!("https://live.douyin.com/webcast/room/web/enter/?{}", query);
    let json: Value = http_client
        .get_json_with_headers(&api, Some(headers))
        .await
        .map_err(|e| format!("Failed to request Douyin web enter API: {}", e))?;

    let room = json
        .get("data")
        .and_then(|d| d.get("data"))
        .and_then(|arr| arr.get(0))
        .cloned()
        .ok_or_else(|| "Douyin web enter API did not return room data".to_string())?;

    let anchor_name = json
        .get("data")
        .and_then(|d| d.get("user"))
        .and_then(|u| u.get("nickname"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut room_mut = room;
    if let Some(name) = anchor_name {
        if let Some(obj) = room_mut.as_object_mut() {
            obj.insert("anchor_name".to_string(), Value::String(name));
        }
    }
    Ok(DouyinRoomData { room: room_mut })
}

fn extract_web_id(id_or_url: &str) -> &str {
    if let Some(pos) = id_or_url.find("live.douyin.com/") {
        let start = pos + "live.douyin.com/".len();
        let remainder = &id_or_url[start..];
        remainder
            .split(['?', '&', '/'])
            .find(|segment| !segment.is_empty())
            .unwrap_or(id_or_url)
    } else {
        id_or_url
    }
}

pub async fn fetch_room_data(
    http_client: &HttpClient,
    raw_id: &str,
    cookies: Option<&str>,
) -> Result<DouyinRoomData, String> {
    let web_id = extract_web_id(raw_id);
    match fetch_room_from_html(http_client, web_id, cookies).await {
        Ok(room) => Ok(room),
        Err(html_err) => {
            println!(
                "[DouyinWebApi] HTML parse failed for {} with error: {}. Falling back to API.",
                web_id, html_err
            );
            fetch_room_from_api(http_client, web_id, cookies).await
        }
    }
}

pub fn choose_flv_stream(room: &Value, desired_quality: &str) -> Option<(String, String)> {
    let flv_map = room
        .get("stream_url")
        .and_then(|v| v.get("flv_pull_url"))
        .and_then(|v| v.as_object())?;

    const QUALITY_ORDER: [&str; 6] = ["OD", "BD", "UHD", "HD", "SD", "LD"];

    let mut entries: Vec<(String, String)> = flv_map
        .iter()
        .filter_map(|(key, value)| value.as_str().map(|url| (key.clone(), url.to_string())))
        .collect();

    if entries.is_empty() {
        return None;
    }

    while entries.len() < QUALITY_ORDER.len() {
        if let Some(last) = entries.last().cloned() {
            entries.push(last);
        } else {
            break;
        }
    }

    let desired = desired_quality.trim().to_uppercase();
    let idx = QUALITY_ORDER
        .iter()
        .position(|q| q.eq_ignore_ascii_case(&desired))
        .unwrap_or(0);

    entries
        .get(idx)
        .cloned()
        .or_else(|| entries.last().cloned())
}
