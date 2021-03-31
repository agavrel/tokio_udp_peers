use std::time::Duration;
use tokio::{sync::mpsc, task, time, net::{TcpStream}, io::{AsyncWriteExt}}; // 1.4.0
use tokio::net::UdpSocket;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use std::env;
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
    match std::env::args().nth(1).as_deref() {
        Some("client") => client().await,
        _ => server().await,
    }
}

async fn server() {
    eprintln!("Starting the server");

    let addr = env::args().nth(1).unwrap_or_else(|| ADDRESS.to_string());
     eprintln!("{}", addr);
    let socket = UdpSocket::bind(&addr).await.unwrap();

    //let listener = TcpListener::bind(ADDRESS).await.expect("Unable to bind");

    loop {
        let mut buf = [0u8; MAX_DATA_LENGTH];
        let bytes= socket.recv_from(&mut buf).await.unwrap();
        let (debounce_tx, mut debounce_rx) = mpsc::channel::<Vec<u8>>(1300); // mpsc::channel<Vec<u8>>(3300);
            let (mut network_tx, mut network_rx) = mpsc::channel::<Vec<u8>>(1300);
println!("gaaaaaa");
        // Listen for events
        let debouncer = task::spawn(async move {
            let mut missing_ids: Vec<i32> = Vec::new();
                missing_ids = vec![0; 10];
            let duration = Duration::from_millis(1300);

            loop {
            eprintln!("heeeeyeytrsrthyrth");
                match time::timeout(duration, debounce_rx.recv()).await {
                    Ok(Some(bytes)) => {
                    eprintln!("{:?}", bytes);
                        eprintln!("Network activity");
                        let id:u8 = bytes.clone()[0];
                        //eprintln!("id: {} {:?}", id, bytes);
                        eprintln!("id: {}", id);
                        missing_ids[id as usize] = 1;
                        eprintln!("{:?}", missing_ids);

                        if missing_ids.iter().all(|x| x == &1i32) {
                            println!("FINISHHHHHHH ");
                            break;
                            //break;
                        }
                    }
                    Ok(None) => {
                        eprintln!("Done: {:?}", missing_ids);

                        break;
                    }
                    Err(_) => {
                        eprintln!("Error: {:?}", missing_ids);

                    }
                }
            }
        });

        // Listen for network activity
        let server = task::spawn({
            let debounce_tx = debounce_tx.clone();
            async move {
                let mut buf = [0u8; MAX_DATA_LENGTH];
                while let Some(bytes) = network_rx.recv().await {
                    eprintln!("Bytes received: {:?}", bytes);
                    // let (len, addr) = socket.recv_from(&mut buf).await.unwrap() { //

                    debounce_tx
                        .send(bytes)
                        .await
                        .expect("Unable to talk to debounce");
                }
            }
        });

        // Prevent deadlocks
        drop(debounce_tx);


    }


/*

   // Wait for everything to finish
        server.await.expect("Server panicked");
        debouncer.await.expect("Debouncer panicked");
        */

    //eprintln!("Server done");
}

async fn client() {
    eprintln!("Starting the client");


     let remote_addr: SocketAddr = env::args()
        .nth(2)
        .unwrap_or_else(|| "127.0.0.1:8080".into()) // cargo run --example udp-client -- 127.0.0.1:8080
        .parse().unwrap();

    // We use port 0 to let the operating system allocate an available port for us.



    println!("Listening on: {}", ADDRESS_CLIENT);
    let socket = UdpSocket::bind(ADDRESS_CLIENT).await.unwrap();
    let data:[u8;5] = [1, 2, 3, 4, 5] ;

    let len = socket.send_to(&data, remote_addr).await.unwrap();



 /*
    socket.send(vec![4, 2, 3]).await.expect("Unable to talk to network");
    socket.send(vec![3, 2, 3]).await.expect("Unable to talk to network");

    time::sleep(Duration::from_millis(1200)).await;

    socket.send(vec![2, 2, 3]).await.expect("Unable to talk to network");

    socket.send(vec![5, 2, 3]).await.expect("Unable to talk to network");

    let mut rng = rand::thread_rng();
    let c = 1; // change for different probability
    let n1: u8 = rng.gen_range(0..c);
    socket.send(vec![n1, 2, 3]).await.expect("Unable to talk to network"); // stop when n1 = 0

    time::sleep(Duration::from_millis(1200)).await;
*/
    eprintln!("Client done");
}
