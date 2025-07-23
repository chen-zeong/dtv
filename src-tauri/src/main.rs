// 在开发模式下允许控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use reqwest;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

// mod douyu; // Removed old direct module
mod platforms;
mod proxy; // Added platforms module
           // use platforms::douyu; // No longer need this specific use if functions are directly available via platforms::douyu::* from main

// Assuming API commands are correctly re-exported or defined in these modules
use platforms::common::DouyinDanmakuState;
use platforms::douyin::danmu::signature::generate_douyin_ms_token;
use platforms::douyin::fetch_douyin_partition_rooms;
use platforms::douyin::fetch_douyin_room_info;
use platforms::douyin::fetch_douyin_streamer_info;
use platforms::douyin::get_douyin_live_stream_url;
use platforms::douyin::start_douyin_danmu_listener;
use platforms::douyu::fetch_categories;
use platforms::douyu::fetch_douyu_room_info;
use platforms::douyu::fetch_three_cate;
use platforms::douyu::{fetch_live_list, fetch_live_list_for_cate3};
// get_stream_url and search_anchor will be directly available via platforms::douyu now

#[derive(Default, Clone)]
pub struct StreamUrlStore {
    pub url: Arc<Mutex<String>>,
}

// State for managing Douyu danmaku listener handles (stop signals)
#[derive(Default, Clone)]
pub struct DouyuDanmakuHandles(Arc<Mutex<HashMap<String, oneshot::Sender<()>>>>);

// DanmakuState remains for the danmaku listener
// struct DanmakuState(Mutex<Option<mpsc::Sender<()>>>); // Old Douyu state, to be replaced by DouyuDanmakuHandles

// DouyinDanmakuState is already defined in and re-exported by platforms::common::types
// No need to redefine it here if it's correctly imported.

// This is the command that should be used for getting stream URL if it interacts with StreamUrlStore
#[tauri::command]
async fn get_stream_url_cmd(room_id: String) -> Result<String, String> {
    // Call the actual function to fetch the stream URL from the new location
    platforms::douyu::get_stream_url(&room_id)
        .await
        .map_err(|e| {
            eprintln!(
                "[Rust Error] Failed to get stream URL for room {}: {}",
                room_id,
                e.to_string()
            );
            format!("Failed to get stream URL: {}", e.to_string())
        })
}

// This is the command that should be used for setting stream URL if it interacts with StreamUrlStore
#[tauri::command]
async fn set_stream_url_cmd(
    url: String,
    state: tauri::State<'_, StreamUrlStore>,
) -> Result<(), String> {
    let mut current_url = state.url.lock().unwrap();
    *current_url = url;
    Ok(())
}

// Command to start Douyu danmaku listener
#[tauri::command]
async fn start_danmaku_listener(
    room_id: String,
    window: tauri::Window,
    danmaku_handles: tauri::State<'_, DouyuDanmakuHandles>,
) -> Result<(), String> {
    // If a listener for this room_id already exists, stop it first.
    if let Some(existing_sender) = danmaku_handles.0.lock().unwrap().remove(&room_id) {
        let _ = existing_sender.send(());
    }

    let (stop_tx, stop_rx) = oneshot::channel();
    danmaku_handles
        .0
        .lock()
        .unwrap()
        .insert(room_id.clone(), stop_tx);

    let window_clone = window.clone();
    let room_id_clone = room_id.clone();
    tokio::spawn(async move {
        let mut client = platforms::douyu::danmu_start::DanmakuClient::new(
            &room_id_clone,
            window_clone,
            stop_rx, // Pass the receiver part of the oneshot channel
        );
        if let Err(e) = client.start().await {
            eprintln!(
                "[Rust Main] Douyu danmaku client for room {} failed: {}",
                room_id_clone, e
            );
        }
    });

    Ok(())
}

// Command to stop Douyu danmaku listener
#[tauri::command]
async fn stop_danmaku_listener(
    room_id: String,
    danmaku_handles: tauri::State<'_, DouyuDanmakuHandles>,
) -> Result<(), String> {
    if let Some(sender) = danmaku_handles.0.lock().unwrap().remove(&room_id) {
        match sender.send(()) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "Failed to stop Douyu danmaku listener for room {}: receiver dropped.",
                room_id
            )),
        }
    } else {
        Ok(())
    }
}

// search_anchor seems fine, assuming douyu::search_anchor is correct
#[tauri::command]
async fn search_anchor(keyword: String) -> Result<String, String> {
    platforms::douyu::perform_anchor_search(&keyword)
        .await
        .map_err(|e| e.to_string())
}

// Main function corrected
fn main() {
    // Create a new HTTP client instance to be managed by Tauri
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()
        .expect("Failed to create reqwest client");

    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .manage(client) // Manage the reqwest client
        // .manage(DanmakuState(Mutex::new(None))) // Old Douyu state, remove this
        .manage(DouyuDanmakuHandles::default()) // Manage new DouyuDanmakuHandles
        .manage(DouyinDanmakuState::default()) // Manage DouyinDanmakuState
        .manage(StreamUrlStore::default())
        .manage(proxy::ProxyServerHandle::default())
        .invoke_handler(tauri::generate_handler![
            get_stream_url_cmd,
            set_stream_url_cmd,
            search_anchor,
            start_danmaku_listener,      // Douyu danmaku start
            stop_danmaku_listener,       // Douyu danmaku stop
            start_douyin_danmu_listener, // Added Douyin danmaku listener command
            proxy::start_proxy,
            proxy::stop_proxy,
            fetch_categories,
            fetch_live_list,
            fetch_live_list_for_cate3,
            fetch_douyu_room_info,
            fetch_three_cate,
            generate_douyin_ms_token,
            fetch_douyin_partition_rooms,
            get_douyin_live_stream_url,
            fetch_douyin_room_info,
            fetch_douyin_streamer_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
