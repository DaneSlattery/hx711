//! HX 711 Polling Example
//!

#![no_std]
#![no_main]

use embedded_hal::delay::DelayNs;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, Level, Output, Pull},
    init, main,
};

use loadcell::{hx711, LoadCell};

#[main]
fn main() -> ! {
    let periph = init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    // setup the pins
    let hx711_sck = Output::new(periph.GPIO5, Level::Low);
    let hx711_dt = Input::new(periph.GPIO4, Pull::None);

    let mut delay = Delay::new();

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
    }
}
