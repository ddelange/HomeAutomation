extern crate linux_embedded_hal as hal;
extern crate bme280;

use hal::{Delay, I2cdev};
use bme280::BME280;

pub fn init() -> BME280<I2cdev, Delay> {
	// using Linux I2C Bus #1 in this example
	let i2c_bus = I2cdev::new("/dev/i2c-1").unwrap();
	// initialize the BME280 using the primary I2C address 0x77
	let mut bme280 = BME280::new_primary(i2c_bus, Delay);
	// initialize the sensor
	bme280.init().unwrap();
	bme280
}

pub fn measure_and_record(mut bme: BME280<I2cdev, Delay> ) -> (f32, f32, f32) {
	// measure temperature, pressure, and humidity
	let measurements = bme.measure().unwrap();

	dbg!(measurements.humidity);
	dbg!(measurements.temperature);
	dbg!(measurements.pressure);

	(measurements.humidity, measurements.temperature, measurements.pressure)
}

