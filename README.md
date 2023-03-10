# Weather Station exporter for Prometheus

A simple, zero-configuration binary for exporting weather data to Prometheus.

This is designed to run on a Raspberry Pi[1] equipped with a Seeed Grove base hat[2],
with a Grove Light Sensor[3] connected to port A0, and a BME280 Barometer sensor[4] connected
to any I2C port.

The exporter listens on `0.0.0.0:9073` and exports these 4 gauges:

 - `weather_temperature_celsius`
 - `weather_humidity_percent`
 - `weather_pressure_pascals`
 - `weather_illumination_relative`

The last value is between 0 (pitch dark) and 1 (maximum brightness)

[1]: https://www.raspberrypi.org/
[2]: https://wiki.seeedstudio.com/Grove_Base_Hat_for_Raspberry_Pi/
[3]: https://wiki.seeedstudio.com/Grove-Light_Sensor/
[4]: https://wiki.seeedstudio.com/Grove-Barometer_Sensor-BME280/
