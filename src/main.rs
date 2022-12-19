use bme280::{self, Measurements};
use linux_embedded_hal::{I2cdev, Delay, I2CError};
use embedded_hal::i2c::blocking::I2c;
use byteorder::{LittleEndian, ByteOrder};
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};
use warp::Filter;

struct LightSensor {
    addr:    u8,
    reg:     u8,
    bus: I2cdev,
}

impl LightSensor {
    fn new(bus: I2cdev, addr: u8, port: u8) -> Self {
        Self { bus, addr, reg: port + 0x10 }
    }

    fn measure(&mut self) -> Result<u16, I2CError> {
        self.bus.write(self.addr, &[self.reg])?;
        //thread::sleep(Duration::from_millis(10));
        let mut buf = [0; 2];
        self.bus.write_read(self.addr, &[self.reg], &mut buf[..])?;
        Ok(LittleEndian::read_u16(&buf))
    }

}

struct Station {
    light: LightSensor,
    bme280: bme280::i2c::BME280<I2cdev>,
}

impl Station {
    fn new() -> Result<Self> {

        let i2c = I2cdev::new("/dev/i2c-1")
            .map_err(|e| anyhow!("Could not open i2c bus: {}", e))?;

        // TODO properly share the device across the two consumers. Probably need to implement I2cdev for RefCell<impl I2cdev> or something
        // for now just open it twice :/
        let i2c2 = I2cdev::new("/dev/i2c-1")
            .map_err(|e| anyhow!("Could not open second i2c bus: {}", e))?;

        let mut bme280 = bme280::i2c::BME280::new_primary(i2c);
        bme280.init(&mut Delay).expect("Could not initialize BME280 sensor");

        let light = LightSensor::new(i2c2, 0x08, 0x00);

        Ok(Self { light, bme280 })
    }

    fn measure(&mut self) -> Result<(Measurements<I2CError>, u16)> {
        let baro = self.bme280.measure(&mut Delay)
            .map_err(|e| anyhow!("Cannot measure bme280: {:?}", e))?;
        let lux = self.light.measure()
            .map_err(|e| anyhow!("Cannot measure light: {:?}", e))?;
        Ok((baro, lux))
    }

    pub fn scrape(&mut self) -> String {

        let ( Measurements { temperature, pressure, humidity, .. }, lux) = match self.measure() {
            Err(e) => {
                return format!("# Error measuring: {:?}", e)
            },
            Ok(v) => v,
        };

        let lux = lux as f64 / 1000f64;

        eprintln!("Scraped data: T={temperature}°C P={pressure}Pa H={humidity}% L={lux}");

        format!(
"# HELP weather_temperature_celsius Temperature in °C
# TYPE weather_temperature_celsius gauge
weather_temperature_celsius {temperature}
# HELP weather_humidity_percent Humidity percentage
# TYPE weather_humidity_percent gauge
weather_humidity_percent {humidity}
# HELP weather_pressure_pascals Atmospheric pressure in Pascals
# TYPE weather_pressure_pascals gauge
weather_pressure_pascals {pressure}
# HELP weather_illumination_relative Relative illumination as a fraction of the maximum
# TYPE weather_illumination_relative gauge
weather_illumination_relative {lux}
")

    }

}

#[tokio::main(flavor="current_thread")]
async fn main() -> Result<()> {

    let home = format!("<html><body><h1>Weather Station v0.1</h1><ul><li><a href=\"https://git.xolus.net/max/weather_exporter\">source code</a></li><li><a href=\"/metrics\">metrics</a></li></ul></body></html>");
    let home: &'static str = Box::leak(home.into_boxed_str());

    let station = Station::new()?;
    let station = Arc::new(Mutex::new(station));

    let filter = warp::path!("metrics").map(move || station.lock().unwrap().scrape())
             .or(warp::path!().map(move || { warp::reply::html(home) }));

    eprintln!("Starting web server.");
    Ok(warp::serve(filter).run(([0,0,0,0], 9073)).await)


}
