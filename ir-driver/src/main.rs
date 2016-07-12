extern crate wiringpi;

use wiringpi::pin::Value::{High, Low};
use wiringpi::time::delay;

fn main() {

	let pi = wiringpi::setup_gpio();
	let pin = pi.output_pin(23);
	
	loop {
		pin.digital_write(High);
		delay(500);
		pin.digital_write(Low);
		delay(500);
	}

}

