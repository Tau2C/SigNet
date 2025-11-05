use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use serde_json;
use signet::Frame;
use std::fs;
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Address of the broker
    #[arg(short, long, default_value = "ws://127.0.0.1:8081/")]
    broker: String,

    /// Keyfile for authentication
    #[arg(short, long)]
    keyfile: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let url = url::Url::parse(&args.broker)?;

    let key_data = match fs::read_to_string(&args.keyfile) {
        Ok(s) => s.trim().to_string(),
        Err(e) => {
            eprintln!("Failed to read keyfile '{}': {}", args.keyfile, e);
            std::process::exit(1);
        }
    };

    println!("{}", key_data);

    loop {
        println!("Attempting to connect to broker...");
        match tokio_tungstenite::connect_async(url.clone()).await {
            Ok((mut ws_stream, _)) => {
                println!("WebSocket connection established to broker.");

                // Prepare a Register frame
                let frame = Frame::Register {
                    id: key_data.clone(),
                };

                // Serialize and send it
                let txt = serde_json::to_string(&frame)?;
                ws_stream.send(Message::Text(txt)).await?;

                println!("Sent Register frame to broker.");

                // Read messages from the broker
                while let Some(message) = ws_stream.next().await {
                    match message {
                        Ok(msg) => match msg {
                            Message::Text(txt) => {
                                // Attempt to deserialize incoming message
                                match serde_json::from_str::<Frame>(&txt) {
                                    Ok(frame) => {
                                        println!("Received frame: {:?}", frame);
                                        // Handle each frame type
                                        match frame {
                                            Frame::Open {
                                                conn_id,
                                                target,
                                                port,
                                            } => {
                                                println!(
                                                    "Open request for {}:{} (conn_id={})",
                                                    target, port, conn_id
                                                );
                                            }
                                            Frame::Data { conn_id, data } => {
                                                println!(
                                                    "Data for {}: {}",
                                                    conn_id,
                                                    data.escape_debug()
                                                );
                                            }
                                            Frame::Close { conn_id } => {
                                                println!("Close connection {}", conn_id);
                                            }
                                            Frame::Register { id } => {
                                                println!(
                                                    "Broker acknowledged registration as {}",
                                                    id
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Invalid frame JSON: {} — raw: {}", e, txt);
                                    }
                                }
                            }
                            Message::Close(Some(close_frame)) => {
                                if close_frame.code == CloseCode::Restart {
                                    println!("Broker restarting, reconnecting soon...");
                                    break;
                                } else if close_frame.code == CloseCode::Normal {
                                    println!("Broker shutting down. Exiting.");
                                    return Ok(());
                                } else {
                                    println!("Broker closed connection: {:?}", close_frame);
                                    return Ok(());
                                }
                            }
                            Message::Close(None) => {
                                println!("Broker disconnected.");
                                break;
                            }
                            _ => {}
                        },
                        Err(e) => {
                            eprintln!("WebSocket error: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to connect to broker: {}. Retrying in 5 seconds...",
                    e
                );
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}
