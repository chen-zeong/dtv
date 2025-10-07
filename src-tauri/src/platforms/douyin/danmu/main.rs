use tokio;
use std::time::Duration;

mod signature;
mod web_fetcher;
mod websocket_connection;
mod message_handler;
mod message_parsers;

// Protobuf generated code (douyin module)
pub mod douyin {
    include!(concat!(env!("OUT_DIR"), "/douyin.rs"));
}

// TEMPORARILY KEEP connect_websocket here. It will be the next major piece to move.
async fn connect_and_process_websocket(
    fetcher: &web_fetcher::DouyinLiveWebFetcher,
    room_id: &str,
    cookie_header: &str,
    user_unique_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (read_stream, ack_tx) = 
        websocket_connection::connect_and_manage_websocket(fetcher, room_id, cookie_header, user_unique_id).await?;

    message_handler::handle_received_messages(read_stream, ack_tx).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let live_id = "427138916527";
    let mut fetcher = web_fetcher::DouyinLiveWebFetcher::new(live_id)?;
    
    fetcher.fetch_room_details().await?;
    
    let room_id = fetcher.get_room_id().await?;
    let cookie_header = fetcher.get_dy_cookie().await?;
    let user_unique_id = fetcher.get_user_unique_id().await?;
    connect_and_process_websocket(&fetcher, &room_id, &cookie_header, &user_unique_id).await?;

    tokio::time::sleep(Duration::from_secs(60)).await;

    Ok(())
}
