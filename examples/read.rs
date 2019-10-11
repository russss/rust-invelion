extern crate invelion;
extern crate log;

use invelion::protocol::MemoryBank;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut reader = invelion::Reader::new(&args[1], 1, 4).unwrap();
    println!("Reading tags...");
    //reader.set_epc_match(&[]).unwrap();
    let data = reader.read(MemoryBank::TID, &[0, 0, 0, 0], 0, 4).unwrap();
    for response in data {
        println!("{:?}", response);
    }
}
