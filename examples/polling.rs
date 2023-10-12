#![no_std]
#![no_main]

use esp_backtrace as _;
use hal::{
    clock::ClockControl,
    entry,
    peripherals::{self, Peripherals},
    prelude::*,
    Delay, IO,
};
use loadcell::{hx711, LoadCell};

#[entry]
fn main() -> ! {
    let periph = Peripherals::take();

    let system = periph.DPORT.split();

    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = IO::new(periph.GPIO, periph.IO_MUX);

    let hx711_sck = io.pins.gpio4.into_push_pull_output();
    let hx711_dt = io.pins.gpio16.into_floating_input();

    let mut delay = Delay::new(&clocks);
    let mut load_sensor = hx711::HX711::new(hx711_sck, hx711_dt, delay);

    load_sensor.tare(16);
    loop {
        let reading = load_sensor.read_scaled();
        delay.delay_ms(5);
    }
}
