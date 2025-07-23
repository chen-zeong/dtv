use actix_web::{dev::ServerHandle, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
// use tokio::sync::Mutex as TokioMutex; // Not used here
use futures_util::TryStreamExt;
use reqwest::Client; // Changed from awc::Client
use std::sync::Mutex as StdMutex;
use tauri::{AppHandle, State}; // Removed unused Manager and async_runtime // For map_err on the stream

// Changed path: StreamUrlStore is now at the crate root (main.rs)
use crate::StreamUrlStore;

// Define a struct to hold the server handle in a Tauri managed state
#[derive(Default)]
pub struct ProxyServerHandle(pub StdMutex<Option<ServerHandle>>);

async fn find_free_port() -> u16 {
    // Using a fixed port as requested by the user for easier debugging
    34719
}

// Your actual proxy logic - this is a simplified placeholder
async fn flv_proxy_handler(
    _req: HttpRequest,
    stream_url_store: web::Data<StreamUrlStore>,
    client: web::Data<Client>, // Changed to reqwest::Client
) -> impl Responder {
    let url = stream_url_store.url.lock().unwrap().clone();
    if url.is_empty() {
        return HttpResponse::NotFound().body("Stream URL is not set or empty.");
    }

    match client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send()
        .await
    {
        Ok(upstream_response) => {
            if upstream_response.status().is_success() {
                let mut response_builder = HttpResponse::Ok();
                response_builder.content_type("video/x-flv");

                
                let byte_stream = upstream_response.bytes_stream().map_err(|e| {
                    eprintln!("[Rust/proxy.rs handler] Error reading bytes from upstream: {}", e);
                    actix_web::error::ErrorInternalServerError(format!("Upstream stream error: {}", e))
                });
                
                response_builder.streaming(byte_stream)

            } else {
                let status_from_reqwest = upstream_response.status(); // Renamed for clarity
                let error_text = upstream_response.text().await.unwrap_or_else(|e| {
                    format!("Failed to read error body from upstream: {}", e)
                });
                eprintln!(
                    "[Rust/proxy.rs handler] Upstream request to {} failed with status: {}. Body: {}",
                    url,
                    status_from_reqwest, // Use the renamed variable for logging
                    error_text
                );
                // Convert reqwest::StatusCode to actix_web::http::StatusCode
                let actix_status_code = actix_web::http::StatusCode::from_u16(status_from_reqwest.as_u16())
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);

                HttpResponse::build(actix_status_code).body(format!(
                    "Error fetching FLV stream from upstream (reqwest): {}. Status: {}. Details: {}",
                    url,
                    status_from_reqwest, // Log original reqwest status code
                    error_text
                ))
            }
        }
        Err(e) => {
            eprintln!(
                "[Rust/proxy.rs handler] Failed to send request to upstream {} with reqwest: {}",
                url,
                e
            );
            HttpResponse::InternalServerError().body(format!(
                "Error connecting to upstream FLV stream {} with reqwest: {}",
                url,
                e
            ))
        }
    }
}

#[tauri::command]
pub async fn start_proxy(
    _app_handle: AppHandle,
    server_handle_state: State<'_, ProxyServerHandle>,
    stream_url_store: State<'_, StreamUrlStore>,
) -> Result<String, String> {
    let port = find_free_port().await;
    let current_stream_url = stream_url_store.url.lock().unwrap().clone();

    if current_stream_url.is_empty() {
        return Err("Stream URL is not set in store. Cannot start proxy.".to_string());
    }

    // stream_url_data_for_actix can be created once and cloned, as StreamUrlStore is Arc based and Send + Sync
    let stream_url_data_for_actix = web::Data::new(stream_url_store.inner().clone());
    // REMOVED: let awc_client_for_actix = web::Data::new(Client::default());

    // Ensure MutexGuard is dropped before .await
    let existing_handle_to_stop = { server_handle_state.0.lock().unwrap().take() };
    if let Some(existing_handle) = existing_handle_to_stop {
        existing_handle.stop(false).await;
    }

    let server = match HttpServer::new(move || {
        let app_data_stream_url = stream_url_data_for_actix.clone();
        // Create reqwest::Client inside the closure for each worker thread
        let app_data_reqwest_client = web::Data::new(Client::new()); // Changed to reqwest::Client
        App::new()
            .app_data(app_data_stream_url)
            .app_data(app_data_reqwest_client) // Provide reqwest client
            .wrap(actix_cors::Cors::permissive())
            .route("/live.flv", web::get().to(flv_proxy_handler))
    })
    .bind(("127.0.0.1", port))
    {
        Ok(srv) => srv,
        Err(e) => {
            let err_msg = format!(
                "[Rust/proxy.rs] Failed to bind server to port {}: {}",
                port, e
            );
            eprintln!("{}", err_msg);
            return Err(err_msg);
        }
    }
    .run();

    let server_handle_for_state = server.handle();
    *server_handle_state.0.lock().unwrap() = Some(server_handle_for_state);

    // Use tauri::async_runtime::spawn directly
    tauri::async_runtime::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("[Rust/proxy.rs] Proxy server run error: {}", e);
        } else {
            println!("[Rust/proxy.rs] Proxy server on port {} shut down.", port);
        }
    });

    let proxy_url = format!("http://127.0.0.1:{}/live.flv", port);
    Ok(proxy_url)
}

#[tauri::command]
pub async fn stop_proxy(server_handle_state: State<'_, ProxyServerHandle>) -> Result<(), String> {
    // Ensure MutexGuard is dropped before .await
    let handle_to_stop = { server_handle_state.0.lock().unwrap().take() };

    if let Some(handle) = handle_to_stop {
        handle.stop(false).await; // Changed to non-graceful shutdown
        println!("[Rust/proxy.rs] stop_proxy: Initiated non-graceful shutdown.");
    } else {
        println!("[Rust/proxy.rs] stop_proxy command: No proxy server was running or handle already taken.");
    }
    Ok(())
}
