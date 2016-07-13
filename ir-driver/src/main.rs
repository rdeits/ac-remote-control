extern crate time;
use time::Tm;

extern crate wiringpi;
use wiringpi::pin::Value::{High, Low};
use wiringpi::pin::{Pin, OutputPin};
use wiringpi::time::{delay, delay_microseconds};
use wiringpi::thread::priority;

use std::cmp;
use std::num::Wrapping;

fn send_header<P: Pin>(pin: &OutputPin<P>) {
	pin.digital_write(High);
	delay_microseconds(3320);
	pin.digital_write(Low);
	delay_microseconds(1730);
}

fn send_data_bit<P: Pin>(pin: &OutputPin<P>, value: bool) {
	match value {
		true => {
			pin.digital_write(High);
			delay_microseconds(480);
			pin.digital_write(Low);
			delay_microseconds(1190);
		},
		false => {
			pin.digital_write(High);
			delay_microseconds(480);
			pin.digital_write(Low);
			delay_microseconds(340);
		},
	};
}

fn send_data_byte<P: Pin>(pin: &OutputPin<P>, data: &u8) {
	for i in 0..8 {
		send_data_bit(pin, (data & (0x01 << i) != 0));
	}
}

fn send_final_marker<P: Pin>(pin: &OutputPin<P>) {
	pin.digital_write(High);
	delay_microseconds(460);
	pin.digital_write(Low);
}

fn send_packet<P: Pin>(pin: &OutputPin<P>, bytes: &Vec<u8>) {
	send_header(pin);
	for byte in bytes { send_data_byte(pin, byte); }
	send_final_marker(pin);
}

enum FanMode {
	Auto,
	Speed(u8),
}

enum VaneMode {
	Auto,
	Move,
	Position(u8),
}

struct Fahrenheit(i32);

enum Feeling {
	TooWarm,
	MuchTooWarm,
	TooCool,
	MuchTooCool,
}

enum HvacMode {
	Heat(Fahrenheit),
	Dry,
	Cool(Fahrenheit),
	Feel(Feeling),
}

enum HvacCommand {
	Off,
	On {
		mode: HvacMode,
		fan: FanMode,
		vane: VaneMode,
		start: Option<Tm>,
		stop: Option<Tm>
	},
}

fn serialize_time(time: &Tm) -> u8 {
	let now = time::now();
	let delta = (time.tm_hour * 6 + time.tm_min / 10) - (now.tm_hour * 6 + now.tm_min / 10);
	(((delta % 24) + 24) % 24) as u8
}

fn serialize(command: &HvacCommand) -> Vec<u8> {
	// let mut data = vec![0x23, 0xcb, 0x26, 0x01, 0x00];
	let mut data = vec![0x00, 0xcb, 0x26, 0x01, 0x00];

	match command {
		&HvacCommand::Off => data.extend([0x20, 0x08, 0x07, 0x00, 0x00, 0x00].iter().clone()),
		&HvacCommand::On {ref mode, ref fan, ref vane, ref start, ref stop} => {

			let mut power_data: u8 = 0b00100100;
			match start {
				&Some(_) => power_data |= 1 << 4,
				&None => {}
			}
			match stop {
				&Some(_) => power_data |= 1 << 3,
				&None => {}
			}
			data.push(power_data);

			match mode {
				&HvacMode::Cool(_) => data.push(0b00000011),
				&HvacMode::Heat(_) => data.push(0b00000001),
				&HvacMode::Dry     => data.push(0b00000010),
				&HvacMode::Feel(_) => data.push(0b00001000),
			}

			let mut temperature_data: u8 = 0;
			match mode {
				&HvacMode::Cool(Fahrenheit(mut temperature)) | &HvacMode::Heat(Fahrenheit(mut temperature)) => {
					temperature = cmp::max(cmp::min(temperature, 89), 59);
					temperature_data |= ((89 - temperature) / 2) as u8;
				},
				&HvacMode::Feel(ref feeling) => {
					temperature_data |= 0b0111;
					match feeling {
				        &Feeling::TooWarm     => temperature_data |= 0b0010 << 4,
				        &Feeling::MuchTooWarm => temperature_data |= 0b1010 << 4,
				        &Feeling::TooCool     => temperature_data |= 0b0001 << 4,
				        &Feeling::MuchTooCool => temperature_data |= 0b1001 << 4,
					}
				}
				&HvacMode::Dry => temperature_data |= 0b0111,
			}
			data.push(temperature_data);

			let mut fan_vane_data: u8 = 0;
			match start {
				&Some(_) => fan_vane_data |= 1 << 6,
				&None => {}
			}
			match stop {
				&Some(_) => fan_vane_data |= 1 << 6, // Yup, start and stop both set the same bit here, but different bits in byte 5. 
				&None => {}
			}
			match fan {
				&FanMode::Auto => {},
				&FanMode::Speed(ref speed) => {
					let speed = cmp::min(cmp::max(*speed, 1), 4);
					fan_vane_data |= speed + 1;
				},
			}
			match vane {
				&VaneMode::Auto => {},
				&VaneMode::Move => fan_vane_data |= 0b111 << 3,
				&VaneMode::Position(mut position) => {
					position = cmp::min(cmp::max(position, 1), 5);
					fan_vane_data |= position << 3;
				},
			}
			data.push(fan_vane_data);

			match stop {
				&Some(time) => data.push(serialize_time(&time)),
				&None => data.push(0x00),
			}

			match start {
				&Some(time) => data.push(serialize_time(&time)),
				&None => data.push(0x00),
			}
		}
	}

	data.push(0x00);
	data.push(0x00);

	assert_eq!(data.len(), 13);

	let Wrapping(checksum) = data.iter().fold(Wrapping(0u8), |sum, &x| sum + Wrapping(x));
	data.push(checksum);

	data
}

fn main() {
	match priority(0x99) {
		false => panic!("Can't set priority"),
		true => {}
	}
	let pi = wiringpi::setup_gpio();
	let pin = pi.output_pin(23);
	let command = HvacCommand::On {
		mode: HvacMode::Cool(Fahrenheit(71)),
		fan: FanMode::Auto,
		vane: VaneMode::Move,
		start: None,
		stop: None,
	};
	
	println!("Running");
	let data = serialize(&command);
	print!("data: ");
	for byte in &data { print!("{:#x}, ", byte) }
	println!("");

	send_packet(&pin, &data);
}

