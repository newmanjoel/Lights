use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::io::{AsyncWriteExt, BufWriter};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Create a broadcast channel
    let (tx, _) = broadcast::channel(100);
    let tx = Arc::new(tx);

    // Spawn the lighting component
    let tx_clone = Arc::clone(&tx);
    tokio::spawn(async move {
        loop {
            // Simulate sending data to the feedback system
            let _ = tx_clone.send("Lighting status update".to_string());
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    // Set up the TCP listener
    let listener = TcpListener::bind("127.0.0.1:8080").await.expect("Failed to bind to address");
    println!("Server listening on 127.0.0.1:8080");

    loop {
        // Accept incoming connections
        if let Ok((socket, _)) = listener.accept().await {
            let tx_clone = Arc::clone(&tx);
            tokio::spawn(handle_connection(socket, tx_clone));
        }
    }
}

async fn handle_connection(mut socket: TcpStream, tx: Arc<broadcast::Sender<String>>) {
    // Create a receiver for the broadcast channel
    let mut rx = tx.subscribe();

    let mut writer = BufWriter::new(socket);

    loop {
        match rx.recv().await {
            Ok(message) => {
                // Send the message to the client
                if writer.write_all(message.as_bytes()).await.is_err() {
                    println!("Failed to send data to client, closing connection");
                    break;
                }
                if writer.write_all(b"\n").await.is_err() {
                    break;
                }
                writer.flush().await.expect("Failed to flush data");
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {
                println!("Client lagged and missed messages");
            }
            Err(_) => {
                println!("Channel closed, stopping connection");
                break;
            }
        }
    }
}
