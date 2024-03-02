//! HX 711 Interrupt Example
//!
//!
//! Uses the Interrupt feature on the esp32 to
//! activate the reading of the hx711 only when the
//! dt pin goes low (is ready), rather than polling it in main.

#![no_std]
#![no_main]

use core::cell::Cell;
use core::{borrow::BorrowMut, cell::RefCell};

use critical_section::Mutex;

use esp_backtrace as _;
use esp_println::println;

use esp32_hal::{
    clock::ClockControl,
    gpio::{Floating, Gpio4, Gpio5, Input, Output, PushPull, IO},
    interrupt,
    peripherals::{self, Peripherals},
    prelude::*,
    Delay,
};

use loadcell::hx711::{Interrupt, HX711};
use loadcell::LoadCell;

type SckPin = Gpio5<Output<PushPull>>;
type DTPin = Gpio4<Input<Floating>>;

// mutex to access during interrupt and in main
static HX711_READING_MUTEX: Mutex<Cell<i32>> = Mutex::new(Cell::new(0));
// mutex to access during interrupt and in main
static HX711_MUTEX: Mutex<RefCell<Option<HX711<SckPin, DTPin, Delay>>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    // Set GPIO15 as an output, and set its state high initially.
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let hx711_sck = io.pins.gpio5.into_push_pull_output();

    let hx711_dt = io.pins.gpio4.into_floating_input();
    let mut load_sensor = HX711::new(hx711_sck, hx711_dt, delay);

    load_sensor.tare_sync(20);
    println!("Tare = {}", load_sensor.get_offset());
    critical_section::with(|cs| {
        HX711_MUTEX.borrow_ref_mut(cs).replace(load_sensor);
        // activate interrupt during critical section
        HX711_MUTEX
            .borrow_ref_mut(cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .enable_interrupt();
    });

    interrupt::enable(peripherals::Interrupt::GPIO, interrupt::Priority::Priority2).unwrap();

    loop {
        critical_section::with(|cs| {
            println!("Last Reading = {}", HX711_READING_MUTEX.borrow(cs).get());
        });

        delay.delay_ms(50u32);
    }
}

#[ram]
#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        let mut bind = HX711_MUTEX.borrow_ref_mut(cs);
        let hx711 = bind.borrow_mut().as_mut().unwrap();
        if hx711.is_ready() {
            HX711_READING_MUTEX.borrow(cs).set(hx711.read().unwrap());
        }
        hx711.clear_interrupt();
    });
}
