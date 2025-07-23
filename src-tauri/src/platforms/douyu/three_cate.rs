// src-tauri/src/api/three_cate.rs
use serde::Deserialize;
// use tauri::command; // Removed as #[tauri::command] macro should suffice
use crate::platforms::common::types_rust::{CommonPlatformCategoryRust, SupportedPlatformRust};
use log::{error, info};

// Assumed structure for a single item in the "three_cate" API response array
// Based on typical Douyu structures, fields like "tag_id", "tag_name", "icon" or "pic" are common.
// The actual API might use slightly different names or nesting.
#[derive(Deserialize, Debug, Clone)]
struct DouyuThreeCateItemRaw {
    #[serde(alias = "tagId", alias = "cateId")] // Common ID fields
    id: String, // Assuming ID is string, could be number
    #[serde(alias = "tagName", alias = "cateName")] // Common name fields
    name: String,
    #[serde(alias = "icon", alias = "pic", alias = "iconUrl")] // Common icon fields
    icon_url: Option<String>,
    // Add other fields if known from the API response, e.g., parent_id, short_name
    // parent_id: Option<String>, // Could be useful to link back to the C2 category
}

// DouyuThreeCateData struct is no longer needed and will be removed.
/*
#[derive(Deserialize, Debug)]
struct DouyuThreeCateData {
    #[serde(alias = "list", alias = "cateList")] // Common list field names
    items: Vec<DouyuThreeCateItemRaw>,
    // Other fields like page count, total items, etc., if present
}
*/

#[derive(Deserialize, Debug)]
struct DouyuThreeCateApiResponse {
    error: i32,
    msg: Option<String>,
    // data field now directly expects a list of items, or null/missing
    data: Option<Vec<DouyuThreeCateItemRaw>>,
}

fn transform_three_cate_to_common(
    raw_items: Vec<DouyuThreeCateItemRaw>,
    parent_cate_id: &str,
) -> Vec<CommonPlatformCategoryRust> {
    raw_items
        .into_iter()
        .map(|raw_item| CommonPlatformCategoryRust {
            id: raw_item.id,
            name: raw_item.name,
            platform: SupportedPlatformRust::Douyu,
            icon_url: raw_item.icon_url,
            parent_id: Some(parent_cate_id.to_string()), // Link to the C2 category that was fetched
        })
        .collect()
}

#[tauri::command]
pub async fn fetch_three_cate(tag_id: i32) -> Result<Vec<CommonPlatformCategoryRust>, String> {
    let tag_id_str = tag_id.to_string();
    let url = format!(
        "https://capi.douyucdn.cn/api/v1/getThreeCate?tag_id={}&client_sys=android",
        tag_id_str
    );
    info!(
        "[API Command] fetch_three_cate called for tag_id: {}",
        tag_id_str
    );

    match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().is_success() {
                let body_text = response
                    .text()
                    .await
                    .map_err(|e| format!("Failed to read response text: {}", e))?;
                match serde_json::from_str::<DouyuThreeCateApiResponse>(&body_text) {
                    Ok(parsed_response) => {
                        if parsed_response.error == 0 {
                            // Directly use parsed_response.data which is Option<Vec<DouyuThreeCateItemRaw>>
                            if let Some(items_list) = parsed_response.data {
                                if !items_list.is_empty() {
                                    let common_data =
                                        transform_three_cate_to_common(items_list, &tag_id_str);
                                    Ok(common_data)
                                } else {
                                    info!("[API Command] fetch_three_cate for {} returned success but empty list (data array was empty).", tag_id_str);
                                    Ok(Vec::new()) // Return empty vec if list is empty but no API error
                                }
                            } else {
                                // This case means the "data" field was null or missing, but error code was 0.
                                info!("[API Command] fetch_three_cate for {} returned success but data field was null or missing.", tag_id_str);
                                Ok(Vec::new())
                            }
                        } else {
                            Err(format!(
                                "ThreeCate API error for tag_id {}. Code: {}, Msg: {:?}",
                                tag_id_str, parsed_response.error, parsed_response.msg
                            ))
                        }
                    }
                    Err(e) => Err(format!(
                        "Failed to parse three_cate JSON for tag_id {}: {}, Body: {}",
                        tag_id_str, e, body_text
                    )),
                }
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error from API".to_string());
                error!(
                    "fetch_three_cate API request failed for tag_id {} with status {}: {}",
                    tag_id_str, status, error_text
                );
                Err(format!(
                    "API request failed for tag_id {} with status {}: {}",
                    tag_id_str, status, error_text
                ))
            }
        }
        Err(e) => {
            error!(
                "fetch_three_cate request failed for tag_id {}: {}",
                tag_id_str, e
            );
            Err(format!("Request failed for tag_id {}: {}", tag_id_str, e))
        }
    }
}
