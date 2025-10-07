use reqwest::header::{HeaderMap, HeaderValue, REFERER, USER_AGENT, COOKIE};
use serde_json::Value;
use tauri::{command, AppHandle, State};

use crate::StreamUrlStore;
use crate::proxy::{start_proxy, ProxyServerHandle};
use crate::platforms::common::types::StreamVariant;

#[command]
pub async fn get_bilibili_live_stream_url_with_quality(
    app_handle: AppHandle,
    stream_url_store: State<'_, StreamUrlStore>,
    proxy_server_handle: State<'_, ProxyServerHandle>,
    payload: crate::platforms::common::GetStreamUrlPayload,
    quality: String,
    cookie: Option<String>,
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
        });
    }

    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

    // Build headers
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(ua).unwrap());
    headers.insert(REFERER, HeaderValue::from_static("https://live.bilibili.com/"));
    if let Some(c) = cookie.as_ref() {
        if !c.is_empty() {
            headers.insert(COOKIE, HeaderValue::from_str(c).unwrap_or(HeaderValue::from_static("")));
        }
    }

    // 添加必要的 Origin，以符合部分接口对 CSRF 的检查
    headers.insert(reqwest::header::ORIGIN, HeaderValue::from_static("https://live.bilibili.com"));
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    // Helper: request playinfo with optional qn
    async fn request_playinfo(client: &reqwest::Client, room_id: &str, qn: Option<i32>) -> Result<Value, String> {
        let url = "https://api.live.bilibili.com/xlive/web-room/v2/index/getRoomPlayInfo";
        let mut params = vec![
            ("room_id", room_id.to_string()),
            ("protocol", "0,1".to_string()),
            ("format", "0,1,2".to_string()),
            ("codec", "0,1".to_string()),
            ("platform", "web".to_string()),
            ("dolby", "5".to_string()),
        ];
        if let Some(q) = qn { params.push(("qn", q.to_string())); }
        let resp = client
            .get(url)
            .query(&params)
            .send()
            .await
            .map_err(|e| format!("PlayInfo request failed: {}", e))?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| format!("Read text failed: {}", e))?;
        if !status.is_success() { return Err(format!("PlayInfo status: {} body: {}", status, text)); }
        serde_json::from_str::<Value>(&text).map_err(|e| format!("JSON parse failed: {} | body: {}", e, text))
    }

    // 1) First request to get qn mapping
    let playinfo = request_playinfo(&client, &room_id, None).await?;
    let playurl = playinfo["data"]["playurl_info"]["playurl"].clone();

    // Build qn->desc map
    let mut qn_map: Vec<(i32, String)> = vec![];
    if let Some(arr) = playurl.get("g_qn_desc").and_then(|v| v.as_array()) {
        for item in arr {
            let qn = item.get("qn").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let desc = item.get("desc").and_then(|v| v.as_str()).unwrap_or("").to_string();
            qn_map.push((qn, desc));
        }
    }
    // 调试输出：可用的 qn 列表及描述
    if !qn_map.is_empty() {
        let qn_str = qn_map.iter().map(|(q, d)| format!("{}:{}", q, d)).collect::<Vec<_>>().join(", ");
        eprintln!("[Bilibili] qn_map for room {} => [{}]", room_id, qn_str);
    } else {
        eprintln!("[Bilibili] qn_map is empty for room {}", room_id);
    }

    // Choose qn by desired quality text
    fn match_qn(qn_map: &[(i32, String)], quality: &str) -> Option<i32> {
        let q = quality.trim();
        // Try exact desc includes
        for (qn, desc) in qn_map.iter() {
            if (q == "原画" && desc.contains("原")) || (q == "高清" && desc.contains("高清")) || (q == "标清" && (desc.contains("标清") || desc.contains("清"))) {
                return Some(*qn);
            }
        }
        // Fallback: choose max qn
        qn_map.iter().map(|(qn, _)| *qn).max()
    }
    let selected_qn = match_qn(&qn_map, &quality);
    let selected_desc = selected_qn.and_then(|qn| qn_map.iter().find(|(q, _)| *q == qn).map(|(_, d)| d.clone()));
    eprintln!("[Bilibili] selected quality '{}' -> qn={:?}, desc={:?}", quality, selected_qn, selected_desc);

    // 2) Second request with selected qn (if any)
    let playinfo2 = request_playinfo(&client, &room_id, selected_qn).await?;
    let playurl2 = playinfo2["data"]["playurl_info"]["playurl"].clone();

    // Determine live status from room_init
    let room_init_url = format!("https://api.live.bilibili.com/room/v1/Room/room_init?id={}", room_id);
    let init_resp = client.get(&room_init_url).send().await.map_err(|e| format!("room_init failed: {}", e))?;
    let init_text = init_resp.text().await.map_err(|e| format!("room_init read text failed: {}", e))?;
    let init_json: Value = serde_json::from_str(&init_text).map_err(|e| format!("room_init json failed: {} | {}", e, init_text))?;
    let live_status = init_json["data"]["live_status"].as_i64().unwrap_or(0);
    if live_status != 1 {
        return Ok(crate::platforms::common::LiveStreamInfo {
            title: init_json["data"]["title"].as_str().map(|s| s.to_string()),
            anchor_name: init_json["data"]["uname"].as_str().map(|s| s.to_string()),
            avatar: None,
            stream_url: None,
            status: Some(0),
            error_message: None,
            upstream_url: None,
            available_streams: None,
        });
    }

    // 收集所有可用的播放地址（包含不同 host）
    let mut variants: Vec<StreamVariant> = Vec::new();
    let mut final_url_ts: Option<String> = None;
    let mut final_url_flv: Option<String> = None;
    if let Some(streams) = playurl2.get("stream").and_then(|v| v.as_array()) {
        for stream_item in streams {
            let protocol = stream_item.get("protocol_name").and_then(|v| v.as_str()).map(|s| s.to_string());
            if let Some(formats) = stream_item.get("format").and_then(|v| v.as_array()) {
                for format_item in formats {
                    let format_name = format_item.get("format_name").and_then(|v| v.as_str()).unwrap_or("");
                    if let Some(codecs) = format_item.get("codec").and_then(|v| v.as_array()) {
                        for codec_item in codecs {
                            let base_url = codec_item.get("base_url").and_then(|v| v.as_str()).unwrap_or("");
                            if let Some(url_infos) = codec_item.get("url_info").and_then(|v| v.as_array()) {
                                for ui in url_infos {
                                    let host = ui.get("host").and_then(|v| v.as_str()).unwrap_or("");
                                    let extra = ui.get("extra").and_then(|v| v.as_str()).unwrap_or("");
                                    let composed = format!("{}{}{}", host, base_url, extra);
                                    if !composed.is_empty() {
                                        // 记录到 variants
                                        variants.push(StreamVariant {
                                            url: composed.clone(),
                                            format: Some(format_name.to_string()),
                                            desc: selected_desc.clone(),
                                            qn: selected_qn,
                                            protocol: protocol.clone(),
                                        });
                                        // 优先选择第一个 TS(M3U8) 地址作为默认播放地址
                                        if final_url_ts.is_none() && format_name == "ts" {
                                            final_url_ts = Some(composed.clone());
                                        }
                                        // 其次选择第一个 FLV 地址作为备用
                                        if final_url_flv.is_none() && format_name == "flv" {
                                            final_url_flv = Some(composed.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if final_url_ts.is_none() && final_url_flv.is_none() {
        return Ok(crate::platforms::common::LiveStreamInfo {
            title: init_json["data"]["title"].as_str().map(|s| s.to_string()),
            anchor_name: init_json["data"]["uname"].as_str().map(|s| s.to_string()),
            avatar: None,
            stream_url: None,
            status: Some(2),
            error_message: Some("未从播放信息中获取到M3U8或FLV地址".to_string()),
            upstream_url: None,
            available_streams: Some(variants),
        });
    }

    let real_url = if let Some(u) = final_url_ts.clone() { u } else { final_url_flv.clone().unwrap() };

    // 根据是否为 HLS 选择是否启动本地代理（目前直接返回 M3U8 上游地址，FLV 仍通过代理）
    let proxied_url = if final_url_ts.is_some() {
        // HLS：直接使用上游 M3U8 地址
        Some(real_url.clone())
    } else {
        // FLV：写入到 Store 并启动代理
        {
            let mut current_url_in_store = stream_url_store.url.lock().unwrap();
            *current_url_in_store = real_url.clone();
        }
        match start_proxy(app_handle, proxy_server_handle, stream_url_store).await {
            Ok(proxy) => Some(proxy),
            Err(e) => {
                eprintln!("[Bilibili] Failed to start proxy: {}", e);
                None
            }
        }
    };

    let final_error_message = if proxied_url.is_none() { Some("代理启动失败".to_string()) } else { None };

    Ok(crate::platforms::common::LiveStreamInfo {
        title: init_json["data"]["title"].as_str().map(|s| s.to_string()),
        anchor_name: init_json["data"]["uname"].as_str().map(|s| s.to_string()),
        avatar: None,
        stream_url: proxied_url,
        status: Some(2),
        error_message: final_error_message,
        upstream_url: Some(real_url),
        available_streams: Some(variants),
    })
}