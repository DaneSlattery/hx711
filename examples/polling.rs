//! HX 711 Polling Example
//!

#![no_std]
#![no_main]

use embedded_hal::delay::DelayNs;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{Input, Io, Level, Output, Pull},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};
use loadcell::{hx711, LoadCell};

#[entry]
fn main() -> ! {
    let periph = Peripherals::take();
    let system = SystemControl::new(periph.SYSTEM);

    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = Io::new(periph.GPIO, periph.IO_MUX);

    // setup the pins
    let hx711_sck = Output::new(io.pins.gpio5, Level::Low);
    let hx711_dt = Input::new(io.pins.gpio4, Pull::None);

    let mut delay = Delay::new(&clocks);

    // create the load sensor
    let mut load_sensor = hx711::HX711::new(hx711_sck, hx711_dt, delay);
    // zero the readings
    load_sensor.tare(16);

    load_sensor.set_scale(1.0);

    loop {
        if load_sensor.is_ready() {
            let reading = load_sensor.read_scaled();
            if let Ok(x) = reading {
                esp_println::println!("Last Reading = {:?}", x)
            }
        }
        delay.delay_ms(5u32);
        // delay.delay_ms(5u32);
    }
}
