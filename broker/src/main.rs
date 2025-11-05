// broker.rs
use futures_util::{SinkExt, StreamExt};
use ring::signature;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, path};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, mpsc};
use tokio_tungstenite::tungstenite::Message;
use webpki::{EndEntityCert, Time, TrustAnchor};
use x509_parser::prelude::*;

use signet::{self, Frame};

type Tx = mpsc::UnboundedSender<Message>;
type Clients = Arc<Mutex<HashMap<String, Tx>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "0.0.0.0";
    let port = "8081";
    let listener = TcpListener::bind(format!("{}:{}", addr, port)).await?;
    println!("Broker listening on {}", addr);

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((stream, _)) = listener.accept().await {
        let clients = clients.clone();
        tokio::spawn(async move {
            let ws = match tokio_tungstenite::accept_async(stream).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("ws handshake err: {}", e);
                    return;
                }
            };
            println!("New ws connection");

            let (mut ws_tx, mut ws_rx) = ws.split();
            // create outbound queue so other tasks can send to this client
            let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
            // spawn writer task
            let write_task = tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    if ws_tx.send(msg).await.is_err() {
                        break;
                    }
                }
            });

            // We need the registered id to store in map
            let mut my_id: Option<String> = None;

            while let Some(msg) = ws_rx.next().await {
                match msg {
                    Ok(Message::Text(t)) => {
                        match serde_json::from_str::<Frame>(&t) {
                            Ok(frame) => {
                                match frame {
                                    Frame::Register { id: client_pem } => {
                                        // Load CA certificate
                                        let path = path::absolute("ca/ca.crt").unwrap();
                                        println!("{}", path.to_str().unwrap());
                                        let ca_pem_bytes = match fs::read(path) {
                                            Ok(p) => p,
                                            Err(e) => {
                                                eprintln!("Failed to read CA certificate: {}", e);
                                                return;
                                            }
                                        };
                                        let ca_pem = match pem::parse_x509_pem(&ca_pem_bytes) {
                                            Ok(p) => p,
                                            Err(e) => {
                                                eprintln!("Failed to parse CA PEM: {:?}", e);
                                                return;
                                            }
                                        };
                                        let ca_anchor = match webpki::TrustAnchor::try_from_cert_der(
                                            &ca_pem.1.contents,
                                        ) {
                                            Ok(a) => a,
                                            Err(e) => {
                                                eprintln!("Failed to parse CA DER: {:?}", e);
                                                return;
                                            }
                                        };
                                        let anchors = webpki::TlsClientTrustAnchors(&[ca_anchor]);

                                        // Load client certificate
                                        let client_pem_bytes = client_pem.as_bytes();
                                        let client_pem = match pem::parse_x509_pem(client_pem_bytes)
                                        {
                                            Ok(p) => p,
                                            Err(e) => {
                                                eprintln!("Failed to parse client PEM: {:?}", e);
                                                return;
                                            }
                                        };
                                        let client_cert = match webpki::EndEntityCert::try_from(
                                            &client_pem.1.contents[..],
                                        ) {
                                            Ok(c) => c,
                                            Err(e) => {
                                                eprintln!("Failed to parse client DER: {:?}", e);
                                                return;
                                            }
                                        };

                                        // Convert current time
                                        let now = match SystemTime::now().duration_since(UNIX_EPOCH)
                                        {
                                            Ok(dur) => {
                                                Time::from_seconds_since_unix_epoch(dur.as_secs())
                                            }
                                            Err(_) => {
                                                eprintln!("System time is before UNIX epoch");
                                                return;
                                            }
                                        };

                                        let algs: &[&webpki::SignatureAlgorithm] =
                                            &[&webpki::ECDSA_P256_SHA256]; // supported EC algorithms

                                        match client_cert.verify_is_valid_tls_client_cert(
                                            algs,     // supported signature algorithms
                                            &anchors, // TlsClientTrustAnchors
                                            &[],      // no intermediates
                                            now,      // current webpki::Time
                                        ) {
                                            Ok(_) => {
                                                println!(
                                                    "Client certificate verified: registration accepted"
                                                );
                                                let id = uuid::Uuid::new_v4().to_string();
                                                my_id = Some(id.clone());
                                                clients.lock().await.insert(id.clone(), tx.clone());

                                                // Acknowledge registration
                                                let ack = Frame::Register { id };
                                                if let Ok(txt) = serde_json::to_string(&ack) {
                                                    let _ = tx.send(Message::Text(txt));
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!(
                                                    "Client certificate not signed by trusted CA: {:?}",
                                                    e
                                                );
                                                return;
                                            }
                                        }
                                    }

                                    // Only allow these frames if the client has registered
                                    Frame::Open {
                                        conn_id,
                                        target,
                                        port,
                                    } => {
                                        if my_id.is_none() {
                                            eprintln!(
                                                "Client sent Open frame before registration. Ignoring."
                                            );
                                            continue;
                                        }
                                        println!("Open request {} -> {}:{}", conn_id, target, port);

                                        if let Some(dest) = clients.lock().await.get(&target) {
                                            let f = Frame::Open {
                                                conn_id,
                                                target: "".into(),
                                                port,
                                            };
                                            if let Ok(s) = serde_json::to_string(&f) {
                                                let _ = dest.send(Message::Text(s));
                                            }
                                        } else {
                                            eprintln!("Target {} not found", target);
                                        }
                                    }

                                    Frame::Data { conn_id, data } => {
                                        if my_id.is_none() {
                                            eprintln!(
                                                "Client sent Data frame before registration. Ignoring."
                                            );
                                            continue;
                                        }
                                        if let Some((_, to, _)) = parse_conn(&conn_id) {
                                            if let Some(dest) = clients.lock().await.get(&to) {
                                                let f = Frame::Data {
                                                    conn_id: conn_id.clone(),
                                                    data,
                                                };
                                                if let Ok(s) = serde_json::to_string(&f) {
                                                    let _ = dest.send(Message::Text(s));
                                                }
                                            }
                                        }
                                    }

                                    Frame::Close { conn_id } => {
                                        if my_id.is_none() {
                                            eprintln!(
                                                "Client sent Close frame before registration. Ignoring."
                                            );
                                            continue;
                                        }
                                        if let Some((_, to, _)) = parse_conn(&conn_id) {
                                            if let Some(dest) = clients.lock().await.get(&to) {
                                                let f = Frame::Close {
                                                    conn_id: conn_id.clone(),
                                                };
                                                if let Ok(s) = serde_json::to_string(&f) {
                                                    let _ = dest.send(Message::Text(s));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => eprintln!("bad frame: {} | {}", e, t),
                        }
                    }
                    Ok(Message::Close(_)) | Err(_) => break,
                    _ => {}
                }
            }

            // cleanup
            if let Some(id) = my_id {
                clients.lock().await.remove(&id);
                println!("Client {} disconnected", id);
            }

            let _ = write_task.await;
        });
    }

    Ok(())
}

fn parse_conn(conn: &str) -> Option<(String, String, String)> {
    // expected "<from>::<to>::<uuid>"
    let parts: Vec<&str> = conn.split("::").collect();
    if parts.len() == 3 {
        Some((parts[0].into(), parts[1].into(), parts[2].into()))
    } else {
        None
    }
}
