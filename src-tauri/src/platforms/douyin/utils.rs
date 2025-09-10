use crate::platforms::common::http_client::HttpClient;
use reqwest::header::COOKIE;



// This is the public function your live.rs will call
pub async fn setup_douyin_cookies(
    http_client: &mut HttpClient,
    _room_id: &str,
) -> Result<(), String> {
    // 使用与JS文件完全一致的cookie值
    let cookie_value = "ttwid=1%7CMzira2CT1P-CLey42gr0QsEGL_Wmq3Yg5PQF2X412hY%7C1677897397%7C0df7a1da2a44ccac7dda848d236c8d5276d3eae070dfb3fe6df6e86fbd896d93;";

    http_client.insert_header(COOKIE, cookie_value)?;

    Ok(())
}
