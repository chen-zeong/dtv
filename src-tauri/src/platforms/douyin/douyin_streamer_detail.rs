use crate::platforms::common::http_client::HttpClient;
use crate::platforms::common::types::StreamVariant;
use crate::platforms::common::GetStreamUrlPayload;
use crate::platforms::common::LiveStreamInfo as CommonLiveStreamInfo;
use crate::platforms::douyin::web_api::{choose_flv_stream, fetch_room_data, DouyinRoomData};
use crate::proxy::{start_proxy, ProxyServerHandle};
use crate::StreamUrlStore;
use serde_json::Value;
use tauri::{command, AppHandle, State};

const QUALITY_OD: &str = "OD";
const QUALITY_BD: &str = "BD";
const QUALITY_UHD: &str = "UHD";
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
        QUALITY_OD.to_string(),
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
    let requested_id = payload.args.room_id_str.trim().to_string();
    if requested_id.is_empty() {
        return Ok(CommonLiveStreamInfo {
            title: None,
            anchor_name: None,
            avatar: None,
            stream_url: None,
            status: None,
            error_message: Some("Douyin web_id cannot be empty.".to_string()),
            upstream_url: None,
            available_streams: None,
            normalized_room_id: None,
            web_rid: None,
        });
    }

    println!(
        "[Douyin Stream Detail] Fetching stream for '{}' with requested quality '{}'",
        requested_id, quality
    );

    let http_client = HttpClient::new_direct_connection()
        .map_err(|e| format!("Failed to create direct connection HttpClient: {}", e))?;

    let DouyinRoomData { room } = fetch_room_data(&http_client, &requested_id, None).await?;
    let web_rid = extract_web_rid(&room).unwrap_or_else(|| requested_id.clone());
    let status = room
        .get("status")
        .and_then(|v| v.as_i64())
        .unwrap_or_default() as i32;
    let title = room
        .get("title")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let anchor_name = extract_anchor_name(&room);
    let avatar = extract_avatar(&room);
    let available_streams = collect_available_streams(&room);

    if status != 2 {
        println!(
            "[Douyin Stream Detail] Room '{}' is not live (status={}). Returning metadata only.",
            web_rid, status
        );
        return Ok(CommonLiveStreamInfo {
            title,
            anchor_name,
            avatar,
            stream_url: None,
            status: Some(status),
            error_message: None,
            upstream_url: None,
            available_streams: available_streams.clone(),
            normalized_room_id: None,
            web_rid: Some(web_rid),
        });
    }

    let target_quality = normalize_quality_tag(&quality);
    let selected = choose_flv_stream(&room, target_quality)
        .or_else(|| first_flv_stream(&room))
        .ok_or_else(|| {
            "[Douyin Stream Detail] No FLV streams available in stream_url.flv_pull_url".to_string()
        })?;
    let (selected_key, real_url) = selected;
    println!(
        "[Douyin Stream Detail] Selected FLV stream key='{}' url='{}'",
        selected_key, real_url
    );

    // Persist upstream URL so the proxy can serve it
    {
        let mut guard = stream_url_store.url.lock().unwrap();
        *guard = real_url.clone();
    }

    let proxied_url = match start_proxy(app_handle, proxy_server_handle, stream_url_store).await {
        Ok(proxy) => {
            println!(
                "[Douyin Stream Detail] Proxy started successfully for '{}': {}",
                web_rid, proxy
            );
            proxy
        }
        Err(e) => {
            eprintln!(
                    "[Douyin Stream Detail] Failed to start proxy for '{}': {}. Returning upstream URL directly.",
                    web_rid, e
                );
            return Ok(CommonLiveStreamInfo {
                title,
                anchor_name,
                avatar,
                stream_url: Some(real_url.clone()),
                status: Some(status),
                error_message: Some(format!("启动本地代理失败: {}", e)),
                upstream_url: Some(real_url),
                available_streams,
                normalized_room_id: None,
                web_rid: Some(web_rid),
            });
        }
    };

    Ok(CommonLiveStreamInfo {
        title,
        anchor_name,
        avatar,
        stream_url: Some(proxied_url.clone()),
        status: Some(status),
        error_message: None,
        upstream_url: Some(real_url),
        available_streams,
        normalized_room_id: None,
        web_rid: Some(web_rid),
    })
}

fn normalize_quality_tag(input: &str) -> &str {
    match input.trim().to_uppercase().as_str() {
        "OD" | "原画" => QUALITY_OD,
        "BD" | "高清" => QUALITY_BD,
        "UHD" | "标清" => QUALITY_UHD,
        _ => QUALITY_OD,
    }
}

pub(crate) fn extract_web_rid(room: &Value) -> Option<String> {
    room.get("owner")
        .and_then(|o| o.get("web_rid"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            room.get("anchor")
                .and_then(|a| a.get("web_rid"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            room.get("web_rid")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

pub(crate) fn extract_anchor_name(room: &Value) -> Option<String> {
    room.get("anchor_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            room.get("owner")
                .and_then(|o| o.get("nickname"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            room.get("anchor")
                .and_then(|a| a.get("nickname"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

pub(crate) fn extract_avatar(room: &Value) -> Option<String> {
    room.get("owner")
        .and_then(|o| o.get("avatar_thumb"))
        .and_then(|thumb| thumb.get("url_list"))
        .and_then(|list| list.get(0))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            room.get("anchor")
                .and_then(|a| a.get("avatar_thumb"))
                .and_then(|thumb| thumb.get("url_list"))
                .and_then(|list| list.get(0))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

pub(crate) fn collect_available_streams(room: &Value) -> Option<Vec<StreamVariant>> {
    let flv_map = room
        .get("stream_url")
        .and_then(|v| v.get("flv_pull_url"))
        .and_then(|v| v.as_object())?;
    let variants = flv_map
        .iter()
        .filter_map(|(k, v)| {
            v.as_str().map(|url| StreamVariant {
                url: url.to_string(),
                format: Some("flv".to_string()),
                desc: Some(k.to_string()),
                qn: None,
                protocol: url.split(':').next().map(|s| s.to_string()),
            })
        })
        .collect::<Vec<_>>();
    if variants.is_empty() {
        None
    } else {
        Some(variants)
    }
}

fn first_flv_stream(room: &Value) -> Option<(String, String)> {
    let flv_map = room
        .get("stream_url")
        .and_then(|v| v.get("flv_pull_url"))
        .and_then(|v| v.as_object())?;
    flv_map
        .iter()
        .find_map(|(k, v)| v.as_str().map(|url| (k.to_string(), url.to_string())))
}
