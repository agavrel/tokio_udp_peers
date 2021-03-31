use rand::Rng;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use std::{env, io};
use tokio::net::UdpSocket;
use tokio::{sync::mpsc, task, time}; // 1.4.0 // 0.8.0

const UDP_HEADER: usize = 8;
const IP_HEADER: usize = 20;
const AG_HEADER: usize = 4;
const MAX_DATA_LENGTH: usize = (64 * 1024 - 1) - UDP_HEADER - IP_HEADER;
const MAX_CHUNK_SIZE: usize = MAX_DATA_LENGTH - AG_HEADER;
const MAX_DATAGRAM_SIZE: usize = 0x10000;
const ADDRESS: &str = "127.0.0.1:8775";

#[tokio::main]
async fn main() {
    // mpsc::channel(3000)::<(usize, SocketAddr)>(3300);

    /*
        eprintln!("waiting for first packet");
        let result = network_rx.recv().await;
        eprintln!("{:?}", result.unwrap());

        eprintln!("start");
    */
      let addr = env::args().nth(1).unwrap_or_else(|| ADDRESS.to_string());

        let socket = UdpSocket::bind(&addr).await.unwrap();
        println!("Listening to {} ", addr);

    let mut missing_ids: Vec<i32> = Vec::new();
    missing_ids = vec![0; 10];

    let (debounce_tx, mut debounce_rx) = mpsc::channel::<Vec<u8>>(1300); // mpsc::channel<Vec<u8>>(3300);
    let (network_tx, mut network_rx) = mpsc::channel::<Vec<u8>>(1300);
    // Listen for events
    let debouncer = task::spawn(async move {
        let duration = Duration::from_millis(1000);

        loop {
            match time::timeout(duration, debounce_rx.recv()).await {
                Ok(Some(bytes)) => {
                    // TODO: add mutex for very first packet

                    eprintln!("Network activity");
                    let id = bytes.clone()[0];
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
                    // write file
                    eprintln!("Debounce finished");
                    break;
                }
                Err(_) => {
                    eprintln!("{:?} since network activity", duration)
                    // request for missing indexes
                }
            }
        }
    });

    // Listen for network activity

    let server = task::spawn({
        let debounce_tx = debounce_tx.clone();
        async move {
            'outer: while let Some(bytes) = network_rx.recv().await {
                // start = true;
                // Received a packet
                let result = debounce_tx.send(bytes).await;

                println!("{:?}", result);
                match result {
                    Ok(v) => println!("working with version: {:?}", v),
                    Err(e) => break 'outer,
                }

                //.expect("Unable to talk to debounce");

                //  eprintln!("Received a packet: {:?}", bytes.clone());
            }
        }
    });

    // Prevent deadlocks
    drop(debounce_tx);
    let  to_send: Option<(usize, SocketAddr)>;





        let mut buf = [0u8; MAX_DATA_LENGTH];
    loop {
        // TODO: SHOULD BE A LOOP
        // let sock = socket.try_clone().expect("Failed to clone socket");

        // [0u8; MAX_DATA_LENGTH];

            let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
       //     to_send = socket. poll_recv_from(&mut buf).await;
        network_tx.send(buf.to_vec()).await.expect("Unable to talk to network");

        /*    // Drive the network input
           // println!("{:?}", debounce_tx);
            network_tx.send(vec![1, 2, 3]).await.expect("Unable to talk to network");
            network_tx.send(vec![4, 2, 3]).await.expect("Unable to talk to network");
            network_tx.send(vec![3, 2, 3]).await.expect("Unable to talk to network");

            time::sleep(Duration::from_millis(1200)).await;

            network_tx.send(vec![2, 2, 3]).await.expect("Unable to talk to network");

            network_tx.send(vec![5, 2, 3]).await.expect("Unable to talk to network");

            let mut rng = rand::thread_rng();
            let c = 1; // change for different probability
            let n1: u8 = rng.gen_range(0..c);
            network_tx.send(vec![n1, 2, 3]).await.expect("Unable to talk to network"); // stop when n1 = 0

            time::sleep(Duration::from_millis(1200)).await;

        */
        //   println!("{:?}", debounce_tx);
    }

    // Close the network
    drop(network_tx);

    // Wait for everything to finish
    server.await.expect("Server panicked");
    debouncer.await.expect("Debouncer panicked");
}
