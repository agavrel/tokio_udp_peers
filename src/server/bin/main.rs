
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::{sync::mpsc, task, time}; // 1.4.0


use std::{env, io};

/// Encryption
use sodiumoxide::crypto::secretstream::xchacha20poly1305::Key;
use sodiumoxide::randombytes::randombytes;

use std::io::prelude::*;

use std::fs::File;
use std::alloc::{alloc, Layout};
use std::mem;
use std::mem::MaybeUninit;

const UDP_HEADER: usize = 8;
const IP_HEADER: usize = 20;
const AG_HEADER: usize = 4;
const MAX_DATA_LENGTH: usize = (64 * 1024 - 1) - UDP_HEADER - IP_HEADER;
const MAX_CHUNK_SIZE: usize = MAX_DATA_LENGTH - AG_HEADER;
const MAX_DATAGRAM_SIZE: usize = 0x10000;
// cmp -l 1.jpg 2.jpg

/// A wrapper for [ptr::copy_nonoverlapping] with different argument order (same as original memcpy)
/// Safety: see `std::ptr::copy_nonoverlapping`.
#[inline(always)]
unsafe fn memcpy(dst_ptr: *mut u8, src_ptr: *const u8, len: usize) {
    std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, len);
}

#[inline(always)]
// Different from https://doc.rust-lang.org/std/primitive.u32.html#method.next_power_of_two
// Returns the [exponent] from the smallest power of two greater than or equal to n.
const fn next_power_of_two_exponent(n: u32) -> u32 {
    return 32 - (n - 1).leading_zeros();
}

#[inline(always)]
fn write_chunks_to_file(filename: &str, bytes: &[u8]) -> io::Result<()> {
    let mut file = File::create(filename)?;
    Ok(file.write_all(bytes)?)
}

// Thanks https://www.rosettacode.org/wiki/Extract_file_extension#Rust
fn extension(filename: &str) -> &str {
    filename
        .rfind('.')
        .map(|idx| &filename[idx..])
        .filter(|ext| ext.chars().skip(1).all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or("")
}

// https://en.wikipedia.org/wiki/List_of_file_signatures
// NB: magic (number) means file signature
fn is_file_extension_matching_magic(filename: &str, bytes: Vec<u8>) -> bool {
    const WILD: u8 = 0xFC; // unspecified byte, could be anything, just make sure
                           // that it is not one of the already used bytes among magic numbers
    let file_extension = extension(filename);

    // get supposed magic based on file extension
    let v = match file_extension {
        ".bmp" => vec![vec![0x42, 0x4D]],
        ".jpg" => vec![vec![0xFF, 0xD8, 0xFF]],
        ".png" => vec![vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]],
        ".gif" => vec![vec![0x47, 0x49, 0x46, 0x38]],
        ".m4a" => vec![vec![
            0x00, 0x00, 0x00, 0x1c, 0x66, 0x74, 0x79, 0x70, 0x69, 0x73, 0x6f, 0x6d, 0x00, 0x00,
            0x02, 0x00, 0x69, 0x73, 0x6f, 0x6d, 0x69, 0x73, 0x6f, 0x32, 0x6d, 0x70, 0x34, 0x31,
        ]],
        ".pdf" => vec![vec![0x25, 0x50, 0x44, 0x46, 0x2d]],
        ".avi" => {
            vec![vec![0x52, 0x49, 0x46, 0x46, WILD, WILD, WILD, WILD, 0x41, 0x56, 0x49, 0x20]]
        }
        ".mp3" => vec![vec![0xFF, 0xFB], vec![0xFF, 0xF2], vec![0xFF, 0xF3]],
        ".webp" => {
            vec![vec![0x52, 0x49, 0x46, 0x46, WILD, WILD, WILD, WILD, 0x57, 0x45, 0x42, 0x50]]
        }
        _ => return true,
    };
    // check that actual magic from bytes match its supposed magic
    'outer: for magic_bytes in v.iter() {
        for i in 0..magic_bytes.len() - 1 {
            //println!("{:x} ", magic_bytes[i]);
            if magic_bytes[i] ^ bytes[i] != 0 && magic_bytes[i] != WILD {
                continue 'outer;
            }
        }
        if magic_bytes[magic_bytes.len() - 1] ^ bytes[magic_bytes.len() - 1] == 0 {
            return true;
        }
    }
    println!(
        "{} with {} ext does not have magic {:x?} matching its extension",
        filename, file_extension, v
    );
    return false;
}

fn generate_key(random_bytes: Vec<u8>) -> Key {
    //fb gena(random_bytes: Vec<u8>)-> Key  {
    let option_key: Option<Key> = Key::from_slice(&random_bytes);
    let key = option_key.unwrap();
    return key;
}

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
 //   let sender = UdpSocket::bind(&addr).await.unwrap();
    let arc = Arc::new(socket);
    let mut buf = [0u8; MAX_DATA_LENGTH];

    let mut peer_addr = MaybeUninit::<SocketAddr>::uninit();
   //         let mut data = std::ptr::null_mut(); // ptr for the file bytes
    let filename = "3.m4a";
    let mut layout = MaybeUninit::<Layout>::uninit();
    let mut chunks_cnt: u16 = 0;
  //  let mut start = false;
    let key_bytes: Vec<u8> = randombytes(0x20);
    let key = generate_key(key_bytes);

    let mut packet_ids: Vec<u16> = Vec::new();
    let mut data = std::ptr::null_mut(); // ptr for the file bytes

    let (debounce_tx, mut debounce_rx) = mpsc::channel::<u16>(256);

    let _debouncer = task::spawn(async move {


        let duration = Duration::from_millis(1300);

        loop {
            match time::timeout(duration, debounce_rx.recv()).await {
                Ok(Some(id)) => {
                     eprintln!("WTF ??????: {:?}", packet_ids);
                 //   let current_chunks_cnt = chunks_cnt.clone();


                        packet_ids[id as usize] = 1;
                    //    eprintln!("{} id packet received:{:?}", id, *packet_ids);
                        if packet_ids.iter().all(|x| x == &1u16) {
                            println!("All packets have been received, stop program ");
                            start = false;
                        }

                }
                Ok(None) => {
                    eprintln!("Done: {:?}", packet_ids);
                    break;
                }
                Err(_) => {
                 //   eprintln!("No activity for 1.3sd");

                     unsafe {
                        let missing_chunks = packet_ids.align_to::<u8>().1; // convert from u16 to u8
                       // arc.clone().send_to(&*missing_chunks, &peer_addr.assume_init()).await;
                   //     println!("Resquesting missing ids: {:?}", packet_ids);
                        // sock.send_to(&missing_chunks, &peer_addr.assume_init())
                        //   .expect("Failed to send a response");
                    }


                }
            }
        }
    });
    // Listen for first packet

    let result = arc.clone().recv_from(&mut buf).await;
    match result {
        Ok((len, addr)) => {
            eprintln!("Bytes len: {} from {}", len, addr);

            chunks_cnt = (buf[2] as u16) << 8 | buf[3] as u16;
            let n: usize = MAX_DATAGRAM_SIZE << next_power_of_two_exponent(chunks_cnt as u32);
            debug_assert_eq!(n.count_ones(), 1); // can check with this function that n is aligned on power of 2
            eprintln!("chunk count: {}", chunks_cnt);
            packet_ids = vec![0; chunks_cnt as usize];

            eprintln!("packets count: {:?}", packet_ids);

            let id: u16 = (buf[0] as u16) << 8 | buf[1] as u16;
             unsafe {
                // SAFETY: layout.as_mut_ptr() is valid for writing and properly aligned
                // SAFETY: align_of<u8>() is nonzero and a power of two thanks to previous function
                // SAFETY: no shift amount will make 0x10000 << x round up to usize::MAX
                layout
                    .as_mut_ptr()
                    .write(Layout::from_size_align_unchecked(n, mem::align_of::<u8>()));
                // SAFETY: layout is initialized right before calling assume_init()
                data = alloc(layout.assume_init());
                peer_addr.as_mut_ptr().write(addr);
                let dst_ptr = data.offset((id as usize * MAX_CHUNK_SIZE) as isize);
                memcpy(dst_ptr, &buf[AG_HEADER], len - AG_HEADER);
            }
if id < chunks_cnt {
            debounce_tx.send(id).await.expect("Unable to talk to debounce");
            }
        }
        Err(_) => {
            eprintln!("Couldnt get datagram");
        }
    }
    start = true;
    // listen for other packets
    while start {
        let thread_socket = arc.clone();
        let debounce_tx = debounce_tx.clone();
    /*    let _server = task::spawn({


            async move {*/

        //    eprintln!("runnning");
                if let result = thread_socket.recv_from(&mut buf).await {
                    match result {
                        Ok((len, addr)) => {
                           //eprintln!("Bytes len: {} from {}", len, addr);

                            let id: u16 = (buf[0] as u16) << 8 | buf[1] as u16;
                            eprintln!("{} id received", id);
                            unsafe {
                                let dst_ptr = data.offset((id as usize * MAX_CHUNK_SIZE) as isize);
                                memcpy(dst_ptr, &buf[AG_HEADER], len - AG_HEADER);
                            };
                        if id < chunks_cnt {
                            debounce_tx
                                .send(id)
                                .await
                                .expect("Unable to talk to debounce");
                                }

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
