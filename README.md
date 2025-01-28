# hx711
A no-std rust library for the hx711 targeting the ESP32.

## Overview

The HX711 is a Load Cell Amplifier. "By connecting the amplifier to your microcontroller you will be able to read the changes in the resistance of the load cell, and with some calibration you’ll be able to get very accurate weight measurements. This can be handy for creating your own industrial scale, process control or simple presence detection." 

This driver implements the requisite bit-banging described in this [datasheet](https://cdn.sparkfun.com/assets/b/f/5/a/e/hx711F_EN.pdf?_gl=1*1yuadp6*_ga*MTY0Mzk3NTc1MS4xNjkxMzU4OTYx*_ga_T369JS7J9N*MTcwMjU4MzgzMC4xNi4wLjE3MDI1ODM4MzAuNjAuMC4w), and abstracts away the fine details to provide a generic `LoadCell` interface, with the ability to scale measurements (calibration) and offset measurements (tare).

This library makes extensive use of the `embedded-hal`, and the entire blocking approach is generic enough for any device supported by the `embedded-hal` (testing help greatly appreciated). The interrupt approach is currently implemented only for the esp32, making use of the `esp32-hal`.

For more information about the hardware and hookup, see this comprehensive guide from [sparkfun](https://www.sparkfun.com/products/13879).

## Installation

This crate is on [crates.io](https://crates.io/crates/loadcell), which means it can be installed by including this line in your `cargo.toml`:

```
loadcell = "0.3.0"
```

## Usage

The `/examples/` folder provides the key usage of the library.

The constructor takes ownership of the provided pins:

```
let mut load_sensor = hx711::HX711::new(hx711_sck, hx711_dt, delay);
```

Then this `load_sensor` object can be used like:
```
// zero the sensor
load_sensor.tare(16);
// set the sensitivity/scale
load_sensor.set_scale(1.0);

loop {
    if load_sensor.is_ready() {
        let reading = load_sensor.read_scaled();
        esp_println::println!("Last Reading = {:?}", reading);
    }
    delay.delay_ms(5u32);
}
```

### Calibration

The output of the loadcell is assumed to be a linear function mapping the raw measurements (`x`) to the output measurement (`y`). When the load cell is initialised, it has some offset (`c`). The device must be calibrated to determine the sensitivity (`m`), which can be done by applying the following formula:

```
y = mx + c
```

First, the offset (`c`) can be determined by "zeroing" the load cell. The `tare` function is used to do this, and calculates the offset automatically using `num_samples` samples.

Secondly, and this should be done for each individual load cell, the `m` value can be calculated by placing known masses on the scale. If you know the value of `y`, `x` and `c`, the value of `m` can be calculated by:

```
m = (y-c)/x
```

Personally, I start with `m = 1`, and adjust it until `y` matches the mass on the scale. 


