use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::{sync::mpsc, task, time}; // 1.4.0
use std::rc::Rc;

const UDP_HEADER: usize = 8;
const IP_HEADER: usize = 20;
const AG_HEADER: usize = 4;
const MAX_DATA_LENGTH: usize = (64 * 1024 - 1) - UDP_HEADER - IP_HEADER;
const MAX_CHUNK_SIZE: usize = MAX_DATA_LENGTH - AG_HEADER;
const MAX_DATAGRAM_SIZE: usize = 0x10000;
const ADDRESS: &str = "127.0.0.1:8080";
const ADDRESS_CLIENT: &str = "127.0.0.1:8000";

#[tokio::main]
async fn main() {
    //server().await;

    match std::env::args().nth(1).as_deref() {
        Some("client") => client().await,
        _ => server().await,
    }
}

async fn server() {
    eprintln!("Starting the server");
        let mut start:bool = false;

    let addr = env::args().nth(1).unwrap_or_else(|| ADDRESS.to_string());
    let socket = UdpSocket::bind(&addr).await.unwrap();
    let arc = Arc::new(socket);
    let mut buf = [0u8; MAX_DATA_LENGTH];
    let (debounce_tx, mut debounce_rx) = mpsc::channel::<Vec<u8>>(MAX_DATAGRAM_SIZE);

    let _debouncer = task::spawn(async move {
        let mut packet_ids: Vec<i32> = Vec::new();
        packet_ids = vec![0; 10];
        let duration = Duration::from_millis(1300);

        'outer: loop {
            match time::timeout(duration, debounce_rx.recv()).await {
                Ok(Some(bytes)) => {
                    let id: u8 = bytes.clone()[0];
                    packet_ids[id as usize] = 1;
                    eprintln!("{} id packet received:{:?}", id, packet_ids);
                    if packet_ids.iter().all(|x| x == &1i32) {
                        println!("All packets have been received, stop program ");
                        start = false;
                        break 'outer;
                    }
                }
                Ok(None) => {
                    eprintln!("Done: {:?}", packet_ids);
                    break;
                }
                Err(_) => {
                    eprintln!("No activity for 1.3sd");
                }
            }
        }
    });
    // Listen for first packet

    let result = arc.clone().recv_from(&mut buf).await;
    match result {
        Ok((len, addr)) => {
            eprintln!("Bytes len: {} from {}", len, addr);
            debounce_tx.send(buf.to_vec()).await.expect("Unable to talk to debounce");
        }
        Err(_) => {
            eprintln!("Couldnt get datagram");
        }
    }
    start = true;
    // listen for other packets
    while (start) {
        let thread_socket = arc.clone();
        let debounce_tx = debounce_tx.clone();
    /*    let _server = task::spawn({


            async move {*/

            eprintln!("runnning");
                if let result = thread_socket.recv_from(&mut buf).await {
                    match result {
                        Ok((len, addr)) => {
                            eprintln!("Bytes len: {} from {}", len, addr);
                            debounce_tx
                                .send(buf.to_vec())
                                .await
                                .expect("Unable to talk to debounce");

                        }
                        Err(_) => {
                            eprintln!("Couldnt get datagram");
                        }
                    }
                }
                // Prevent deadlocks
                drop(debounce_tx);
         /*   }
        });*/
    }
    // Wait for everything to finish
    /* server.await.expect("Server panicked");
       debouncer.await.expect("Debouncer panicked");
    */
}

async fn client() {
    eprintln!("Starting the client");

    let remote_addr: SocketAddr = env::args()
        .nth(2)
        .unwrap_or_else(|| ADDRESS.into()) // cargo run --example udp-client -- 127.0.0.1:8080
        .parse()
        .unwrap();

    // We use port 0 to let the operating system allocate an available port for us.
    let local_addr: SocketAddr = if remote_addr.is_ipv4() {
        ADDRESS_CLIENT // "0.0.0.0:0" //
    } else {
        "[::]:0"
    }
    .parse()
    .unwrap();
    let socket = UdpSocket::bind(ADDRESS_CLIENT).await.unwrap();

    socket.connect(&remote_addr).await.unwrap();

    socket.send(&[0, 2, 3]).await.expect("Unable to talk to network");
    socket.send(&[1, 2, 3]).await.expect("Unable to talk to network");
    time::sleep(Duration::from_millis(1200)).await;
    socket.send(&[2, 2, 3]).await.expect("Unable to talk to network");
    socket.send(&[3, 2, 3]).await.expect("Unable to talk to network");
    socket.send(&[4, 2, 3]).await.expect("Unable to talk to network");
    socket.send(&[5, 2, 3]).await.expect("Unable to talk to network");
    socket.send(&[6, 2, 3]).await.expect("Unable to talk to network");
    socket.send(&[7, 2, 3]).await.expect("Unable to talk to network");
    time::sleep(Duration::from_millis(1200)).await;
    socket.send(&[8, 2, 3]).await.expect("Unable to talk to network");
    time::sleep(Duration::from_millis(3200)).await;
    socket.send(&[9, 2, 3]).await.expect("Unable to talk to network"); // stop when n1 = 0

    eprintln!("Client done");
}
