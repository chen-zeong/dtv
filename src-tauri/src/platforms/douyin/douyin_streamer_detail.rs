use crate::platforms::common::http_client::HttpClient;
use crate::platforms::common::LiveStreamInfo as CommonLiveStreamInfo;
use crate::platforms::common::GetStreamUrlPayload;
use crate::proxy::{start_proxy, ProxyServerHandle};
use crate::StreamUrlStore;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, COOKIE, REFERER, USER_AGENT};
use serde_json::Value;
use tauri::{command, AppHandle, State};
use regex::Regex;
use std::fs;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static COLLECTED_DY_COOKIE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

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
        };
        // 写入桌面文件
        write_douyin_return_to_desktop_simple(&result, &room_id_str, &quality, "N/A");
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
         write_douyin_return_to_desktop(&http_client, &room_id_str, &quality, parse_path, &res);
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

    // 新解析逻辑：优先从 live_core_sdk_data.pull_data.stream_data 中提取指定画质的 FLV，失败则回退到 flv/hls 映射
    let mut upstream_url: Option<String> = None;

    // 先尝试解析 live_core_sdk_data.pull_data.stream_data 的 JSON
    let stream_data_str = stream_url_val
        .get("live_core_sdk_data")
        .and_then(|v| v.get("pull_data"))
        .and_then(|v| v.get("stream_data"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if !stream_data_str.is_empty() && stream_data_str.starts_with('{') {
        println!("[Douyin Stream Detail] 检测到 stream_data JSON，优先尝试解析 main.flv");
        if let Ok(sd_json) = serde_json::from_str::<Value>(stream_data_str) {
            let data_node = sd_json.get("data").cloned().unwrap_or(Value::Null);
            let desired_key = match quality.as_str() { "原画" => "origin", "高清" => "hd", "标清" => "sd", _ => "origin" };
            let candidates = [desired_key, "origin", "hd", "sd"];
            for k in candidates.iter() {
                if let Some(main) = data_node.get(*k).and_then(|v| v.get("main")) {
                    if let Some(flv) = main.get("flv").and_then(|v| v.as_str()) {
                        if !flv.is_empty() {
                            println!("[Douyin Stream Detail] 从 stream_data.data.{}.main.flv 解析到 FLV: {}", k, flv);
                            upstream_url = Some(flv.to_string());
                            break;
                        }
                    } else {
                        println!("[Douyin Stream Detail] stream_data.data.{}.main 未找到 flv 字段", k);
                    }
                } else {
                    println!("[Douyin Stream Detail] stream_data.data 未包含 key='{}' 的 main 节点", k);
                }
            }
        } else {
            eprintln!("[Douyin Stream Detail] 解析 stream_data 失败，回退到其他字段");
        }
    } else {
        println!("[Douyin Stream Detail] 未检测到有效的 stream_data 字段，准备回退");
    }

    // 如果拿到的初始 FLV 不含 "pull-flv"，尝试解析重定向拿到最终地址
    if let Some(initial_flv_url_candidate) = upstream_url.clone() {
        if !initial_flv_url_candidate.contains("pull-flv") {
            println!("[Douyin Stream Detail] 候选 FLV 不包含 'pull-flv'，尝试探测重定向: {}", initial_flv_url_candidate);
            match reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build() {
                Ok(http_client_for_redirect) => {
                    match http_client_for_redirect.get(&initial_flv_url_candidate).send().await {
                        Ok(response) => {
                            println!("[Douyin Stream Detail] 重定向探测响应状态: {}", response.status());
                            if response.status().is_redirection() {
                                if let Some(location_header) = response.headers().get(reqwest::header::LOCATION) {
                                    if let Ok(redirected_url_str) = location_header.to_str() {
                                        if !redirected_url_str.is_empty() {
                                            println!("[Douyin Stream Detail] 发现重定向 Location: {}", redirected_url_str);
                                            upstream_url = Some(redirected_url_str.to_string());
                                        } else {
                                            println!("[Douyin Stream Detail] 重定向 Location 为空，保留原始地址");
                                        }
                                    }
                                } else {
                                    println!("[Douyin Stream Detail] 无 Location 头，保留原始地址");
                                }
                            } else {
                                println!("[Douyin Stream Detail] 非重定向状态，保留原始地址");
                            }
                        }
                        Err(e) => {
                            eprintln!("[Douyin Stream Detail] 重定向探测失败，保留原始地址，错误: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[Douyin Stream Detail] 构建重定向探测客户端失败，错误: {}", e);
                }
            }
        } else {
            println!("[Douyin Stream Detail] 候选地址已包含 'pull-flv'，无需重定向探测");
        }
    }

    // 如果仍未获得地址，回退到 flv/hls 映射
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

async fn fetch_room_detail_by_web_rid_html(http_client: &HttpClient, web_rid: &str) -> Result<Value, String> {
    let room_url = format!("https://live.douyin.com/{}", web_rid);
    let ua = DouyinSitePyDefaults::ua();

    // 先通过 HEAD/GET 收集必要 Cookie，避免 444 拦截
    let mut dy_cookie = String::new();
    let mut collected_cookie_names: Vec<String> = Vec::new();
    // HEAD 收集初始 cookie
    let head_resp = http_client
        .inner
        .head(&room_url)
        .header(USER_AGENT, ua)
        .header(REFERER, DouyinSitePyDefaults::REFERER)
        .header(reqwest::header::HeaderName::from_static("authority"), HeaderValue::from_static("live.douyin.com"))
        .send()
        .await
        .map_err(|e| format!("获取房间页面失败: {}", e))?;
    println!("[Douyin Stream Detail] HEAD 状态: {}", head_resp.status());
    for val in head_resp.headers().get_all("set-cookie").iter() {
        if let Ok(s) = val.to_str() {
            let first = s.split(';').next().unwrap_or("");
            let name = first.split('=').next().unwrap_or("").to_string();
            if first.contains("ttwid")
                || first.contains("__ac_nonce")
                || first.contains("msToken")
                || first.contains("s_v_web_id")
                || first.contains("tt_scid")
            {
                dy_cookie.push_str(first);
                dy_cookie.push(';');
                collected_cookie_names.push(name);
            }
        }
    }
    println!("[Douyin Stream Detail] 初始收集到 Cookie 名称: {:?}", collected_cookie_names);

    // 首次 GET 尝试获取页面与补充 cookie
    let get_resp = http_client
        .inner
        .get(&room_url)
        .header(USER_AGENT, ua)
        .header(REFERER, DouyinSitePyDefaults::REFERER)
        .send()
        .await
        .map_err(|e| format!("获取房间页面失败: {}", e))?;
    println!("[Douyin Stream Detail] 首次 GET 状态: {}", get_resp.status());

    for val in get_resp.headers().get_all("set-cookie").iter() {
        if let Ok(s) = val.to_str() {
            let first = s.split(';').next().unwrap_or("");
            let name = first.split('=').next().unwrap_or("").to_string();
            if first.contains("ttwid")
                || first.contains("__ac_nonce")
                || first.contains("msToken")
                || first.contains("s_v_web_id")
                || first.contains("tt_scid")
            {
                dy_cookie.push_str(first);
                dy_cookie.push(';');
                collected_cookie_names.push(name);
            }
        }
    }
    // 记录组装后的Cookie供写文件使用
    {
        let mut g = COLLECTED_DY_COOKIE.lock().unwrap();
        *g = Some(dy_cookie.clone());
    }
    println!("[Douyin Stream Detail] GET 收集到 Cookie 名称累计: {:?}", collected_cookie_names);

    // 如果首次 GET 失败或返回 444，再携带 Cookie 重试
    let mut text: Option<String> = None;
    if get_resp.status().is_success() {
        let body = get_resp
            .text()
            .await
            .map_err(|e| format!("读取房间页面响应失败: {}", e))?;
        if body.contains("Access Denied") {
            println!("[Douyin Stream Detail] 页面内容包含 'Access Denied'，将携带 Cookie 重试");
        } else {
            println!("[Douyin Stream Detail] 首次 GET 读取页面成功，长度={}，跳过重试", body.len());
            text = Some(body);
        }
    } else {
        println!("[Douyin Stream Detail] 首次 GET 非成功状态，准备重试");
    }

    if text.is_none() {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("zh-CN,zh;q=0.9"));
        headers.insert(REFERER, HeaderValue::from_static(DouyinSitePyDefaults::REFERER));
        headers.insert(USER_AGENT, HeaderValue::from_str(ua).unwrap_or_else(|_| HeaderValue::from_static(DouyinSitePyDefaults::REFERER)));
        headers.insert(reqwest::header::HeaderName::from_static("cookie"), HeaderValue::from_str(&dy_cookie).unwrap_or_else(|_| HeaderValue::from_static("")));

        // 使用现有客户端重试
        let resp2 = http_client
            .inner
            .get(&room_url)
            .headers(headers.clone())
            .send()
            .await
            .map_err(|e| format!("获取房间页面失败: {}", e))?;
        println!("[Douyin Stream Detail] 重试 GET 状态: {}", resp2.status());
        if resp2.status().is_success() {
            let body = resp2
                .text()
                .await
                .map_err(|e| format!("读取房间页面响应失败: {}", e))?;
            if !body.contains("Access Denied") {
                println!("[Douyin Stream Detail] 重试 GET 读取页面成功，长度={}", body.len());
                text = Some(body);
            } else {
                println!("[Douyin Stream Detail] 重试 GET 仍包含 'Access Denied'，准备直连客户端再试");
            }
        } else {
            println!("[Douyin Stream Detail] 重试 GET 非成功状态，准备直连客户端再试");
        }

        // 如果仍失败，使用直连客户端再试一次，规避代理拦截
        if text.is_none() {
            let direct = HttpClient::new_direct_connection().map_err(|e| format!("创建直连客户端失败: {}", e))?;
            println!("[Douyin Stream Detail] 直连客户端发起 GET，携带 Cookie 名称: {:?}", collected_cookie_names);
            let resp3 = direct
                .inner
                .get(&room_url)
                .headers(headers)
                .send()
                .await
                .map_err(|e| format!("获取房间页面失败: {}", e))?;
            let status_code = resp3.status();
            println!("[Douyin Stream Detail] 直连 GET 状态: {}", status_code);
            if !status_code.is_success() {
                let err_text = resp3
                    .text()
                    .await
                    .unwrap_or_else(|_| "Failed to read error body".to_string());
                eprintln!(
                    "[Douyin Stream Detail] 直连 GET 失败: GET `{}` failed with status {}: {}",
                    room_url,
                    status_code,
                    err_text
                );
                return Err(format!(
                    "获取房间页面失败: GET `{}` failed with status {}: {}",
                    room_url,
                    status_code,
                    err_text
                ));
            }
            let body = resp3
                .text()
                .await
                .map_err(|e| format!("读取房间页面响应失败: {}", e))?;
            println!("[Douyin Stream Detail] 直连 GET 读取页面成功，长度={}", body.len());
            text = Some(body);
        }
    }

    let text = text.expect("房间页面文本不可为空");
    println!("[Douyin Stream Detail] 房间页面最终文本长度={}", text.len());

    // 解析内嵌的 state JSON（保持原逻辑一致）
    let re = Regex::new(r#"\{\\\"state\\\":\{\\\"appStore.*?\}\\n"#)
        .map_err(|e| format!("构建正则失败: {}", e))?;
     let m = re
        .find(&text)
        .ok_or_else(|| "未能在 HTML 中解析到 Douyin state 数据".to_string())?;
    println!("[Douyin Stream Detail] state JSON 片段匹配成功，长度={}", m.as_str().len());
    let raw = m.as_str().trim();
    let s = raw.replace("\\\"", "\"").replace("\\\\", "\\").replace("]\\n", "");
    let data: Value = serde_json::from_str(&s).map_err(|e| format!("解析 state JSON 失败: {}", e))?;
    println!("[Douyin Stream Detail] state JSON 解析成功，准备返回 state 节点");
    Ok(data["state"].clone())
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

fn extract_detail_from_html_state(web_rid: &str, state: &Value) -> Option<DetailInfo> {
    let room_info = state.get("roomStore")?.get("roomInfo")?;
    let room = room_info.get("room")?;
    let anchor = room_info.get("anchor").cloned().unwrap_or(Value::Null);
    let status = room.get("status")?.as_i64()? as i32;
    let title = room.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
    let _cover = room.get("cover").and_then(|c| c.get("url_list")).and_then(|ul| ul.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string());
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

    println!(
        "[Douyin Stream Detail] extract_detail_from_html_state: webRid={}, status={}, room_id={:?}, title={:?}, owner_nickname={:?}, stream_url_present={}",
        web_rid, status, room_id, title, owner_nickname, stream_url.is_some()
    );

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

// 写桌面文件：完整版本（包含 client headers 与HTML/reflow路径信息）
fn write_douyin_return_to_desktop(client: &HttpClient, room_id_str: &str, quality: &str, parse_path: &str, result: &CommonLiveStreamInfo) {
    let mut content = String::new();
    content.push_str("[Request]\n");
    content.push_str(&format!("room_id_str: {}\nquality: {}\nparse_path: {}\n", room_id_str, quality, parse_path));

    content.push_str("\n[Client Headers]\n");
    content.push_str(&client.get_debug_headers());

    // Cookies
    content.push_str("\n[Cookies]\n");
    let dy_cookie_opt = COLLECTED_DY_COOKIE.lock().unwrap().clone();
    if let Some(c) = dy_cookie_opt {
        content.push_str(&format!("dy_cookie(HTML重试Cookie): {}\n", c));
    } else {
        content.push_str("dy_cookie: <none>\n");
    }

    // 特定路径的请求信息
    if parse_path == "HTML(webRid)" {
        content.push_str("\n[HTML Request]\n");
        content.push_str(&format!("UA: {}\nReferer: {}\nRoom URL: https://live.douyin.com/{}\n", DouyinSitePyDefaults::ua(), DouyinSitePyDefaults::REFERER, room_id_str));
        if let Some(c) = COLLECTED_DY_COOKIE.lock().unwrap().clone() {
            content.push_str(&format!("Cookie: {}\n", c));
        }
    } else {
        // 构造 reflow 请求参数
        let params = vec![
            ("type_id", "0"),
            ("live_id", "1"),
            ("room_id", room_id_str),
            ("sec_user_id", ""),
            ("version_code", "99.99.99"),
            ("app_id", "6383"),
        ];
        let mut query = String::new();
        for (i, (k, v)) in params.iter().enumerate() {
            if i > 0 { query.push('&'); }
            query.push_str(&format!("{}={}", k, v));
        }
        content.push_str("\n[Reflow Request]\n");
        content.push_str(&format!("URL: https://webcast.amemv.com/webcast/room/reflow/info/?{}\n", query));
        content.push_str(&format!("Headers: UA={} Referer={} Accept=application/json, text/plain, */* Accept-Language=zh-CN,zh;q=0.9\n", DouyinSitePyDefaults::ua(), DouyinSitePyDefaults::REFERER));
    }

    content.push_str("\n[Return]\n");
    match serde_json::to_string_pretty(result) {
        Ok(s) => content.push_str(&s),
        Err(_) => content.push_str("<failed to serialize return>"),
    }

    let _ = fs::write("/Users/czeong/Desktop/douyin_live_return.txt", content);
}

// 简化版写桌面文件：用于早期N/A路径
fn write_douyin_return_to_desktop_simple(result: &CommonLiveStreamInfo, room_id_str: &str, quality: &str, parse_path: &str) {
    let mut content = String::new();
    content.push_str("[Request]\n");
    content.push_str(&format!("room_id_str: {}\nquality: {}\nparse_path: {}\n", room_id_str, quality, parse_path));
    content.push_str("\n[Return]\n");
    match serde_json::to_string_pretty(result) {
        Ok(s) => content.push_str(&s),
        Err(_) => content.push_str("<failed to serialize return>"),
    }
    let _ = fs::write("/Users/czeong/Desktop/douyin_live_return.txt", content);
}