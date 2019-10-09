extern crate invelion;
extern crate log;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut reader = invelion::Reader::new(&args[1], 1, 4).unwrap();
    println!("Return loss");
    for i in 0..4 {
        reader.set_work_antenna(i).unwrap();
        assert_eq!(reader.get_work_antenna().unwrap(), i);
        println!("=========\nAntenna {}", i);
        println!("Connection detector threshold: {} dB", reader.get_antenna_connection_detector().unwrap());
        for j in 0..7 {
            let freq = j as f32 * 0.5 + 865.;
            println!("{} MHz: {:?} dB", freq, reader.measure_return_loss(freq).unwrap());
        }
    }    
}
