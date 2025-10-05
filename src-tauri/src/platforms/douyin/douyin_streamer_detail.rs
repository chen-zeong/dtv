use crate::platforms::common::http_client::HttpClient;
use crate::platforms::common::LiveStreamInfo as CommonLiveStreamInfo;
use crate::platforms::common::GetStreamUrlPayload;
use crate::proxy::{start_proxy, ProxyServerHandle};
use crate::StreamUrlStore;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, COOKIE, REFERER, USER_AGENT};
use serde_json::Value;
use tauri::{command, AppHandle, State};
use regex::Regex;

#[derive(Debug, Clone)]
struct DetailInfo {
    web_rid: Option<String>,
    room_id: Option<String>,
    status: i32,
    title: Option<String>,
    owner_nickname: Option<String>,
    avatar: Option<String>,
    stream_url: Option<Value>,
}

#[command]
pub async fn get_douyin_live_stream_url(
    app_handle: AppHandle,
    stream_url_store: State<'_, StreamUrlStore>,
    proxy_server_handle: State<'_, ProxyServerHandle>,
    payload: GetStreamUrlPayload,
) -> Result<CommonLiveStreamInfo, String> {
    get_douyin_live_stream_url_with_quality(
        app_handle,
        stream_url_store,
        proxy_server_handle,
        payload,
        "原画".to_string(),
    )
    .await
}

#[command]
pub async fn get_douyin_live_stream_url_with_quality(
    app_handle: AppHandle,
    stream_url_store: State<'_, StreamUrlStore>,
    proxy_server_handle: State<'_, ProxyServerHandle>,
    payload: GetStreamUrlPayload,
    quality: String,
) -> Result<CommonLiveStreamInfo, String> {
    let room_id_str = payload.args.room_id_str;

    if room_id_str.is_empty() {
        return Ok(CommonLiveStreamInfo {
            title: None,
            anchor_name: None,
            avatar: None,
            stream_url: None,
            status: None,
            error_message: Some("Room ID cannot be empty.".to_string()),
        });
    }

    // 直连 HTTP 客户端，绕过所有代理
    let mut http_client = HttpClient::new_direct_connection()
        .map_err(|e| format!("Failed to create direct connection HttpClient: {}", e))?;

    // 保证 ttwid 存在
    ensure_ttwid(&mut http_client).await.ok();

    // 根据长度判断是 webRid 还是 roomId（与 Python 一致）
    let detail = if room_id_str.len() <= 16 {
        // HTML 解析路径
        match fetch_room_detail_by_web_rid_html(&http_client, &room_id_str).await {
            Ok(state_json) => extract_detail_from_html_state(&room_id_str, &state_json)
                .ok_or_else(|| "未能从 HTML state 中解析到房间详情".to_string())?,
            Err(e) => {
                return Ok(CommonLiveStreamInfo {
                    title: None,
                    anchor_name: None,
                    avatar: None,
                    stream_url: None,
                    status: None,
                    error_message: Some(format!("HTML 解析失败: {}", e)),
                });
            }
        }
    } else {
        // 通过 reflow info 接口
        match fetch_room_detail_by_room_id(&http_client, &room_id_str).await {
            Ok(json) => extract_detail_from_reflow(&json)
                .ok_or_else(|| "未能从 reflow info 中解析到房间详情".to_string())?,
            Err(e) => {
                return Ok(CommonLiveStreamInfo {
                    title: None,
                    anchor_name: None,
                    avatar: None,
                    stream_url: None,
                    status: None,
                    error_message: Some(format!("Reflow 接口请求失败: {}", e)),
                });
            }
        }
    };

    // 不在线直接返回基础信息，stream_url 为空
    if detail.status != 2 {
        return Ok(CommonLiveStreamInfo {
            title: detail.title,
            anchor_name: detail.owner_nickname,
            avatar: detail.avatar,
            stream_url: None,
            status: Some(detail.status),
            error_message: None,
        });
    }

    let stream_url_val = match detail.stream_url.clone() {
        Some(v) => v,
        None => {
            return Ok(CommonLiveStreamInfo {
                title: detail.title,
                anchor_name: detail.owner_nickname,
                avatar: detail.avatar,
                stream_url: None,
                status: Some(detail.status),
                error_message: Some("主播在线，但未找到 stream_url".to_string()),
            })
        }
    };

    let qualities = parse_play_qualities(&stream_url_val);
    let urls = get_play_urls_by_quality(&qualities, if quality.is_empty() { None } else { Some(&quality) });

    // 优先选择 FLV
    let mut upstream_url: Option<String> = None;
    for u in &urls {
        if u.contains(".flv") || u.contains("pull-flv") {
            upstream_url = Some(u.clone());
            break;
        }
    }
    if upstream_url.is_none() {
        upstream_url = urls.get(0).cloned();
    }

    if let Some(real_url) = upstream_url {
        // 写入到 StreamUrlStore 以便代理获取
        {
            let mut guard = stream_url_store.url.lock().unwrap();
            *guard = real_url.clone();
        }

        // 启动/重启本地代理，返回代理地址
        let proxied_url = match start_proxy(app_handle, proxy_server_handle, stream_url_store).await {
            Ok(proxy) => proxy,
            Err(e) => {
                // 代理失败则直接返回真实地址，同时附带错误信息
                return Ok(CommonLiveStreamInfo {
                    title: detail.title,
                    anchor_name: detail.owner_nickname,
                    avatar: detail.avatar,
                    stream_url: Some(real_url),
                    status: Some(detail.status),
                    error_message: Some(format!("代理启动失败: {}", e)),
                });
            }
        };

        Ok(CommonLiveStreamInfo {
            title: detail.title,
            anchor_name: detail.owner_nickname,
            avatar: detail.avatar,
            stream_url: Some(proxied_url),
            status: Some(detail.status),
            error_message: None,
        })
    } else {
        Ok(CommonLiveStreamInfo {
            title: detail.title,
            anchor_name: detail.owner_nickname,
            avatar: detail.avatar,
            stream_url: None,
            status: Some(detail.status),
            error_message: Some("未能解析到任何可用的播放地址".to_string()),
        })
    }
}

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

async fn fetch_room_detail_by_room_id(http_client: &HttpClient, room_id: &str) -> Result<Value, String> {
    let url = "https://webcast.amemv.com/webcast/room/reflow/info/";
    // 构建查询参数
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

    // 参考 Python 的 UA/Referer，作为额外 headers 传入
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

async fn fetch_room_detail_by_web_rid_html(http_client: &HttpClient, web_rid: &str) -> Result<Value, String> {
    // 先 GET 房间页面获取必要 cookie
    let room_url = format!("https://live.douyin.com/{}", web_rid);
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(DouyinSitePyDefaults::ua()).unwrap());
    headers.insert(REFERER, HeaderValue::from_static(DouyinSitePyDefaults::REFERER));

    let text = http_client
        .get_text_with_headers(&room_url, Some(headers))
        .await
        .map_err(|e| format!("获取房间页面失败: {}", e))?;

    // 解析内嵌的 state JSON（与 Python 版逻辑一致）
    let re = Regex::new(r#"\{\\"state\\":\{\\"appStore.*?\]\\n"#)
        .map_err(|e| format!("构建正则失败: {}", e))?;
    let m = re
        .find(&text)
        .ok_or_else(|| "未能在 HTML 中解析到 Douyin state 数据".to_string())?;
    let raw = m.as_str().trim();
    let s = raw.replace("\\\"", "\"").replace("\\\\", "\\").replace("]\\n", "");
    let data: Value = serde_json::from_str(&s).map_err(|e| format!("解析 state JSON 失败: {}", e))?;
    Ok(data["state"].clone())
}

fn extract_detail_from_reflow(json: &Value) -> Option<DetailInfo> {
    let room = json.get("data")?.get("room")?;
    let owner = room.get("owner").cloned().unwrap_or(Value::Null);
    let status = room.get("status")?.as_i64()? as i32;
    let web_rid = owner.get("web_rid").and_then(|v| v.as_str()).map(|s| s.to_string());
    let room_id = room.get("id_str").and_then(|v| v.as_str()).map(|s| s.to_string()).or_else(|| room.get("id").and_then(|v| v.as_i64()).map(|n| n.to_string()));
    let title = room.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
    let cover = room.get("cover").and_then(|c| c.get("url_list")).and_then(|ul| ul.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string());
    let owner_nickname = owner.get("nickname").and_then(|v| v.as_str()).map(|s| s.to_string());
    let avatar = owner.get("avatar_thumb").and_then(|a| a.get("url_list")).and_then(|ul| ul.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string());
    let stream_url = room.get("stream_url").cloned();

    Some(DetailInfo {
        web_rid,
        room_id,
        status,
        title,
        owner_nickname,
        avatar,
        stream_url,
    })
}

fn extract_detail_from_html_state(web_rid: &str, state: &Value) -> Option<DetailInfo> {
    let room_info = state.get("roomStore")?.get("roomInfo")?;
    let room = room_info.get("room")?;
    let anchor = room_info.get("anchor").cloned().unwrap_or(Value::Null);
    let status = room.get("status")?.as_i64()? as i32;
    let title = room.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
    let cover = room.get("cover").and_then(|c| c.get("url_list")).and_then(|ul| ul.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string());
    let owner = room.get("owner").cloned().unwrap_or(Value::Null);
    let owner_nickname = if status == 2 {
        owner.get("nickname").and_then(|v| v.as_str()).map(|s| s.to_string())
    } else {
        anchor.get("nickname").and_then(|v| v.as_str()).map(|s| s.to_string())
    };
    let avatar = if status == 2 {
        owner.get("avatar_thumb").and_then(|a| a.get("url_list")).and_then(|ul| ul.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string())
    } else {
        anchor.get("avatar_thumb").and_then(|a| a.get("url_list")).and_then(|ul| ul.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string())
    };
    let room_id = room.get("id_str").and_then(|v| v.as_str()).map(|s| s.to_string());
    let stream_url = room.get("stream_url").cloned();

    Some(DetailInfo {
        web_rid: Some(web_rid.to_string()),
        room_id,
        status,
        title,
        owner_nickname,
        avatar,
        stream_url,
    })
}

#[derive(Debug, Clone)]
struct QualityEntry {
    quality: String,
    sort: i32,
    urls: Vec<String>,
}

fn parse_play_qualities(stream_url: &Value) -> Vec<QualityEntry> {
    let mut result: Vec<QualityEntry> = Vec::new();
    let pull_data = stream_url
        .get("live_core_sdk_data")
        .and_then(|v| v.get("pull_data"))
        .cloned()
        .unwrap_or(Value::Null);
    let qualities = pull_data
        .get("options")
        .and_then(|v| v.get("qualities"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let stream_data = pull_data.get("stream_data").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();

    if !stream_data.is_empty() && stream_data.starts_with('{') {
        // JSON 新格式
        if let Ok(sd_json) = serde_json::from_str::<Value>(&stream_data) {
            let quality_data = sd_json.get("data").cloned().unwrap_or(Value::Null);
            for q in &qualities {
                let name = q.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let level = q.get("level").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let sdk_key = q.get("sdk_key").and_then(|v| v.as_str());
                let mut urls: Vec<String> = Vec::new();
                if let Some(key) = sdk_key {
                    if let Some(node) = quality_data.get(key) {
                        let flv = node.get("main").and_then(|m| m.get("flv")).and_then(|v| v.as_str()).map(|s| s.to_string());
                        let hls = node.get("main").and_then(|m| m.get("hls")).and_then(|v| v.as_str()).map(|s| s.to_string());
                        if let Some(f) = flv { urls.push(f); }
                        if let Some(h) = hls { urls.push(h); }
                    }
                }
                if !urls.is_empty() {
                    result.push(QualityEntry { quality: name, sort: level, urls });
                }
            }
        }
    } else {
        // 旧格式：从 flv/hls map 逆序取
        let flv_map = stream_url.get("flv_pull_url").and_then(|v| v.as_object());
        let hls_map = stream_url.get("hls_pull_url_map").and_then(|v| v.as_object());
        let flv_list: Vec<String> = flv_map
            .map(|m| m.values().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let hls_list: Vec<String> = hls_map
            .map(|m| m.values().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        for q in &qualities {
            let name = q.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let level = q.get("level").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let mut urls: Vec<String> = Vec::new();
            let flv_index = if flv_list.is_empty() { -1 } else { (flv_list.len() as i32) - level };
            if flv_index >= 0 && (flv_index as usize) < flv_list.len() { urls.push(flv_list[flv_index as usize].clone()); }
            let hls_index = if hls_list.is_empty() { -1 } else { (hls_list.len() as i32) - level };
            if hls_index >= 0 && (hls_index as usize) < hls_list.len() { urls.push(hls_list[hls_index as usize].clone()); }
            if !urls.is_empty() {
                result.push(QualityEntry { quality: name, sort: level, urls });
            }
        }
    }

    // 按 sort 从高到低排序
    result.sort_by(|a, b| b.sort.cmp(&a.sort));
    result
}

fn get_play_urls_by_quality(qualities: &[QualityEntry], quality_name: Option<&str>) -> Vec<String> {
    if qualities.is_empty() { return Vec::new(); }
    if let Some(qn) = quality_name {
        for q in qualities {
            if q.quality == qn { return q.urls.clone(); }
        }
    }
    qualities[0].urls.clone()
}

struct DouyinSitePyDefaults;
impl DouyinSitePyDefaults {
    const REFERER: &'static str = "https://live.douyin.com";
    fn ua() -> &'static str {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 Edg/125.0.0.0"
    }
}