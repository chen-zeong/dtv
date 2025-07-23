use chrono::Utc;
use futures_util::{stream::SplitStream, SinkExt, StreamExt};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Sender};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use tokio_tungstenite::{connect_async, MaybeTlsStream};
use urlencoding;
// use url::Url; // REMOVED AGAIN
// use rand::Rng; // REMOVED AGAIN

// Adjusted imports to use `super` for sibling modules within `danmu`
use super::gen::PushFrame; // Removed ::douyin
use super::signature; // For generate_signature
use super::web_fetcher::DouyinLiveWebFetcher;
use prost::Message as ProstMessage; // For encoding heartbeat

// Define a type alias for the WebSocket stream for brevity
pub type WsStream = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;

// This function will establish the connection and spawn send/heartbeat tasks.
// It returns the read half of the stream and the sender for the outgoing message channel.
pub async fn connect_and_manage_websocket(
    fetcher: &DouyinLiveWebFetcher, // Changed to immutable reference as we only read from it now
    room_id: &str,
    ttwid: &str,
) -> Result<(SplitStream<WsStream>, Sender<WsMessage>), Box<dyn std::error::Error + Send + Sync>> {
    let ws_cookie_header = format!("ttwid={}", ttwid);
    let current_timestamp_ms = Utc::now().timestamp_millis();
    let first_req_ms = current_timestamp_ms - 100;
    let cursor = format!(
        "d-1_u-1_fh-7392091211001140287_t-{}_r-1",
        current_timestamp_ms
    );
    let internal_ext_original = format!(
        "internal_src:dim|wss_push_room_id:{}|wss_push_did:7319483754668557238|first_req_ms:{}|fetch_time:{}|seq:1|wss_info:0-{}-0-0|wrds_v:7392094459690748497",
        room_id, first_req_ms, current_timestamp_ms, current_timestamp_ms
    ).replace("\n", "").replace(" ", "");

    let wss_url_str_for_signature = format!(
        "wss://webcast5-ws-web-hl.douyin.com/webcast/im/push/v2/?app_name=douyin_web&version_code=180800&webcast_sdk_version=1.0.14-beta.0&update_version_code=1.0.14-beta.0&compress=gzip&device_platform=web&cookie_enabled=true&screen_width=1536&screen_height=864&browser_language=zh-CN&browser_platform=Win32&browser_name=Mozilla&browser_version=5.0%20(Windows%20NT%2010.0;%20Win64;%20x64)%20AppleWebKit/537.36%20(KHTML,%20like%20Gecko)%20Chrome/126.0.0.0%20Safari/537.36&browser_online=true&tz_name=Asia/Shanghai&cursor={}&internal_ext={}&host=https://live.douyin.com&aid=6383&live_id=1&did_rule=3&endpoint=live_pc&support_wrds=1&user_unique_id=7319483754668557238&im_path=/webcast/im/fetch/&identity=audience&need_persist_msg_count=15&insert_task_id=&live_reason=&room_id={}&heartbeatDuration=0",
        &cursor, 
        &internal_ext_original, 
        room_id
    );
    let signature = signature::generate_signature(&wss_url_str_for_signature).await?;
    let internal_ext_encoded = urlencoding::encode(&internal_ext_original);
    let final_wss_url_str = format!(
        "{}&signature={}",
        format!(
            "wss://webcast5-ws-web-hl.douyin.com/webcast/im/push/v2/?app_name=douyin_web&version_code=180800&webcast_sdk_version=1.0.14-beta.0&update_version_code=1.0.14-beta.0&compress=gzip&device_platform=web&cookie_enabled=true&screen_width=1536&screen_height=864&browser_language=zh-CN&browser_platform=Win32&browser_name=Mozilla&browser_version=5.0%20(Windows%20NT%2010.0;%20Win64;%20x64)%20AppleWebKit/537.36%20(KHTML,%20like%20Gecko)%20Chrome/126.0.0.0%20Safari/537.36&browser_online=true&tz_name=Asia/Shanghai&cursor={}&internal_ext={}&host=https://live.douyin.com&aid=6383&live_id=1&did_rule=3&endpoint=live_pc&support_wrds=1&user_unique_id=7319483754668557238&im_path=/webcast/im/fetch/&identity=audience&need_persist_msg_count=15&insert_task_id=&live_reason=&room_id={}&heartbeatDuration=0",
            &cursor, 
            &internal_ext_encoded, 
            room_id
        ),
        signature
    );

    let mut client_request = final_wss_url_str.into_client_request()?;
    let headers = client_request.headers_mut();
    headers.insert("User-Agent", fetcher.user_agent.parse()?);
    headers.insert("Cookie", ws_cookie_header.parse()?);

    let (ws_stream, _response) = connect_async(client_request).await?;

    let (mut write, read) = ws_stream.split(); // read will be returned

    // Channel for sending messages to the WebSocket Sink
    let (tx, mut rx) = mpsc::channel::<WsMessage>(32);

    // Spawn task for sending heartbeats and other messages from the channel
    tokio::spawn(async move {
        let heartbeat_msg_proto = PushFrame {
            payload_type: "hb".to_string(),
            log_id: 0,
            payload: vec![],
            ..Default::default()
        };
        let mut heartbeat_buf = Vec::new();
        heartbeat_msg_proto.encode(&mut heartbeat_buf).unwrap(); // Assuming encode is from ProstMessage trait
        let ws_ping_msg = WsMessage::Ping(heartbeat_buf);

        let mut ticker = tokio::time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if let Err(e) = write.send(ws_ping_msg.clone()).await {
                        println!("【X】心跳任务发送错误: {}, 连接可能已断开", e);
                        break;
                    }
                }
                Some(msg_to_send) = rx.recv() => {
                    if let Err(e) = write.send(msg_to_send).await {
                        println!("【X】消息发送任务错误: {}, 连接可能已断开", e);
                        // Potentially break or signal error if a crucial message (like ACK) fails
                        if matches!(e, tokio_tungstenite::tungstenite::Error::ConnectionClosed |
                                       tokio_tungstenite::tungstenite::Error::AlreadyClosed) {
                            break; // Stop if connection is closed
                        }
                    }
                }
                else => {
                    // Channel closed, no more messages to send from other tasks
                    break;
                }
            }
        }
        println!("WebSocket send/heartbeat task ended.");
    });

    Ok((read, tx)) // Return the read stream and the sender for other tasks to send messages
}
