use bme280;
use linux_embedded_hal::{I2cdev, Delay};
use std::thread;
use std::time::Duration;

fn main() {

    let loop_time = Duration::from_secs(5);

    let i2c = I2cdev::new("/dev/i2c-1")
        .expect("Could not open i2c bus");

    let mut sensor = bme280::i2c::BME280::new_primary(i2c);

    sensor.init(&mut Delay).expect("Could not initialize BME280 sensor");

    loop {
        let baro = sensor.measure(&mut Delay).expect("Could not perform measurement");
        println!("H {}%, P {} PA, T {}°C", baro.humidity, baro.pressure, baro.temperature);
        thread::sleep(loop_time);        
    }

}
