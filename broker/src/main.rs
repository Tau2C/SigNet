use clap::Parser;
use figment::{
    providers::{Env, Format, Toml, Serialized},
    Figment, Provider, Profile, Metadata, Error,
};
use figment::value::{Dict, Map};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Address to listen on
    #[arg(short, long)]
    address: Option<String>,
}

impl Provider for Cli {
    fn metadata(&self) -> Metadata {
        Metadata::named("Command-Line Arguments")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let mut data = Dict::new();
        if let Some(address) = &self.address {
            data.insert("address".into(), address.clone().into());
        }
        Ok(Map::from([(Profile::Default, data)]))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    address: String,
}

impl Default for Config {
    fn default() -> Self {
        Config { address: "0.0.0.0:8081".into() }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config: Config = Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("Broker.toml").nested())
        .merge(Env::prefixed("BROKER_").split("__"))
        .merge(cli)
        .extract()?;

    let listener = TcpListener::bind(&config.address).await?;
    println!("Broker listening on: {}", &config.address);

    let (shutdown_tx, _) = broadcast::channel(1);

    let server_future = async move {
        loop {
            let shutdown_rx = shutdown_tx.subscribe();
            tokio::select! {
                Ok((stream, _)) = listener.accept() => {
                    tokio::spawn(handle_connection(stream, shutdown_rx));
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("\nCtrl+C received, sending shutdown signal.");
                    shutdown_tx.send(()).unwrap();
                    break;
                }
            }
        }
    };

    server_future.await;

    Ok(())
}

async fn handle_connection(raw_stream: TcpStream, mut shutdown_rx: broadcast::Receiver<()>) {
    println!("Incoming TCP connection from: {}", raw_stream.peer_addr().unwrap());

    let mut ws_stream = match tokio_tungstenite::accept_async(raw_stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    println!("WebSocket connection established.");

    loop {
        tokio::select! {
            Some(message) = ws_stream.next() => {
                match message {
                    Ok(msg) => {
                        if msg.is_text() || msg.is_binary() {
                            println!("Received a message: {:?}", msg);
                            // Echo the message back for now
                            if let Err(e) = ws_stream.send(msg).await {
                                eprintln!("Error sending message: {}", e);
                                break;
                            }
                        } else if msg.is_close() {
                            println!("Client disconnected.");
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {}", e);
                        break;
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                println!("Shutdown signal received, closing connection.");
                // To tell the client to reconnect, use CloseCode::Restart
                // To tell the client not to reconnect, use CloseCode::Normal
                let close_frame = CloseFrame {
                    code: CloseCode::Normal,
                    reason: "Server is shutting down".into(),
                };
                if let Err(e) = ws_stream.send(Message::Close(Some(close_frame))).await {
                    eprintln!("Error sending close frame: {}", e);
                }
                break;
            }
        }
    }
}
