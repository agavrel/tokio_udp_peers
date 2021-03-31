use std::{collections::BTreeSet, time::Duration};
use tokio::{sync::mpsc, task, time, net::{TcpStream, TcpListener}, io::{AsyncWriteExt, AsyncReadExt}}; // 1.4.0

const ADDRESS: &str = "127.0.0.1:8775";

#[tokio::main]
async fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("server") => server().await,
        _ => client().await,
    }
}

async fn server() {
    eprintln!("Starting the server");

    let listener = TcpListener::bind(ADDRESS).await.expect("Unable to bind");

    while let Ok((socket, _)) = listener.accept().await {
        let (mut network_rx, mut network_tx) = socket.into_split();
        let (debounce_tx, mut debounce_rx) = mpsc::channel(10);

        // Listen for events
        let debouncer = task::spawn(async move {
            let mut ids = BTreeSet::new();
            let duration = Duration::from_millis(10);

            loop {
                match time::timeout(duration, debounce_rx.recv()).await {
                    Ok(Some(id)) => {
                        ids.insert(id);
                    }
                    Ok(None) => {
                        eprintln!("Sending: {:?}", ids);
                        for &v in &ids {
                            network_tx.write_i32(v).await.expect("Unable to send ID");
                        }
                        break;
                    }
                    Err(_) => {
                        eprintln!("Sending: {:?}", ids);
                        for &v in &ids {
                            network_tx.write_i32(v).await.expect("Unable to send ID");
                        }
                        ids.clear();
                    }
                }
            }
        });

        // Listen for network activity
        let server = task::spawn({
            let debounce_tx = debounce_tx.clone();
            async move {
                while let Ok(id) = network_rx.read_i32().await {
                    debounce_tx
                        .send(id)
                        .await
                        .expect("Unable to talk to debounce");
                }
            }
        });

        // Prevent deadlocks
        drop(debounce_tx);

        // Wait for everything to finish
        server.await.expect("Server panicked");
        debouncer.await.expect("Debouncer panicked");
    }

    eprintln!("Server done");
}

async fn client() {
    eprintln!("Starting the client");

    let mut connection = TcpStream::connect(ADDRESS).await.expect("Unable to connect");

    connection.write_i32(1).await.expect("Unable to talk to network");
    connection.write_i32(2).await.expect("Unable to talk to network");
    connection.write_i32(3).await.expect("Unable to talk to network");

    time::sleep(Duration::from_millis(20)).await;

    connection.write_i32(4).await.expect("Unable to talk to network");
    connection.write_i32(5).await.expect("Unable to talk to network");
    connection.write_i32(6).await.expect("Unable to talk to network");

    time::sleep(Duration::from_millis(20)).await;

    eprintln!("Client done");
}
