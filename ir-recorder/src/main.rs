extern crate wiringpi;
extern crate time;
extern crate csv;

use wiringpi::pin::Value::{High, Low};
use wiringpi::thread::priority;
use std::path::Path;

fn main() {

	match priority(99) {
		false => panic!("can't set priority"),
		true => {},
	}

	let pi = wiringpi::setup_gpio();
	let pin = pi.input_pin(23);
	
	println!("ready");

	let mut data = Vec::<(bool, i64)>::with_capacity(1000);

	while pin.digital_read() == Low {}

	let mut last_change_time = time::PreciseTime::now();
	let mut state = High;

	loop {
		let current_time = time::PreciseTime::now();
		let time_since_change = last_change_time.to(current_time);

		if time_since_change > time::Duration::milliseconds(20) {
			data.push((state == High, time_since_change.num_microseconds().expect("couldn't convert to usec")));
			break;
		}
		
		let value = pin.digital_read();
		if value == state {
			continue;
		} else {
			data.push((state == High, time_since_change.num_microseconds().expect("couldn't convert to us")));
			state = value;
			last_change_time = current_time;
		}
	}


	println!("writing {} lines", data.len());
	let path = Path::new("ir_data.csv");
	let mut writer = csv::Writer::from_file(path).expect("couldn't open file");
	for record in data.into_iter() {
		let result = writer.encode(record);
		assert!(result.is_ok());
	}
}
