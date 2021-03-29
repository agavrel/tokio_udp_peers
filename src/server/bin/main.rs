#![warn(rust_2018_idioms)]

use std::error::Error;
use std::net::SocketAddr;
use std::{env, io};
use tokio::net::UdpSocket;


use std::alloc::{alloc, dealloc, Layout};
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::mem::MaybeUninit;
use std::thread;

/// Encryption
use sodiumoxide::randombytes::randombytes;
use sodiumoxide::crypto::secretstream::{Stream, Tag};
use sodiumoxide::crypto::secretstream::xchacha20poly1305::Key;
use std::time::Duration;
// delay before asking more chunks
use std::time;

//mod tests;

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
        ".avi" => vec![vec![
            0x52, 0x49, 0x46, 0x46, WILD, WILD, WILD, WILD, 0x41, 0x56, 0x49, 0x20,
        ]],
        ".mp3" => vec![vec![0xFF, 0xFB], vec![0xFF, 0xF2], vec![0xFF, 0xF3]],
        ".webp" => vec![vec![
            0x52, 0x49, 0x46, 0x46, WILD, WILD, WILD, WILD, 0x57, 0x45, 0x42, 0x50,
        ]],
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

fn generate_key(random_bytes: Vec<u8>) -> Key {//fb gena(random_bytes: Vec<u8>)-> Key  {
    let option_key: Option<Key> = Key::from_slice(&random_bytes);
    let key = option_key.unwrap();
    return key;
}


struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    to_send: Option<(usize, SocketAddr)>,
}

impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server {
            socket,
            mut buf,
            mut to_send,
        } = self;

        loop {
            // First we check to see if there's a message we need to echo back.
            // If so then we try to send it back to the original source, waiting
            // until it's writable and we're able to do so.

            if let Some((size, peer)) = to_send {
                println!("hey");
                let amt = socket.send_to(&buf[..size], &peer).await?;

                println!("Echoed {}/{} bytes to {}", amt, size, peer);
            }

            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            to_send = Some(socket.recv_from(&mut buf).await?);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    println!("Listening on: {}", socket.local_addr()?);

    let server = Server {
        socket,
        buf: vec![0; 1024],
        to_send: None,
    };
    let filename = "3.m4a";
    let mut len: usize = 0; // total len of bytes that will be written
    let mut missing_indexes: Vec<u16> = Vec::new();
    let mut layout = MaybeUninit::<Layout>::uninit();
    let mut peer_addr = MaybeUninit::<SocketAddr>::uninit();
    //let mut data = std::ptr::null_mut(); // ptr for the file bytes
    let mut buf = [0u8; MAX_DATA_LENGTH];

    let key_bytes: Vec<u8> = randombytes(0x20);
    let key = generate_key(key_bytes);





    // This starts the server task.
    server.run().await?;

    Ok(())
}