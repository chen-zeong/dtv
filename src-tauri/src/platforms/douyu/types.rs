use serde::{Deserialize, Serialize};

// Represents a single game category from Douyu API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyuGameCategory {
    pub cate_id: String, // Category ID, Douyu might use `tag_id` or `cate_id` interchangeably in different APIs
    pub game_name: String, // Full name of the game/category
    pub short_name: Option<String>, // Short name (e.g., lol, dota2)
    pub game_url: Option<String>, // URL path segment for this category
    pub game_icon: Option<String>, // URL for category icon
    pub game_src: Option<String>, // URL for a larger banner/image
                         // pub count: Option<i32>,      // Sometimes live stream count is included
                         // pub is_child_cate: Option<i32>, // Indicator for sub-categories
                         // pub child_cates: Option<Vec<DouyuGameCategory>>, // Nested sub-categories
}

// Represents a group of categories (e.g., "热门游戏", "网游竞技")
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyuCategoryGroup {
    pub tag_id: String,           // Group ID
    pub tag_name: String,         // Group name
    pub icon_url: Option<String>, // Optional icon for the group
    #[serde(alias = "list", alias = "cate_list")] // Douyu API can be inconsistent with field names
    pub categories: Vec<DouyuGameCategory>, // List of categories within this group
}

// Expected structure for the overall category API response from Douyu.
// This often comes nested under a `data` field.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyuCategoriesData {
    // Example: Douyu API might return `game_cate` or `cate_list` or similar for groups
    // Adjust field name based on actual API response.
    // For example, if the groups are under a field named `cateInfo` which is an array:
    // pub cateInfo: Vec<DouyuCategoryGroup>,
    // Or if it's directly an array of groups:
    #[serde(alias = "gameList", alias = "cate_list")] // Common names Douyu uses
    pub category_groups: Vec<DouyuCategoryGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyuBaseResponse<T> {
    pub error: i32,
    pub data: Option<T>, // Data can be absent on error
}

// Representing the data as fetched from Douyu API directly for room list within a category
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyuRoomInCategory {
    pub room_id: String,
    pub room_name: String,
    pub nickname: String,
    pub owner_uid: Option<String>,
    pub avatar_mid: Option<String>,    // Streamer avatar
    pub online: i64,                   // Viewer count
    pub show_status: String,           // "1" for live
    pub room_src: Option<String>,      // Thumbnail image for the room
    pub category_name: Option<String>, // Cate name
    pub cate_id: Option<String>,       // Category id
    pub is_vertical: Option<i32>,      // If it's a vertical/mobile stream
    pub vertical_src: Option<String>,  // Thumbnail for vertical stream
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyuRoomsInCateListData {
    #[serde(alias = "rl")]
    pub list: Vec<DouyuRoomInCategory>,
    #[serde(alias = "pgcnt")]
    pub page_count: Option<i32>, // Total page count
    pub ct: Option<i32>, // Current item count or total items
}

// This is the typical structure for Douyu list responses (like rooms in a category)
// pub type DouyuRoomsInCateResponse = DouyuBaseResponse<DouyuRoomsInCateListData>;
// Using a concrete struct for clarity if needed for specific command return types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DouyuRoomsInCateResponse {
    pub error: i32,
    pub data: Option<DouyuRoomsInCateListData>,
}
