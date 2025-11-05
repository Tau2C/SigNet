use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = url::Url::parse("ws://127.0.0.1:8081/")?;

    loop {
        println!("Attempting to connect to broker...");
        match tokio_tungstenite::connect_async(url.clone()).await {
            Ok((mut ws_stream, _)) => {
                println!("WebSocket connection established to broker.");

                // Send a message to the broker
                if let Err(e) = ws_stream.send(Message::Text("Hello from agent!".into())).await {
                    eprintln!("Error sending message: {}", e);
                    continue;
                }
                println!("Sent 'Hello from agent!' to broker.");

                // Read messages from the broker
                while let Some(message) = ws_stream.next().await {
                    match message {
                        Ok(msg) => {
                            if msg.is_text() || msg.is_binary() {
                                println!("Received message from broker: {:?}", msg);
                            } else if let Message::Close(Some(close_frame)) = msg {
                                if close_frame.code == CloseCode::Restart {
                                    println!("Broker is restarting, will reconnect...");
                                    break; // Break from the inner loop to reconnect
                                } else if close_frame.code == CloseCode::Normal {
                                    println!("Broker is shutting down, exiting.");
                                    return Ok(()); // Exit the program
                                } else {
                                    println!("Broker disconnected: {:?}", close_frame);
                                    return Ok(()); // Exit the program
                                }
                            } else if msg.is_close() {
                                println!("Broker disconnected without a specific reason.");
                                break; // Break from the inner loop to reconnect
                            }
                        }
                        Err(e) => {
                            eprintln!("Error receiving message from broker: {}", e);
                            break; // Break from the inner loop to reconnect
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to broker: {}. Retrying in 5 seconds...", e);
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}