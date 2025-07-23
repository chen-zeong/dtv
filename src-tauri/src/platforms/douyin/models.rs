use serde::{Deserialize, Serialize};
// use serde_json::Value; // For parts of the response that are complex or not fully typed - REMOVED IF UNUSED
use std::collections::HashMap; // For flv_pull_url, hls_pull_url_map

// --- Public struct for the Tauri command result ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LiveStreamInfo {
    pub title: Option<String>,
    pub anchor_name: Option<String>,
    pub avatar: Option<String>,
    pub stream_url: Option<String>, // This will hold the final chosen FLV or HLS URL
    pub error_message: Option<String>,
}

// --- Structs for parsing the Douyin API response (e.g., from /webcast/room/web/enter/) ---

// Represents the overall JSON response from the API
#[derive(Debug, Deserialize, Clone)]
pub struct DouyinApiResponse {
    pub status_code: i32,
    pub data: Option<MainDataContainer>, // data can be null or an object on error
                                         // pub prompts: Option<String>, // Example: "该内容暂时无法无法查看"
                                         // ... other top-level fields if any (e.g., extra, log_pb)
}

#[derive(Debug, Deserialize, Clone)]
pub struct MainDataContainer {
    // The actual room data is often nested in an array, typically with one element
    pub data: Option<Vec<RoomDataEntry>>, // Corresponds to data.data[0] in the demo
    pub user: Option<UserData>,
    // pub partition_road_map: Option<Value>, // If category info is needed later
    pub prompts: Option<String>, // Prompts can also be inside data object
}

#[derive(Debug, Deserialize, Clone)]
pub struct RoomDataEntry {
    pub title: Option<String>,
    pub status: i32, // 2 for live, 4 for replay/not live
    #[serde(rename = "stream_url")]
    pub stream_url_container: Option<StreamUrlContainer>, // Changed from stream_url to avoid conflict if it were a simple string
                                                          // pub owner_user_id: Option<i64>,
                                                          // ... other fields from data.data[0]
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserData {
    pub nickname: Option<String>,
    #[serde(rename = "avatar_thumb")]
    pub avatar_thumb: Option<AvatarData>,
    // ... other user fields
}

#[derive(Debug, Deserialize, Clone)]
pub struct AvatarData {
    #[serde(rename = "url_list")]
    pub url_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)] // Allow dead code for fields used by serde
pub struct StreamUrlContainer {
    #[serde(rename = "flv_pull_url")]
    pub flv_pull_url: Option<HashMap<String, String>>, // e.g., {"FULL_HD1": "url1", "HD1": "url2"}
    #[serde(rename = "hls_pull_url_map")] // As seen in demo, could also be hls_pull_url
    pub hls_pull_url_map: Option<HashMap<String, String>>,
    #[serde(rename = "live_core_sdk_data")]
    pub live_core_sdk_data: Option<LiveCoreSdkData>, // For the nested JSON string
                                                     // pub default_resolution: Option<String>,
                                                     // ... other stream related fields
}

#[derive(Debug, Deserialize, Clone)]
pub struct LiveCoreSdkData {
    #[serde(rename = "pull_data")]
    pub pull_data: Option<PullDataContainer>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PullDataContainer {
    #[serde(rename = "stream_data")]
    pub stream_data: Option<String>, // This is the JSON string e.g., "{\"data\":{\"origin\":...}}"
}

// --- Structs for parsing the nested JSON string within stream_data ---
#[derive(Debug, Deserialize, Clone)]
pub struct InnerStreamDataWrapper {
    // The actual qualities map might be nested further, e.g. under a "data" key or direct
    // Adjust based on actual structure of the stream_data JSON string
    pub data: Option<StreamQualitiesMap>, // Assuming it's like {"data": { "origin": ...}}
                                          // Or if it's directly the map: #[serde(flatten)] pub qualities: StreamQualitiesMap,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StreamQualitiesMap {
    pub origin: Option<StreamQualityDetail>,
    pub sd: Option<StreamQualityDetail>,
    pub hd: Option<StreamQualityDetail>,
    // Potentially others: ld, uhd
}

#[derive(Debug, Deserialize, Clone)]
pub struct StreamQualityDetail {
    pub main: Option<StreamLinks>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)] // Allow dead code for fields used by serde
pub struct StreamLinks {
    pub flv: Option<String>,
    pub hls: Option<String>, // Added HLS field based on typical structure
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
