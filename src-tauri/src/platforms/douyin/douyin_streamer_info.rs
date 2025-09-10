use crate::platforms::common::http_client::HttpClient;
use crate::platforms::douyin::models::*;
use crate::platforms::douyin::utils::setup_douyin_cookies;
use reqwest::header::{REFERER, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT, HeaderName};
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
        });
    }

    // 使用直连HTTP客户端，绕过所有代理设置
    let mut http_client =
        HttpClient::new_direct_connection().map_err(|e| format!("Failed to create direct connection HttpClient: {}", e))?;

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

    // 设置与JS文件一致的headers
    http_client.insert_header(ACCEPT, "application/json, text/plain, */*")?;
    http_client.insert_header(ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9")?;
    http_client.insert_header(HeaderName::from_static("priority"), "u=1, i")?;
    http_client.insert_header(REFERER, "https://live.douyin.com/7254458840")?;
    http_client.insert_header(HeaderName::from_static("sec-ch-ua"), "\"Chromium\";v=\"140\", \"Not=A?Brand\";v=\"24\", \"Google Chrome\";v=\"140\"")?;
    http_client.insert_header(HeaderName::from_static("sec-ch-ua-mobile"), "?0")?;
    http_client.insert_header(HeaderName::from_static("sec-ch-ua-platform"), "\"macOS\"")?;
    http_client.insert_header(HeaderName::from_static("sec-fetch-dest"), "empty")?;
    http_client.insert_header(HeaderName::from_static("sec-fetch-mode"), "cors")?;
    http_client.insert_header(HeaderName::from_static("sec-fetch-site"), "same-origin")?;
    http_client.insert_header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")?;

    let api_url = format!(
        "https://live.douyin.com/webcast/room/web/enter/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&enter_from=web_homepage_follow&cookie_enabled=true&screen_width=1920&screen_height=1080&browser_language=zh-CN&browser_platform=MacIntel&browser_name=Chrome&browser_version=140.0.0.0&web_rid={}&enter_source=&is_need_double_stream=false&insert_task_id=&live_reason=&msToken=djIQSLNfdq3BLVY9-hIFbpJVQs238wUtsl1_Zvc2-rkmUSUy44JUt-L_jMcpo--kcwpK8Sc4C7fUvX-QL-mrqE1RM0E65tIZ8Rz4UoVXrzbCAhvwNKSX0TG8r1KNdI3K9dbBvI3Lb6W62nr7LStyw-41pkfZkFW2Vfi9zqnnLDSM-NMhCJTrxQ%3D%3D&a_bogus=EJ0fkF67Dx%2FfPdKGuObyCHlU2lxMNB8yQZixWCluCNzJOXUTjuP7gcbZboqs4doR3bpsiHIHTx0lYEncTdUs1ZrkumkfSmzyJzACVgsL8qwsGFJQgHfZeukFqwBN0Rsqa%2FcIE1g78sBK2d5W9HAQldBaC5Pa5QmDWHqydM9bj9WbDAyPu3rROMEWiEwPBQ2-rf%3D%3D",
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
            
            // 获取调试信息
            let debug_headers = http_client.get_debug_headers();
            let debug_cookies = http_client.get_debug_cookies(&api_url);
            
            println!("[Douyin Info RS] API request failed.");
            println!("URL: {}", api_url);
            println!("Headers:\n{}", debug_headers);
            println!("Cookies: {}", debug_cookies);
            println!("Error: {}", e);
            println!("Raw error text: {}", raw_error_text);
            
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
