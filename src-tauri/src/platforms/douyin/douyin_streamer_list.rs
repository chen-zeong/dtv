use crate::platforms::common::http_client::HttpClient;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::{Deserialize, Serialize};
use tauri::State; // Removed SET_COOKIE

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyinRoomCover {
    pub url_list: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyinRoomOwner {
    pub nickname: String,
    pub avatar_thumb: Option<DouyinRoomCover>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyinRoomStats {
    pub total_user_str: String,
    pub user_count_str: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyinRoom {
    #[serde(rename = "id_str")]
    // id_str is inside the JSON room object and contains the correct room_id
    pub room_id: String,

    pub title: String,
    pub cover: DouyinRoomCover,
    pub owner: DouyinRoomOwner,
    pub stats: DouyinRoomStats,
    // Add other fields from the JSON room object if necessary
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyinPartitionRoomData {
    #[serde(rename = "web_rid")] // Capture the top-level web_rid from JSON
    pub actual_web_rid_for_frontend: String, // Field to hold the true web_rid

    pub room: DouyinRoom,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyinPartitionDataWrapper {
    pub data: Vec<DouyinPartitionRoomData>,
    pub count: i32,
    pub offset: i32,
    pub has_more: Option<bool>, // Added for pagination
                                // pub total: Option<i32>, // If available and needed
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyinPartitionApiResponse {
    pub data: DouyinPartitionDataWrapper,
    pub status_code: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LiveRoomFrontend {
    pub web_rid: String,
    pub room_id: String,
    pub title: String,
    pub cover_url: String,
    pub owner_nickname: String,
    pub user_count_str: String,
    pub avatar_url: String,
}

// This struct will wrap the list of rooms and the has_more flag for the frontend.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyinLiveListResponse {
    pub rooms: Vec<LiveRoomFrontend>,
    pub has_more: bool,
    pub next_offset: i32, // The offset to use for the next request
}

#[tauri::command]
pub async fn fetch_douyin_partition_rooms(
    _http_client: State<'_, reqwest::Client>,
    partition: String,
    partition_type: String,
    offset: i32, // This is the offset for the current request (0, 15, 30...)
    ms_token: String,
) -> Result<DouyinLiveListResponse, String> {
    let count: i32 = 15; // Number of items requested per page, explicitly typed as i32

    // 使用直连HTTP客户端，绕过所有代理设置
    let local_client = HttpClient::new_direct_connection()
        .map_err(|e| format!("Failed to create direct connection HttpClient: {}", e))?;

    // Use hardcoded ttwid and odin_tt from the user's working test for now
    let hardcoded_odin_tt = "54c68ba8fa8ce792ad017c55272d171c283baedc87b2f6282ca8706df295cbd89c5d55449b587b7ebe0a2e352e394a86975955c9ed7f98f209996bdca2749479619aceecc7b75c2374e146b5a722b2e1";
    let hardcoded_ttwid = "1%7CdVwg8DUriPlMDlcGA6XsVP8FZW2vzZEtEnoAxpXQxP8%7C1757517390%7C954f1753f33b21b018d616437b3f053026c22f17cde00bccd655bfb0d71056c5";

    let cookie_string = format!("odin_tt={}; ttwid={}", hardcoded_odin_tt, hardcoded_ttwid);

    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        HeaderValue::from_str(&cookie_string)
            .map_err(|e| format!("Failed to create cookie header value: {}", e))?,
    );

    let url = format!(
        "https://live.douyin.com/webcast/web/partition/detail/room/v2/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&enter_from=web_homepage_hot&cookie_enabled=true&screen_width=1920&screen_height=1080&browser_language=zh-CN&browser_platform=MacIntel&browser_name=Chrome&browser_version=120.0.0.0&count={}&offset={}&partition={}&partition_type={}&req_from=2&msToken={}&a_bogus=D7sfgF7iON8bKd%2FGmOiievelRgdlNs8yQrTxbS%2FKSNzrcw0Y%2FSNHicSsjxzJ32CZGuphkCI7iEslbjdcsIXz11npomkDuziW-zAnn0sLMqqsT0JQEHREShzzuwsnUb4q-%2Fc5Elk51sBe2DQW9rAdlpMGH5TaQmRDWHFWdM0cc9WYfSyPm3aROZEAPfqumb243D%3D%3D",
        count, offset, partition, partition_type, ms_token
    );

    match local_client
        .get_json_with_headers::<DouyinPartitionApiResponse>(&url, Some(headers))
        .await
    {
        Ok(api_response) => {
            if api_response.status_code == 0 {
                let mut frontend_rooms = Vec::new();
                let received_rooms_count = api_response.data.data.len(); // Number of rooms actually received from this API call

                for room_data in api_response.data.data {
                    let room_details = room_data.room;

                    let avatar_url = room_details
                        .owner
                        .avatar_thumb
                        .as_ref()
                        .and_then(|thumb| thumb.url_list.get(0))
                        .cloned()
                        .unwrap_or_default();

                    let user_count_display = room_details
                        .stats
                        .user_count_str
                        .clone()
                        .unwrap_or_else(|| room_details.stats.total_user_str.clone());

                    frontend_rooms.push(LiveRoomFrontend {
                        web_rid: room_data.actual_web_rid_for_frontend.clone(),
                        room_id: room_details.room_id.clone(),
                        title: room_details.title,
                        cover_url: room_details
                            .cover
                            .url_list
                            .get(0)
                            .cloned()
                            .unwrap_or_default(),
                        owner_nickname: room_details.owner.nickname,
                        user_count_str: user_count_display,
                        avatar_url,
                    });
                }

                // New has_more logic: true if we received exactly 'count' items
                let has_more = received_rooms_count == (count as usize);

                // New next_offset logic: current offset + number of items requested for a page
                let next_offset_for_frontend = offset + count;

                Ok(DouyinLiveListResponse {
                    rooms: frontend_rooms,
                    has_more,
                    next_offset: next_offset_for_frontend,
                })
            } else {
                Err(format!(
                    "Douyin API returned non-zero status code: {}",
                    api_response.status_code
                ))
            }
        }
        Err(e) => Err(format!("Network error fetching Douyin room list: {}", e)),
    }
}
