
use sodiumoxide::randombytes::randombytes;

fn main() {
    let v: [u8;5] = ['l' as u8, 'o' as u8, 'r' as u8, 'e' as u8, 'm' as u8];
    let n = 2;
    let mut chunks_list: Vec<&[u8]> = Vec::new();
    chunks_list = v.chunks(n).collect();
    println!("{:?}", chunks_list);
}

//   chunks_list = chunks.into_iter().map(|x| x).collect();

// https://doc.rust-lang.org/std/primitive.slice.html#method.chunks

/*
use std::fs::File;

fn main() {
    let f = File::open("hello.txt");
    let mut x: Vec<u16> = Vec::new();
    x = vec![0; 32 as usize];
    let f = match f {
        Ok(file) => {
            x[2 as usize] = 4;
            println!("{:?}", x);
            file
        }

        Err(error) => {
            x[2 as usize] = 4;
            println!("{:?}", x);
            panic!("Problem opening the file: {:?}", error)
        }
    };
}
*/