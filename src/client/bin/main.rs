
#![warn(rust_2018_idioms)]
use std::env;
use std::error::Error;
use std::io;
use std::io::Read;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use std::sync::Arc;

const UDP_HEADER: usize = 8;
const IP_HEADER: usize = 20;
const AG_HEADER: usize = 4;
const MAX_DATA_LENGTH: usize = (64 * 1024 - 1) - UDP_HEADER - IP_HEADER;
const MAX_CHUNK_SIZE: usize = MAX_DATA_LENGTH - AG_HEADER;

pub fn get_chunks_from_file(
    mut filename: String,
    total_size: &mut usize,
) -> Result<Vec<Vec<u8>>, io::Error> {
    filename.pop(); // get read of the trailing '\n' in user input.
    let mut file = std::fs::File::open(filename)?;
    let mut list_of_chunks = Vec::new();

    loop {
        let mut chunk = Vec::with_capacity(MAX_CHUNK_SIZE);
        let n = file.by_ref().take(MAX_CHUNK_SIZE as u64).read_to_end(&mut chunk)?;
        *total_size += n;
        if n == 0 {
            break;
        }
        //let start:usize = if list_of_chunks.len() != 0 { 0 } else { 0x20 }; // skip header
        for i in 0..n {
            chunk[i] = !chunk[i]; // neg
        }
        list_of_chunks.push(chunk);
        if n < MAX_CHUNK_SIZE {
            break;
        }
    }
    Ok(list_of_chunks)
}

/*
fn get_stdin_data() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    stdin().read_to_end(&mut buf)?;
    Ok(buf)
}*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let remote_addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".into()) // cargo run --example udp-client -- 127.0.0.1:8080
        .parse()?;

    // We use port 0 to let the operating system allocate an available port for us.
    let local_addr: SocketAddr = if remote_addr.is_ipv4() {
       "127.0.0.1:8000"// "0.0.0.0:0" //
    } else {
        "[::]:0"
    }
    .parse()?;
    println!("Listening on: {}", local_addr);
    let socket = UdpSocket::bind(local_addr).await?;

// let arc = Arc::new(socket);
    //const MAX_DATAGRAM_SIZE: usize = 65_507;
    socket.connect(&remote_addr).await?;



    let mut nb = 0; // total number of chunks to be sent


        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read from stdin");
  //      println!("{}", input);
        // input = String::from_utf8_lossy(&buffer).to_string();
        let mut total_size: usize = 0;
        let result: Result<Vec<Vec<u8>>, io::Error> = get_chunks_from_file(input, &mut total_size); // set total_size at the same time
        match result {
            Ok(ref chunks) => {
                // socket.send(input.as_bytes()).expect("Failed to write to server"); // send file
                nb = chunks.len() as u16;
                //input.as_bytes();
                let header: &mut [u8; 4] = &mut [0, 0, (nb >> 8) as u8, (nb & 0xff) as u8];
                let mut index: u16 = 0;
                for chunk in chunks.iter() {
                    header[0] = (index >> 8) as u8; // 0xFF..
                    header[1] = (index & 0xff) as u8; // 0x..FF
                    let data: Vec<u8> = [header.as_ref(), chunk].concat();
                    // println!("Chunk {} BYTES\n {:?}", index, chunk);
                    println!("Chunk {} sent", index);
                    /*     println!(
                        "size: {} FILE {:?} of {} BYTES\n {:?}",
                        total_size,
                        (header[0] as u16) << 8 | header[1] as u16,
                        nb - 1,
                        [0]
                    );*/
                    socket.send(&data).await?; //socket.send(&data).expect("Failed to write to server");

                    index += 1;
                }
                //let mut buffer = [0u16; index];

            },
            Err(ref e) => eprintln!("Error: {}", e),
        };
   //println!("HELLLLO {}", socket.clone());
           // let len = socket.recv(&mut data).await?;

           let chunks = result.unwrap();

             let mut buffer = [0u8; MAX_DATA_LENGTH];

          loop {
           let len:usize = socket.recv(&mut buffer).await.unwrap();
           println!("receiving  new message");


               unsafe {
                 //   let missing_indexes: Vec<u16> =
                   //     (buffer[..len].align_to::<u16>().1).to_vec();
                        println!("{:?}", &buffer[..len]);
                    let header2: &mut [u8; 4] =
                        &mut [0, 0, (nb >> 8) as u8, (nb & 0xff) as u8];
                    for (i, missing_index) in buffer[..len].iter().enumerate() {
                       // let index = missing_index >> 8 | (missing_index & 0xff) << 8; // need to switch bytes because of little endian
                        if missing_index == &0u8 { // chunk was received
                            println!("Chunk {} not received by peer, resending...", i);
                            header2[0] = (i >> 8) as u8; // 0xFF..
                            header2[1] = (i & 0xff) as u8; // 0x..FF
                            let missing_chunk = &chunks[i];
                            let data: Vec<u8> = [header2.as_ref(), &missing_chunk].concat();
                            socket.send(&data).await?; //.expect("Failed to write to server");
                        }
                    }
                }
}
                //print!( "{}",str::from_utf8(&buffer).expect("Could not write buffer as string"));
                //  println!( "Chunk not received by server {:?}", &buffer);



        /*
            let mut data = String::new();
            io::stdin().read_line(&mut data).expect("Failed to read from stdin");
          //  let data:Vec<u8> = get_stdin_data()?;
            println!("{:?}", data);
            println!("ok");
           socket.send(&data).await?;
            let mut data = vec![0u8; MAX_DATAGRAM_SIZE];


           let len = socket.recv(&mut data).await?;
            println!(
                "Received {} bytes:\n{}",
                len,
                String::from_utf8_lossy(&data[..len])
            );
        */
//Ok(())

}
