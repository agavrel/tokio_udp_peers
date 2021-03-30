use std::time::Duration;
use tokio::{sync::mpsc, task, time}; // 1.3.0

#[tokio::main]
async fn main() {
    // mpsc::channel(3000)::<(usize, SocketAddr)>(3300);

    let mut start = false;

    let (debounce_tx, mut debounce_rx) = mpsc::channel::<Vec<u8>>(3300); // mpsc::channel<Vec<u8>>(3300);
    let (network_tx, mut network_rx) = mpsc::channel::<Vec<u8>>(3300);
    // Listen for events
    let debouncer = task::spawn(async move {
        let duration = Duration::from_millis(3000);
        let mut total_list_of_ids: Vec<i32> = Vec::new();
        total_list_of_ids = vec![0; 6];

        loop {
            match time::timeout(duration, debounce_rx.recv()).await {
                Ok(Some(bytes)) => {
                    // TODO: add mutex for very first packet

                    eprintln!("Network activity");
                    let id = bytes.clone()[0];
                    eprintln!("id: {}", id);
                    total_list_of_ids[id as usize] = 1;
                    eprintln!("{:?}", total_list_of_ids);
                }
                Ok(None) => {
                    if total_list_of_ids.iter().all(|x| x == &1i32) {
                        // write file
                        eprintln!("Debounce finished");
                        start = false;
                        break;
                    }
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
            while let Some(bytes) = network_rx.recv().await {
                start = true;
                // Received a packet
                debounce_tx.send(bytes).await.expect("Unable to talk to debounce");
                //  eprintln!("Received a packet: {:?}", bytes.clone());
            }
        }
    });

    // Prevent deadlocks
    drop(debounce_tx);

    /*
    // Drive the network input
    network_tx.send(vec![1, 2, 3]).await.expect("Unable to talk to network");
    network_tx.send(vec![4, 2, 3]).await.expect("Unable to talk to network");
    network_tx.send(vec![3, 2, 3]).await.expect("Unable to talk to network");

    time::sleep(Duration::from_millis(3200)).await;

    network_tx.send(vec![2, 2, 3]).await.expect("Unable to talk to network");
    network_tx.send(vec![5, 2, 3]).await.expect("Unable to talk to network");
    network_tx.send(vec![0, 2, 3]).await.expect("Unable to talk to network");

    time::sleep(Duration::from_millis(3200)).await;

    // Close the network
    drop(network_tx);

    // Wait for everything to finish
    server.await.expect("Server panicked");
    debouncer.await.expect("Debouncer panicked");

    */
}
