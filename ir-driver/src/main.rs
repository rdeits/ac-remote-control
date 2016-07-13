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
	delay_microseconds(3400);
	pin.digital_write(Low);
	delay_microseconds(1750);
}

fn send_data_bit<P: Pin>(pin: &OutputPin<P>, value: bool) {
	match value {
		true => {
			pin.digital_write(High);
			delay_microseconds(450);
			pin.digital_write(Low);
			delay_microseconds(1300);
		},
		false => {
			pin.digital_write(High);
			delay_microseconds(450);
			pin.digital_write(Low);
			delay_microseconds(420);
		},
	};
}

fn send_data_byte<P: Pin>(pin: &OutputPin<P>, data: &u8) {
	for i in 0..8 {
		send_data_bit(pin, (data & (0x01 << i) != 0));
	}
}

fn send_repeat_marker<P: Pin>(pin: &OutputPin<P>) {
	pin.digital_write(High);
	delay_microseconds(440);
	pin.digital_write(Low);
	delay_microseconds(17100);
}

fn send_packet<P: Pin>(pin: &OutputPin<P>, bytes: &Vec<u8>) {
	for i in 0..2 {
		send_header(pin);
		for byte in bytes { send_data_byte(pin, byte); }
		send_repeat_marker(pin);
	}
}

enum Power {
	On,
	Off,
}

enum HvacMode {
	Heat,
	Dry,
	Cold,
	Auto,
}

enum FanMode {
	Auto,
	Speed(u8),
}

enum VaneMode {
	Auto,
	Move,
	Set(u8),
}

struct Celsius(i32);

struct HvacCommand {
	power: Power,
	mode: HvacMode,
	temperature: Celsius,
	fan: FanMode,
	vane: VaneMode,
	clock: Tm,
	start: Option<Tm>,
	end: Option<Tm>,
}

fn serialize_time(time: &Tm) -> u8 {
	(time.tm_hour * 6 + time.tm_min / 10) as u8
}

fn serialize(command: &HvacCommand) -> Vec<u8> {
	let mut data = vec![0x23, 0xcb, 0x26, 0x01, 0x00];

	match command.power {
		Power::On => data.push(0x20),
		Power::Off => data.push(0x00),
	}

	match command.mode {
		HvacMode::Heat => data.push(0x08),
		HvacMode::Dry => data.push(0x10),
		HvacMode::Cold => data.push(0x18),
		HvacMode::Auto => data.push(0x20),
	}

	let Celsius(mut temperature) = command.temperature;
	temperature = cmp::max(temperature, 16);
	temperature = cmp::min(temperature, 31);
	data.push((temperature - 16) as u8);

	match command.mode {
		HvacMode::Heat => data.push(0x30),
		HvacMode::Dry => data.push(0x32),
		HvacMode::Cold => data.push(0x36),
		HvacMode::Auto => data.push(0x30),
	}

	let mut fan_vane_data: u8 = 0x00;
	match command.fan {
		FanMode::Auto => fan_vane_data |= 1 << 7,
		FanMode::Speed(mut speed) => {
			speed = cmp::min(cmp::max(speed, 0), 7);
			fan_vane_data |= speed
		},
	}
	match command.vane {
		VaneMode::Set(mut angle) => {
			angle = cmp::min(cmp::max(angle, 5), 1);
			fan_vane_data |= 0b01000000;
			fan_vane_data |= angle << 3;
		},
		VaneMode::Auto => fan_vane_data |= 0b01000000,
		VaneMode::Move => fan_vane_data |= 0b01111000,
	}
	data.push(fan_vane_data);

	data.push(serialize_time(&command.clock));
	
	let mut prog_mode_data: u8 = 0x00;
	match command.end {
		Some(time) => {
			data.push(serialize_time(&time));
			prog_mode_data |= 0b00000101;
		}
		None => data.push(0x00),
	}

	match command.start {
		Some(time) => {
			data.push(serialize_time(&time));
			prog_mode_data |= 0b00000011;
		},
		None => data.push(0x00),
	}

	data.push(prog_mode_data);

	data.push(0x00);
	data.push(0x00);

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
	let command = HvacCommand{
		power: Power::On,
		mode: HvacMode::Cold,
		temperature: Celsius(23),
		fan: FanMode::Auto,
		vane: VaneMode::Move,
		clock: time::now(),
		start: None,
		end: None,
	};
	
	println!("Running");
	let data = serialize(&command);
	print!("data: ");
	for byte in &data { print!("{:#x}, ", byte) }
	println!("");

	send_packet(&pin, &data);
}

