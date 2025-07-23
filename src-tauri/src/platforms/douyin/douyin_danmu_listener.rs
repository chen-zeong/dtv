use tauri::Emitter;
use tokio::sync::mpsc as tokio_mpsc;

// Assuming DouyinDanmakuState is defined in main.rs (crate root)
// and common::GetStreamUrlPayload is accessible.

#[tauri::command]
pub async fn start_douyin_danmu_listener(
    payload: crate::platforms::common::GetStreamUrlPayload,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, crate::platforms::common::DouyinDanmakuState>,
) -> Result<(), String> {
    let room_id_or_url = payload.args.room_id_str;
    println!(
        "[Douyin Danmaku] Received request for room_id_or_url: {}",
        room_id_or_url
    );

    let previous_tx = {
        let mut lock = state.inner().0.lock().unwrap();
        lock.take()
    };

    if let Some(tx) = previous_tx {
        println!("[Douyin Danmaku] Sending shutdown to previous Douyin listener task.");
        if tx.send(()).await.is_err() {
            eprintln!("[Douyin Danmaku] Failed to send shutdown. Task might have already completed or panicked.");
        }
    }

    if room_id_or_url == "stop_listening" {
        println!(
            "[Douyin Danmaku] Received stop_listening signal. Listener will not be restarted."
        );
        return Ok(());
    }

    let (tx_shutdown, mut rx_shutdown) = tokio_mpsc::channel::<()>(1);
    {
        let mut lock = state.inner().0.lock().unwrap();
        *lock = Some(tx_shutdown);
    }

    let app_handle_clone = app_handle.clone();
    let room_id_str_clone = room_id_or_url.clone();

    tokio::spawn(async move {
        println!(
            "[Douyin Danmaku] Spawning listener for room: {}",
            room_id_str_clone
        );

        let task_result = async {
            let mut fetcher = crate::platforms::douyin::danmu::web_fetcher::DouyinLiveWebFetcher::new(&room_id_str_clone)?;
            fetcher.fetch_room_details().await.map_err(|e| format!("Failed to fetch room details: {}", e))?;
            
            let actual_room_id = fetcher.get_room_id().await?;
            let ttwid = fetcher.get_ttwid().await?;
            
            println!("[Douyin Danmaku] Using: room_id={}, ttwid={}", actual_room_id, ttwid);

            let (read_stream, ack_tx) = 
                crate::platforms::douyin::danmu::websocket_connection::connect_and_manage_websocket(
                    &fetcher, 
                    &actual_room_id, 
                    &ttwid
                ).await?;
            
            println!("[Douyin Danmaku] WebSocket connected for room: {}", actual_room_id);

            tokio::select! {
                res = crate::platforms::douyin::danmu::message_handler::handle_received_messages(
                    read_stream, 
                    ack_tx, 
                    app_handle_clone.clone(),
                    actual_room_id.clone()
                ) => {
                    if let Err(e) = res {
                        return Err(e); 
                    }
                }
                _ = rx_shutdown.recv() => {
                    println!("[Douyin Danmaku] Received shutdown signal for room {}.", actual_room_id);
                }
            }
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        }.await;

        if let Err(e) = task_result {
            eprintln!(
                "[Douyin Danmaku] Listener task for room {} critically failed: {}",
                room_id_str_clone, e
            );
            let error_payload =
                crate::platforms::douyin::danmu::message_parsers::DanmakuFrontendPayload {
                    room_id: room_id_str_clone.clone(),
                    user: "系统消息".to_string(),
                    content: format!("弹幕连接发生错误: {}", e),
                    user_level: 0,
                    fans_club_level: 0,
                };
            if let Err(emit_err) = app_handle.emit("danmaku-message", error_payload) {
                eprintln!(
                    "[Douyin Danmaku] Failed to emit error event to frontend: {}",
                    emit_err
                );
            }
        } else {
            println!(
                "[Douyin Danmaku] Listener task for room {} completed.",
                room_id_str_clone
            );
        }
    });
    Ok(())
}
