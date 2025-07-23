use crate::platforms::common::http_client::HttpClient;
use crate::platforms::douyin::models::*;
use crate::platforms::douyin::utils::setup_douyin_cookies;
use reqwest::header::REFERER;
use tauri::command;

const DOUYIN_API_REFERER: &str = "https://live.douyin.com/";

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
        });
    }

    let mut http_client =
        HttpClient::new().map_err(|e| format!("Failed to create HttpClient: {}", e))?;

    if let Err(e) = setup_douyin_cookies(&mut http_client, &room_id_str).await {
        return Ok(crate::platforms::common::LiveStreamInfo {
            title: None,
            anchor_name: None,
            avatar: None,
            stream_url: None,
            status: None,
            error_message: Some(format!("Cookie setup failed: {}", e)),
        });
    }

    http_client.insert_header(REFERER, DOUYIN_API_REFERER)?;

    let api_url = format!(
        "https://live.douyin.com/webcast/room/web/enter/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&enter_from=web_live&cookie_enabled=true&screen_width=1920&screen_height=1080&browser_language=zh-CN&browser_platform=MacIntel&browser_name=Chrome&browser_version=116.0.0.0&web_rid={}",
        room_id_str
    );
    println!("[Douyin Info RS] Constructed API URL: {}", api_url);

    let api_response: DouyinApiResponse = match http_client.get_json(&api_url).await {
        Ok(resp) => resp,
        Err(e) => {
            let raw_error_text = http_client
                .get_text(&api_url)
                .await
                .unwrap_or_else(|_| "Failed to get raw error text".to_string());
            println!(
                "[Douyin Info RS] API request failed. Raw error text (if any): {}",
                raw_error_text
            );
            return Ok(crate::platforms::common::LiveStreamInfo {
                title: None,
                anchor_name: None,
                avatar: None,
                stream_url: None,
                status: None,
                error_message: Some(format!("API request failed: {}. URL: {}", e, api_url)),
            });
        }
    };

    if api_response.status_code != 0 {
        let prompts = api_response
            .data
            .as_ref()
            .and_then(|d| d.prompts.as_ref())
            .cloned()
            .unwrap_or_else(|| "Unknown API error".to_string());
        return Ok(crate::platforms::common::LiveStreamInfo {
            title: None,
            anchor_name: None,
            avatar: None,
            stream_url: None,
            status: None,
            error_message: Some(format!(
                "API error (status_code: {}): {}",
                api_response.status_code, prompts
            )),
        });
    }

    let main_data = match api_response.data {
        Some(d) => d,
        None => {
            return Ok(crate::platforms::common::LiveStreamInfo {
                title: None,
                anchor_name: None,
                avatar: None,
                stream_url: None,
                status: None,
                error_message: Some("API response contained no main 'data' object".to_string()),
            })
        }
    };

    let room_data_entry = main_data
        .data
        .as_ref()
        .and_then(|data_vec| data_vec.first())
        .ok_or_else(|| "No room data entry (data.data[0]) found in API response".to_string())?;

    let current_status = room_data_entry.status;

    // Unlike get_douyin_live_stream_url, we don't need to fetch actual stream URLs here.
    // We just return the metadata.

    Ok(crate::platforms::common::LiveStreamInfo {
        title: room_data_entry.title.clone(),
        anchor_name: main_data.user.as_ref().and_then(|u| u.nickname.clone()),
        avatar: main_data
            .user
            .as_ref()
            .and_then(|u| u.avatar_thumb.as_ref())
            .and_then(|at| at.url_list.as_ref())
            .and_then(|ul| ul.first().cloned()),
        stream_url: None, // Explicitly None, as we are not fetching/proxying the stream
        status: Some(current_status),
        error_message: None, // No stream-specific errors here, API errors handled above.
    })
}
