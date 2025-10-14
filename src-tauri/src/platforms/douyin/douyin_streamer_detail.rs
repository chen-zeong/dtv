use crate::platforms::common::http_client::HttpClient;
use crate::platforms::common::LiveStreamInfo as CommonLiveStreamInfo;
use crate::platforms::common::GetStreamUrlPayload;
use crate::proxy::{start_proxy, ProxyServerHandle};
use crate::StreamUrlStore;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, COOKIE, REFERER, USER_AGENT};
use serde_json::Value;
use tauri::{command, AppHandle, State};

#[derive(Debug, Clone)
]
#[allow(dead_code)]
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

    println!("[Douyin Stream Detail] 请求获取直播流: room_id_str='{}', 画质='{}'", room_id_str, quality);
    if room_id_str.is_empty() {
        let result = CommonLiveStreamInfo {
            title: None,
            anchor_name: None,
            avatar: None,
            stream_url: None,
            status: None,
            error_message: Some("Room ID cannot be empty.".to_string()),
            upstream_url: None,
            available_streams: None,
            normalized_room_id: None,
        };
        // 写入桌面文件
        // 已移除：写入桌面文件调用 write_douyin_return_to_desktop_simple(&result, &room_id_str, &quality, "N/A");
        return Ok(result);
    }

    // 直连 HTTP 客户端，绕过所有代理
    let mut http_client = HttpClient::new_direct_connection()
        .map_err(|e| format!("Failed to create direct connection HttpClient: {}", e))?;

    // 保证 ttwid 存在
    ensure_ttwid(&mut http_client).await.ok();
    println!("[Douyin Stream Detail] ensure_ttwid 完成，准备选择解析路径");

    let parse_path = "reflow(room_id)";
    println!(
        "[Douyin Stream Detail] 解析路径选择: {} -> {}",
        room_id_str,
        parse_path
    );

     // 封装统一写入并返回的闭包
     let write_and_ok = |res: CommonLiveStreamInfo| {
         // 已移除：写入桌面文件调用 write_douyin_return_to_desktop(&http_client, &room_id_str, &quality, parse_path, &res);
         Ok(res)
     };

    // 统一使用 room_id 的 reflow info 接口
    let detail = match fetch_room_detail_by_room_id(&http_client, &room_id_str).await {
        Ok(json) => extract_detail_from_reflow(&json)
            .ok_or_else(|| "未能从 reflow info 中解析到房间详情".to_string())?,
        Err(e) => {
            let result = CommonLiveStreamInfo {
                title: None,
                anchor_name: None,
                avatar: None,
                stream_url: None,
                status: None,
                error_message: Some(format!("Reflow 接口请求失败: {}", e)),
                upstream_url: None,
                available_streams: None,
                normalized_room_id: None,
            };
            return write_and_ok(result);
        }
    };

    // 不在线直接返回基础信息，stream_url 为空
    if detail.status != 2 {
        let result = CommonLiveStreamInfo {
            title: detail.title,
            anchor_name: detail.owner_nickname,
            avatar: detail.avatar,
            stream_url: None,
            status: Some(detail.status),
            error_message: None,
            upstream_url: None,
            available_streams: None,
            normalized_room_id: detail.room_id.clone(),
        };
        return write_and_ok(result);
    }

    let stream_url_val = match detail.stream_url.clone() {
        Some(v) => v,
        None => {
            let result = CommonLiveStreamInfo {
                title: detail.title,
                anchor_name: detail.owner_nickname,
                avatar: detail.avatar,
                stream_url: None,
                status: Some(detail.status),
                error_message: Some("主播在线，但未找到 stream_url".to_string()),
                upstream_url: None,
                available_streams: None,
                normalized_room_id: detail.room_id.clone(),
            };
            return write_and_ok(result);
        }
    };

    let has_live_core = stream_url_val.get("live_core_sdk_data").is_some();
    let has_flv_map = stream_url_val.get("flv_pull_url").is_some();
    let has_hls_map = stream_url_val.get("hls_pull_url_map").is_some();
    println!(
        "[Douyin Stream Detail] stream_url 字段存在: live_core_sdk_data={}, flv_pull_url={}, hls_pull_url_map={}, 目标画质='{}'",
        has_live_core, has_flv_map, has_hls_map, quality
    );
    let mut upstream_url: Option<String> = None;
    if upstream_url.is_none() {
        if let Some(flv_map) = stream_url_val.get("flv_pull_url").and_then(|v| v.as_object()) {
            let desired_name = match quality.as_str() { "原画" => "origin", "高清" => "hd", "标清" => "sd", _ => "origin" };
            let mut chosen: Option<String> = None;
            let mut chosen_key: Option<String> = None;
            println!("[Douyin Stream Detail] 回退解析 flv_pull_url，目标画质='{}'", desired_name);
            for (k, v) in flv_map.iter() {
                if let Some(url) = v.as_str() {
                    let key_lower = k.to_ascii_lowercase();
                    if (desired_name == "origin" && (key_lower.contains("origin") || key_lower.contains("full_hd"))) ||
                       (desired_name == "hd" && key_lower.contains("hd")) ||
                       (desired_name == "sd" && (key_lower.contains("sd") || key_lower.contains("ld"))) {
                        chosen = Some(url.to_string());
                        chosen_key = Some(k.clone());
                        break;
                    }
                }
            }
            if chosen.is_none() {
                if let Some((k, vv)) = flv_map.iter().find(|(_, vv)| vv.as_str().is_some()) {
                    chosen = vv.as_str().map(|s| s.to_string());
                    chosen_key = Some(k.clone());
                }
            }
            if let Some(c) = chosen {
                println!("[Douyin Stream Detail] 从 flv_pull_url 选取 key='{}' -> {}", chosen_key.unwrap_or("<unknown>".to_string()), c);
                upstream_url = Some(c);
            } else {
                eprintln!("[Douyin Stream Detail] 未能从 flv_pull_url 选取到地址");
            }
        }

        if upstream_url.is_none() {
            if let Some(hls_map) = stream_url_val.get("hls_pull_url_map").and_then(|v| v.as_object()) {
                let desired_name = match quality.as_str() { "原画" => "origin", "高清" => "hd", "标清" => "sd", _ => "origin" };
                let mut chosen: Option<String> = None;
                let mut chosen_key: Option<String> = None;
                println!("[Douyin Stream Detail] 回退解析 hls_pull_url_map，目标画质='{}'", desired_name);
                for (k, v) in hls_map.iter() {
                    if let Some(url) = v.as_str() {
                        let key_lower = k.to_ascii_lowercase();
                        if (desired_name == "origin" && (key_lower.contains("origin") || key_lower.contains("full_hd"))) ||
                           (desired_name == "hd" && key_lower.contains("hd")) ||
                           (desired_name == "sd" && (key_lower.contains("sd") || key_lower.contains("ld"))) {
                            chosen = Some(url.to_string());
                            chosen_key = Some(k.clone());
                            break;
                        }
                    }
                }
                if chosen.is_none() {
                    if let Some((k, vv)) = hls_map.iter().find(|(_, vv)| vv.as_str().is_some()) {
                        chosen = vv.as_str().map(|s| s.to_string());
                        chosen_key = Some(k.clone());
                    }
                }
                if let Some(c) = chosen {
                    println!("[Douyin Stream Detail] 从 hls_pull_url_map 选取 key='{}' -> {}", chosen_key.unwrap_or("<unknown>".to_string()), c);
                    upstream_url = Some(c);
                } else {
                    eprintln!("[Douyin Stream Detail] 未能从 hls_pull_url_map 选取到地址");
                }
            }
        }
    }

    if let Some(real_url) = upstream_url {
        println!("[Douyin Stream Detail] 最终解析得到上游地址: {}", real_url);
        {
            let mut guard = stream_url_store.url.lock().unwrap();
            *guard = real_url.clone();
        }
        println!("[Douyin Stream Detail] 已写入 StreamUrlStore，准备启动本地代理");
        let proxied_url = match start_proxy(app_handle, proxy_server_handle, stream_url_store).await {
            Ok(proxy) => {
                println!("[Douyin Stream Detail] 代理启动成功，返回代理地址: {}", proxy);
                proxy
            },
            Err(e) => {
                eprintln!("[Douyin Stream Detail] 代理启动失败: {}，将返回真实地址", e);
                return Ok(CommonLiveStreamInfo {
                    title: detail.title,
                    anchor_name: detail.owner_nickname,
                    avatar: detail.avatar,
                    stream_url: Some(real_url),
                    status: Some(detail.status),
                    error_message: Some(format!("代理启动失败: {}", e)),
                    upstream_url: None,
                    available_streams: None,
                    normalized_room_id: detail.room_id.clone(),
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
            upstream_url: Some(real_url),
            available_streams: None,
            normalized_room_id: detail.room_id.clone(),
        })
    } else {
        eprintln!("[Douyin Stream Detail] 未能解析到任何可用的播放地址");
        Ok(CommonLiveStreamInfo {
            title: detail.title,
            anchor_name: detail.owner_nickname,
            avatar: detail.avatar,
            stream_url: None,
            status: Some(detail.status),
            error_message: Some("未能解析到任何可用的播放地址".to_string()),
            upstream_url: None,
            available_streams: None,
            normalized_room_id: detail.room_id.clone(),
        })
    }
}

async fn ensure_ttwid(http_client: &mut HttpClient) -> Result<(), String> {
    let live_url = "https://live.douyin.com/";
    println!("[Douyin Stream Detail] ensure_ttwid: 请求 {} 以获取 ttwid", live_url);
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
        println!("[Douyin Stream Detail] ensure_ttwid: 已设置 ttwid 到 Cookie 头");
    } else {
        println!("[Douyin Stream Detail] ensure_ttwid: 未获取到 ttwid，继续请求流程");
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

fn extract_detail_from_reflow(json: &Value) -> Option<DetailInfo> {
    let room = json.get("data")?.get("room")?;
    let owner = room.get("owner").cloned().unwrap_or(Value::Null);
    let status = room.get("status")?.as_i64()? as i32;
    let web_rid = owner.get("web_rid").and_then(|v| v.as_str()).map(|s| s.to_string());
    let room_id = room.get("id_str").and_then(|v| v.as_str()).map(|s| s.to_string());
    let title = room.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
    let _cover = room.get("cover").and_then(|c| c.get("url_list")).and_then(|ul| ul.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string());
    let owner_nickname = owner.get("nickname").and_then(|v| v.as_str()).map(|s| s.to_string());
    let avatar = owner.get("avatar_thumb").and_then(|a| a.get("url_list")).and_then(|ul| ul.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string());
    let stream_url = room.get("stream_url").cloned();

    println!(
        "[Douyin Stream Detail] extract_detail_from_reflow: status={}, web_rid={:?}, room_id={:?}, title={:?}, owner_nickname={:?}, stream_url_present={}",
        status, web_rid, room_id, title, owner_nickname, stream_url.is_some()
    );

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

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct QualityEntry {
    quality: String,
    sort: i32,
    urls: Vec<String>,
}

struct DouyinSitePyDefaults;
impl DouyinSitePyDefaults {
    const REFERER: &'static str = "https://live.douyin.com";
    fn ua() -> &'static str {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 Edg/125.0.0.0"
    }
}
