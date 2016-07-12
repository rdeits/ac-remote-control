extern crate wiringpi;

use wiringpi::pin::Value::{High, Low};
use wiringpi::pin::{Pin, OutputPin};
use wiringpi::time::{delay, delay_microseconds};
use wiringpi::thread::priority;

fn send_header<P: Pin>(pin: &OutputPin<P>) {
	pin.digital_write(Low);
	delay_microseconds(3400);
	pin.digital_write(High);
	delay_microseconds(1750);
}

fn send_data_bit<P: Pin>(pin: &OutputPin<P>, value: bool) {
	match value {
		true => {
			pin.digital_write(Low);
			delay_microseconds(450);
			pin.digital_write(High);
			delay_microseconds(1300);
		},
		false => {
			pin.digital_write(Low);
			delay_microseconds(450);
			pin.digital_write(High);
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
	pin.digital_write(Low);
	delay_microseconds(440);
	pin.digital_write(High);
	delay_microseconds(17100);
}

fn send_packet<P: Pin>(pin: &OutputPin<P>, bytes: &Vec<u8>) {
	send_header(pin);
	for byte in bytes { send_data_byte(pin, byte); }
	send_repeat_marker(pin);
	send_header(pin);
	for byte in bytes { send_data_byte(pin, byte); }
}

fn main() {

	match priority(0x99) {
		false => panic!("Can't set priority"),
		true => {}
	}
	let pi = wiringpi::setup_gpio();
	let pin = pi.output_pin(23);
	
	println!("Running");
	loop {
		send_packet(&pin, &vec![0x23,]);
		delay(100);
	}

}

