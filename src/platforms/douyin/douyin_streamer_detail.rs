            Err(e) => {
                // Fallback to fetching raw text if JSON parsing fails
                let _raw_error_text = http_client.get_text(&api_url).await.unwrap_or_else(|_| "Failed to get raw error text".to_string());
                // eprintln!("Douyin API ({}) raw response: {}", api_url, _raw_error_text); // Log the raw response
            }

    if let Some(_data_content) = &api_response.data { 
        // The presence of data_content itself means success, even if streamer_info is None (e.g. offline)
        if let Some(streamer_info_json) = &api_response.data.as_ref().and_then(|d| d.streamer_info.as_ref()) {
// ... existing code ...
        }
    } 