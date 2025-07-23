use crate::platforms::common::http_client::HttpClient;
use crate::platforms::common::http_client::DEFAULT_USER_AGENT;
use reqwest::header::{COOKIE, REFERER, USER_AGENT};

const DOUYIN_BASE_URL: &str = "https://live.douyin.com/";

#[derive(Debug, Default)]
struct InitialCookies {
    ttwid: Option<String>,
    odin_tt: Option<String>,
}

// Function to extract initial cookies like ttwid and odin_tt
async fn fetch_initial_cookies(room_url: &str) -> Result<InitialCookies, String> {
    let temp_client = reqwest::Client::new();
    let response = temp_client
        .get(DOUYIN_BASE_URL) // Fetch from base URL first to get general cookies
        .header(USER_AGENT, DEFAULT_USER_AGENT)
        .header(REFERER, room_url) // Referer can be the specific room or just base URL
        .send()
        .await
        .map_err(|e| {
            format!(
                "Failed to make initial request to {} for cookies: {}",
                DOUYIN_BASE_URL, e
            )
        })?;

    if !response.status().is_success() {
        return Err(format!(
            "Initial request to {} for cookies failed with status: {}",
            DOUYIN_BASE_URL,
            response.status()
        ));
    }

    let mut cookies_found = InitialCookies::default();

    for cookie in response.cookies() {
        match cookie.name() {
            "ttwid" => cookies_found.ttwid = Some(cookie.value().to_string()),
            "odin_tt" => cookies_found.odin_tt = Some(cookie.value().to_string()),
            _ => {}
        }
    }

    if cookies_found.ttwid.is_none() {
        return Err("ttwid cookie not found in initial response".to_string());
    }
    Ok(cookies_found)
}

// Function to extract __ac_nonce cookie
// The demo implies this might be on the specific room page, or also from a general page.
// Let's assume fetching the room_url itself provides it.
async fn fetch_ac_nonce(room_url: &str) -> Result<String, String> {
    let temp_client = reqwest::Client::new();
    let response = temp_client
        .get(room_url)
        .header(USER_AGENT, DEFAULT_USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("Failed to request {} for __ac_nonce: {}", room_url, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Request to {} for __ac_nonce failed with status: {}",
            room_url,
            response.status()
        ));
    }

    let mut found_cookie_value: Option<String> = None;
    for cookie in response.cookies() {
        if cookie.name() == "__ac_nonce" {
            found_cookie_value = Some(cookie.value().to_string());
            break;
        }
    }
    found_cookie_value
        .ok_or_else(|| format!("__ac_nonce cookie not found in response from {}", room_url))
}

// This is the public function your live.rs will call
pub async fn setup_douyin_cookies(
    http_client: &mut HttpClient,
    room_id: &str,
) -> Result<(), String> {
    let room_url = format!("{}{}", DOUYIN_BASE_URL, room_id);

    let initial_cookies = fetch_initial_cookies(&room_url).await?;
    let ttwid = initial_cookies
        .ttwid
        .ok_or_else(|| "ttwid not found after fetch_initial_cookies".to_string())?;
    let odin_tt_val = initial_cookies.odin_tt.as_deref().unwrap_or_default(); // Use empty if not found, or handle error

    // Correctly handle the Result for ac_nonce
    let mut actual_ac_nonce_to_use = String::new();
    match fetch_ac_nonce(&room_url).await {
        Ok(nonce_value) => {
            if !nonce_value.is_empty() {
                actual_ac_nonce_to_use = nonce_value;
            } else {
                println!("[Cookies WARN] Fetched __ac_nonce was empty. Proceeding without it.");
            }
        }
        Err(e) => {
            eprintln!(
                "[Cookies WARN] Failed to fetch __ac_nonce: {}. Proceeding without it.",
                e
            );
            // actual_ac_nonce_to_use remains String::new()
        }
    }

    let mut cookie_parts = vec![format!("ttwid={}", ttwid)];
    if !odin_tt_val.is_empty() {
        cookie_parts.push(format!("odin_tt={}", odin_tt_val));
    }
    if !actual_ac_nonce_to_use.is_empty() {
        cookie_parts.push(format!("__ac_nonce={}", actual_ac_nonce_to_use));
    }

    let cookie_value = cookie_parts.join("; ");

    http_client.insert_header(COOKIE, &cookie_value)?;

    Ok(())
}
