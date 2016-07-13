extern crate wiringpi;
extern crate time;

use wiringpi::pin::Value::{High, Low};
use wiringpi::thread::priority;

#[derive(Clone)]
enum Mode {
	Waiting,
	HeaderHigh,
	HeaderLow,
	BitHigh,
	BitLow,
	Done,
}

fn main() {

	match priority(99) {
		false => panic!("can't set priority"),
		true => {},
	}

	let pi = wiringpi::setup_gpio();
	let pin = pi.input_pin(23);
	
	println!("ready");

	loop {

	let mut mode = Mode::Waiting;
	let mut bit_low_start = time::SteadyTime::now();
	let mut data: Vec<u8> = Vec::new();
	let mut current_byte: u8 = 0;
	let mut byte_index: u8 = 0;

	{
		let mut append = |bit: u8| {
			current_byte |= bit << byte_index;
			if byte_index < 7 {
				byte_index += 1;
			} else {
				byte_index = 0;
				data.push(current_byte);
				current_byte = 0;
			}
		};
		
		loop {
			let value = pin.digital_read();
			mode = match (&mode, value) {
				(&Mode::Waiting, High) => Mode::HeaderHigh,
				(&Mode::HeaderHigh, Low) => Mode::HeaderLow,
				(&Mode::HeaderLow, High) => Mode::BitHigh,
				(&Mode::BitHigh, Low) => {bit_low_start = time::SteadyTime::now(); Mode::BitLow},
				(&Mode::BitLow, High) => match (time::SteadyTime::now() - bit_low_start).num_microseconds() {
						Some(i) if i < 800 => {append(0); Mode::BitHigh},
						Some(i) if i >= 800 && i < 2000 => {append(1); Mode::BitHigh}, 
						_ => panic!("could not match microseconds"),
				},
				(&Mode::BitLow, Low) => match (time::SteadyTime::now() - bit_low_start).num_microseconds() {
						Some(i) if i >= 20000 => Mode::Done,
						_ => Mode::BitLow,
					},
				(&Mode::Done, Low) => break,
				(&Mode::Done, High) => break,
				_ => {mode}
			}
		}
	}
	
	for (i, byte) in data.iter().enumerate() {
		println!("{}:\t{:08b}", i, byte);
	}

	println!("");

	}
}
