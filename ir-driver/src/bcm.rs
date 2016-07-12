extern crate libc;
use libc::{c_int, c_uint};

#[link(name="bcm2835")]
extern {
	fn bcm2835_init() -> c_int;
	fn bcm2835_gpio_fsel(pin: u8, mode: u8);
	fn bcm2835_gpio_write(pin: u8, mode: u8);
	fn bcm2835_delay(millis: c_uint);
	fn bcm2835_close() -> c_int;
	fn bcm2835_delayMicroseconds(micros: u64);
}

fn main() {
	const PIN: u8 = 23;

	match unsafe {bcm2835_init()} {
		0 => panic!("bcm init failed"),
		_ => {}
	}

	unsafe {bcm2835_gpio_fsel(PIN, 0x01)};

	loop {
		unsafe {bcm2835_gpio_write(PIN, 0x01)};
		unsafe {bcm2835_delay(500)};
		unsafe {bcm2835_gpio_write(PIN, 0x00)};
		unsafe {bcm2835_delay(500)};
	}

	unsafe {bcm2835_close()};

	println!("hello world!");
}
