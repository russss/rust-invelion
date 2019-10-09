extern crate invelion;
extern crate log;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut reader = invelion::Reader::new(&args[1], 1, 4).unwrap();
    println!("Version: {:?}", reader.get_version().unwrap());
    println!("Output power: {:?}", reader.get_output_power().unwrap());
    println!("Temperature: {:?}", reader.get_temperature().unwrap());
    println!("Tags {:?}", reader.real_time_inventory(255).unwrap());
}
