use isahc::config::{Configurable, RedirectPolicy}; // For HttpClient configuration
use isahc::{http, prelude::*, HttpClient, Request}; // For HTTP client and request building
use md5::Digest; // For hasher
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use std::time::{SystemTime, UNIX_EPOCH}; // For timestamp for did // For URL encoding keyword

// Renamed from search_anchor to avoid ambiguity with Tauri command
pub async fn perform_anchor_search(keyword: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = HttpClient::builder()
        .redirect_policy(RedirectPolicy::Follow)
        .default_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")
        .build()?;

    let mut hasher = md5::Md5::new();
    hasher.update(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_nanos()
            .to_string(),
    );
    let did = format!("{:x}", hasher.finalize());

    let url = format!(
        "https://www.douyu.com/japi/search/api/searchUser?kw={}&page=1&pageSize=20&filterType=0",
        percent_encode(keyword.as_bytes(), NON_ALPHANUMERIC)
    );

    let request = Request::builder()
        .method(http::Method::GET)
        .uri(url)
        .header("Referer", "https://www.douyu.com/search/")
        .header("Cookie", format!("dy_did={}; acf_did={}", did, did))
        .body(())?;

    let mut response = client.send(request)?;
    let text = response.text()?;

    Ok(text)
}
