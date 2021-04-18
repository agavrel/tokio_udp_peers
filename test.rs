use qrcode::QrCode; // QR encoding
use image::Luma;
use rqrr; // QR decoding


fn decode_qr(s : &[u8]) -> Result<(), rqrr::DeQRError> {
    let img = image::open("/tmp/qrcode.png").unwrap().to_luma8();
    // Prepare for detection
    let mut img = rqrr::PreparedImage::prepare(img);
    // Search for grids, without decoding
    let grids = img.detect_grids();
    assert_eq!(grids.len(), 1);
    // Decode the grid
    let (meta, content) = grids[0].decode()?;
    assert_eq!(meta.ecc_level, 0);
    assert_eq!(content.as_bytes(), s);

    Ok(())
}

fn main()  {
    // Encode some data into bits.
    let s:&[u8] = b"https://www.google.com";
    let code = QrCode::new(s).unwrap();

    // Render the bits into an image.
    let image = code.render::<Luma<u8>>().build();

    // Save the image.
    image.save("/tmp/qrcode.png").unwrap();

    // You can also render it into a string.
    let string = code.render()
        .light_color(' ')
        .dark_color('#')
        .build();
    println!("{}", string);

    let result = decode_qr(s);
    match result {
         Err(e) => println!("error parsing QR code: {:?}", e),
         Ok(v) => println!("working QR code"),
        }

}

/*

use sodiumoxide::randombytes::randombytes;

fn main() {
    let v: [u8;5] = ['l' as u8, 'o' as u8, 'r' as u8, 'e' as u8, 'm' as u8];
    let n = 2;
    let mut chunks_list: Vec<&[u8]> = Vec::new();
    chunks_list = v.chunks(n).collect();
    println!("{:?}", chunks_list);
}

*/

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