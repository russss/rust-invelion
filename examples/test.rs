extern crate yr905;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut reader = yr905::Reader::new(&args[1]).unwrap();
    reader.get_version().unwrap();
}
