use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::client::IntoClientRequest};
use futures_util::StreamExt;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub async fn connect_with_retry<F, Fut>(url: &str, mut handler: F) 
where 
    F: FnMut(Message) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut backoff = Duration::from_millis(0);
    let max_backoff = Duration::from_secs(30);

    loop {
        info!("Connecting to WebSocket: {}", url);
        
        let mut request = url.into_client_request().unwrap();
        request.headers_mut().insert("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".parse().unwrap());
        request.headers_mut().insert("Origin", "https://polymarket.com".parse().unwrap());

        match connect_async(request).await {
            Ok((mut ws_stream, _)) => {
                info!("Connected to {}", url);
                backoff = Duration::from_millis(0); // Reset backoff on success

                while let Some(msg) = ws_stream.next().await {
                    match msg {
                        Ok(m) => handler(m).await,
                        Err(e) => {
                            error!("WebSocket stream error for {}: {}", url, e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to connect to {}: {}", url, e);
            }
        }

        warn!("Rescheduling connection to {} in {:?}", url, backoff);
        sleep(backoff).await;
        backoff = std::cmp::min(backoff * 2 + Duration::from_millis(100), max_backoff);
    }
}
