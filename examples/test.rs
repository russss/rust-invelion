extern crate yr905;
extern crate log;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut reader = yr905::Reader::new(&args[1]).unwrap();
    println!("Version: {:?}", reader.get_version().unwrap());
    println!("Output power: {:?}", reader.get_output_power().unwrap());
    println!("Temperature: {:?}", reader.get_temperature().unwrap());
    println!("Tags {:?}", reader.inventory(255).unwrap());
}
