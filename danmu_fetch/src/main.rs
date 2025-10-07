// src/main.rs
use std::env;
use std::time::Duration;

use danmu_fetch::models::BiliMessage;
use danmu_fetch::websocket::BiliLiveClient;

fn main() {
    // Parse CLI args
    let args: Vec<String> = env::args().skip(1).collect();

    // Determine room id: first numeric positional arg, or env ROOM_ID, or default
    let room_id = args
        .iter()
        .find(|s| s.chars().all(|c| c.is_ascii_digit()))
        .cloned()
        .or_else(|| env::var("ROOM_ID").ok())
        .unwrap_or_else(|| "26808337".to_string());

    // Determine cookie: env var BILI_COOKIE first, then CLI --cookie or --cookie=<str>
    let mut cookie_opt = env::var("BILI_COOKIE").ok();
    if cookie_opt.is_none() {
        // support --cookie <str>
        if let Some(pos) = args.iter().position(|s| s == "--cookie") {
            if let Some(cookie_val) = args.get(pos + 1) {
                cookie_opt = Some(cookie_val.clone());
            }
        }
        // support --cookie=<str>
        if cookie_opt.is_none() {
            if let Some(cookie_arg) = args.iter().find(|s| s.starts_with("--cookie=")) {
                let val = cookie_arg.trim_start_matches("--cookie=").to_string();
                if !val.is_empty() {
                    cookie_opt = Some(val);
                }
            }
        }
    }

    let mut client = match cookie_opt {
        Some(cookie) => {
            // Do NOT print cookie for security
            BiliLiveClient::new_with_cookie(&cookie, &room_id)
        }
        None => BiliLiveClient::new_without_cookie(&room_id),
    };

    // Send auth message after establishing connection
    client.send_auth();

    loop {
        if let Some(msg) = client.read_once() {
            match msg {
                BiliMessage::Danmu { user, text } => {
                    println!("[Danmu] {}: {}", user, text);
                }
                BiliMessage::Gift { .. } => {
                    // suppress
                }
                BiliMessage::Unsupported { .. } => {
                    // suppress
                }
            }
        }
        // Small sleep to avoid busy spinning
        std::thread::sleep(Duration::from_millis(50));
    }
}