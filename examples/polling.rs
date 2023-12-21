//! HX 711 Polling Example
//!

#![no_std]
#![no_main]

use esp32_hal::{clock::ClockControl, entry, peripherals::Peripherals, prelude::*, Delay, IO};
use esp_backtrace as _;
use loadcell::{hx711, LoadCell};

#[entry]
fn main() -> ! {
    let periph = Peripherals::take();
    let system = periph.DPORT.split();

    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = IO::new(periph.GPIO, periph.IO_MUX);

    // setup the pins
    let hx711_sck = io.pins.gpio5.into_push_pull_output();
    let hx711_dt = io.pins.gpio4.into_floating_input();

    let mut delay = Delay::new(&clocks);

    // create the load sensor
    let mut load_sensor = hx711::HX711::new(hx711_sck, hx711_dt, delay);
    // zero the readings
    load_sensor.tare(16);

    load_sensor.set_scale(1.0);

    loop {
        if load_sensor.is_ready() {
            let reading = load_sensor.read_scaled();
            match reading {
                Ok(x) => esp_println::println!("Last Reading = {:?}", x),
                Err(_) => (),
            }
        }
        delay.delay_ms(5u32);
    }
}
