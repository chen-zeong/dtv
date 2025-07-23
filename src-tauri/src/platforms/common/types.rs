use serde::{Deserialize, Serialize};

// Wrapper for payload like { args: { room_id_str: "..." } }
// Used by get_douyin_live_stream_url and start_douyin_danmaku_listener
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct PayloadWrapperForRoomId {
    pub args: RoomIdDetail,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct RoomIdDetail {
    #[serde(alias = "roomIdStr")]
    pub room_id_str: String,
}

// New payload wrapper specifically for get_douyin_live_stream_url
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct GetStreamUrlArgs {
    pub room_id_str: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct GetStreamUrlPayload {
    pub args: GetStreamUrlArgs,
}

// For the return type of get_douyin_live_stream_url
// Matches LiveStreamInfo interface in DouyinLive.vue
#[derive(Serialize, Clone, Debug)]
pub struct LiveStreamInfo {
    pub title: Option<String>,
    pub anchor_name: Option<String>,
    pub avatar: Option<String>,
    pub stream_url: Option<String>,
    pub status: Option<i32>,
    pub error_message: Option<String>,
}

#[derive(Default, Clone)]
#[allow(dead_code)]
pub struct StreamUrlStore {
    pub url: std::sync::Arc<std::sync::Mutex<String>>,
}

// Moved from main.rs
// State for the Douyin Danmaku listener
#[derive(Default)]
#[allow(dead_code)]
pub struct DouyinDanmakuState(pub std::sync::Mutex<Option<tokio::sync::mpsc::Sender<()>>>);

// State for the Douyu Danmaku listener
#[derive(Default)]
#[allow(dead_code)]
pub struct DouyuDanmakuState(pub std::sync::Mutex<Option<tokio::sync::mpsc::Sender<()>>>);
