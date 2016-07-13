extern crate wiringpi;
extern crate time;
extern crate csv;

use wiringpi::pin::Value::{High, Low};
use wiringpi::thread::priority;
use std::fs::File;
use std::path::Path;

fn main() {

	match priority(99) {
		false => panic!("can't set priority"),
		true => {},
	}

	let pi = wiringpi::setup_gpio();
	let pin = pi.input_pin(23);
	
	println!("ready");

	let start_time = time::PreciseTime::now();

	let mut data = : Vec<(i64, bool)>::with_capacity(200000);

	while !pin.digital_read() {}

	let mut last_high_time = time::PreciseTime::now();

	loop {
		current_time = time::PreciseTime::now();
		let value = pin.digital_read();
		match value {
			High => last_high_time = current_time;
			Low => {
				if current_time - last_high_time > Duration::milliseconds(20) {
					break;
				}
			}
		data.push(((current_time - start_time).num_microseconds().expect("couldn't convert microseconds"), value));
	}


	let path = Path::new("ir_data.csv");
	let mut writer = csv::Writer::from_file(path).expect("couldn't open file");
	for record in data.into_iter() {
		let result = writer.encode(record);
		assert!(resuilt.is_ok());
	}
	println!("wrote {} lines", data.len());
}
