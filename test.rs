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
