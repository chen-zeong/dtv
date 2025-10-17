use serde::{Deserialize, Serialize};
use tauri::command;

use crate::platforms::common::http_client::HttpClient;

#[derive(Debug, Serialize, Deserialize)]
pub struct HuyaStreamerFrontend {
    pub room_id: String,
    pub title: String,
    pub nickname: String,
    pub avatar: String,
    pub room_cover: String,
    pub viewer_count_str: String,
    pub platform: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HuyaLiveListFrontendResponse {
    pub error: i32,
    pub msg: Option<String>,
    pub data: Option<Vec<HuyaStreamerFrontend>>, // simple list, frontend can decide pagination by page size
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HuyaApiResponse {
    #[serde(rename = "vList")]
    v_list: Option<Vec<serde_json::Value>>, // Items are dynamic; we'll map selectively
}

fn map_huya_item_to_frontend(item: &serde_json::Value) -> Option<HuyaStreamerFrontend> {
    let s_nick = item
        .get("sNick")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let s_intro = item
        .get("sIntroduction")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let s_screenshot = item
        .get("sScreenshot")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let l_profile_room = item
        .get("lProfileRoom")
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        .to_string();
    let s_avatar_180 = item
        .get("sAvatar180")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let l_user_count = item.get("lUserCount").and_then(|v| v.as_i64()).unwrap_or(0);

    // Viewer count string: format simple number with suffix if large
    let viewer_count_str = if l_user_count >= 10_000 {
        format!("{:.1}万", (l_user_count as f64) / 10_000.0)
    } else {
        l_user_count.to_string()
    };

    Some(HuyaStreamerFrontend {
        room_id: l_profile_room,
        title: s_intro,
        nickname: s_nick,
        avatar: s_avatar_180,
        room_cover: s_screenshot,
        viewer_count_str,
        platform: "huya".to_string(),
    })
}

#[command]
pub async fn fetch_huya_live_list(
    i_gid: String,
    i_page_no: u32,
    i_page_size: u32,
) -> HuyaLiveListFrontendResponse {
    let url = format!(
        "https://live.huya.com/liveHttpUI/getLiveList?iGid={}&iPageNo={}&iPageSize={}",
        urlencoding::encode(&i_gid),
        i_page_no,
        i_page_size
    );

    let client = match HttpClient::new_direct_connection() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[Huya Backend] Failed to init HTTP client: {}", e);
            return HuyaLiveListFrontendResponse {
                error: 500,
                msg: Some(e),
                data: None,
            };
        }
    };

    // 修复：get_json 是异步方法，需要 .await；并直接匹配 Result 而不是对 Result 使用 .await
    let resp_value: serde_json::Value = match client.get_json::<serde_json::Value>(&url).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[Huya Backend] Request failed: {}", e);
            return HuyaLiveListFrontendResponse {
                error: 500,
                msg: Some(e),
                data: None,
            };
        }
    };

    // 兼容两种可能的返回结构：顶层 vList 或 data.vList
    let v_list_opt = resp_value
        .get("vList")
        .and_then(|v| v.as_array())
        .cloned()
        .or_else(|| {
            resp_value
                .get("data")
                .and_then(|d| d.get("vList"))
                .and_then(|v| v.as_array())
                .cloned()
        });

    if let Some(arr) = v_list_opt {
        let mapped: Vec<HuyaStreamerFrontend> = arr
            .iter()
            .filter_map(|item| map_huya_item_to_frontend(item))
            .collect();
        HuyaLiveListFrontendResponse {
            error: 0,
            msg: Some("Success".to_string()),
            data: Some(mapped),
        }
    } else {
        HuyaLiveListFrontendResponse {
            error: -1,
            msg: Some("No vList in response".to_string()),
            data: None,
        }
    }
}
