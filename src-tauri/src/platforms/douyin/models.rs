use serde::{Deserialize, Serialize};
// use serde_json::Value; // For parts of the response that are complex or not fully typed - REMOVED IF UNUSED
use std::collections::HashMap; // For flv_pull_url

// --- Public struct for the Tauri command result ---
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct LiveStreamInfo {
    pub title: Option<String>,
    pub anchor_name: Option<String>,
    pub avatar: Option<String>,
    pub stream_url: Option<String>,
    pub error_message: Option<String>,
}

// --- Structs for parsing the Douyin API response (e.g., from /webcast/room/web/enter/) ---

// [REMOVED] DouyinApiResponse and MainDataContainer as unused per user request

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct RoomDataEntry {
    pub title: Option<String>,
    pub status: i32,
    #[serde(rename = "stream_url")]
    pub stream_url_container: Option<StreamUrlContainer>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct LiveCoreSdkData {
    #[serde(rename = "pull_data")]
    pub pull_data: Option<PullDataContainer>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct UserData {
    pub nickname: Option<String>,
    #[serde(rename = "avatar_thumb")]
    pub avatar_thumb: Option<AvatarData>,
    // ... other user fields
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct AvatarData {
    #[serde(rename = "url_list")]
    pub url_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)] // Allow dead code for fields used by serde
pub struct StreamUrlContainer {
    #[serde(rename = "flv_pull_url")]
    pub flv_pull_url: Option<HashMap<String, String>>, // e.g., {"FULL_HD1": "url1", "HD1": "url2"}
    #[serde(rename = "live_core_sdk_data")]
    pub live_core_sdk_data: Option<LiveCoreSdkData>, // For the nested JSON string
                                                     // pub default_resolution: Option<String>,
                                                     // ... other stream related fields
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct PullDataContainer {
    #[serde(rename = "stream_data")]
    pub stream_data: Option<String>,
    #[serde(rename = "options")]
    pub options: Option<PullOptions>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct PullOptions {
    #[serde(rename = "qualities")]
    pub qualities: Option<Vec<PullQualityOption>>, // e.g., [{ name: "原画", level: 3, sdk_key: "FULL_HD1" }, ...]
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct PullQualityOption {
    pub name: Option<String>,
    pub level: Option<i32>,
    #[serde(rename = "sdk_key")]
    pub sdk_key: Option<String>,
}

// --- Structs for parsing the nested JSON string within stream_data ---
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct InnerStreamDataWrapper {
    pub data: Option<StreamQualitiesMap>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct StreamQualitiesMap {
    pub origin: Option<StreamQualityDetail>,
    pub sd: Option<StreamQualityDetail>,
    pub hd: Option<StreamQualityDetail>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct StreamQualityDetail {
    pub main: Option<StreamLinks>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)] // Allow dead code for fields used by serde
pub struct StreamLinks {
    pub flv: Option<String>,
    // pub cdn_multi_arc: Option<Value>,
    // pub sdk_params: Option<String>,
}

// Enum for stream resolutions, if needed for consistent key access
// (Not strictly necessary if using string keys directly from preferred order list)
// pub enum ResolutionKey {
//     FullHd1, // "FULL_HD1"
//     Hd1,     // "HD1"
//     Sd1,     // "SD1"
//     Sd2,     // "SD2"
// }
