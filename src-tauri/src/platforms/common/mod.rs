#![allow(unused_imports)]
pub mod http_client;
pub mod types;
pub mod types_rust;

// Re-export necessary types to make them available directly under platforms::common::TypeName
pub use http_client::FollowHttpClient;
pub use types::BilibiliDanmakuState;
pub use types::DanmakuFrontendPayload;
pub use types::DouyinDanmakuState;
pub use types::DouyuDanmakuState;
pub use types::GetStreamUrlPayload;
pub use types::HuyaDanmakuState;
pub use types::LiveStreamInfo;
