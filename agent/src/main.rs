use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = url::Url::parse("ws://127.0.0.1:8081/")?;

    let (mut ws_stream, _) = tokio_tungstenite::connect_async(url).await?;

    println!("WebSocket connection established to broker.");

    // Send a message to the broker
    ws_stream.send(Message::Text("Hello from agent!".into())).await?;
    println!("Sent 'Hello from agent!' to broker.");

    // Read messages from the broker
    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(msg) => {
                if msg.is_text() || msg.is_binary() {
                    println!("Received message from broker: {:?}", msg);
                } else if msg.is_close() {
                    println!("Broker disconnected.");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving message from broker: {}", e);
                break;
            }
        }
    }

    Ok(())
}