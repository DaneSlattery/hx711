//! HX 711 Library
//!
//! This prints "Interrupt" when the boot button is pressed.
//! It also blinks an LED like the blinky example.

#![no_std]
#![no_main]

use core::cell::{Cell, RefMut};
use core::convert::Infallible;
use core::fmt;
use core::{borrow::BorrowMut, cell::RefCell};
use critical_section::Mutex;

use esp_backtrace as _;
use esp_println::println;

use esp32_hal::{
    clock::ClockControl,
    gpio::{
        Event, Floating, Gpio16, Gpio4, GpioPin, GpioProperties, Input, InputPin, Output,
        OutputPin, PushPull, IO,
    },
    interrupt,
    peripherals::{self, Peripherals},
    prelude::*,
    Delay,
};
use loadcell::hx711::{GainMode, Interrupt, HX711};
use loadcell::LoadCell;
// mod hx711;
type SckPin = Gpio4<Output<PushPull>>;
type DTPin = Gpio16<Input<Floating>>;
type ESCK = Infallible;
type EDT = Infallible;

static HX711_READING_MUTEX: Mutex<Cell<i32>> = Mutex::new(Cell::new(0));
// static DT_PIN_MUTEX: Mutex<RefCell<Option<DTPin>>> = Mutex::new(RefCell::new(None));
static HX711_MUTEX: Mutex<RefCell<Option<HX711<SckPin, DTPin, Delay>>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    // Set GPIO15 as an output, and set its state high initially.
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let hx711_sck = io.pins.gpio4.into_push_pull_output();

    let mut hx711_dt = io.pins.gpio16.into_floating_input();
    let mut load_sensor = HX711::new(hx711_sck, hx711_dt, delay);
    // load_sensor.disable_interrupt(); // make sure interrupts are disabled when reading manually.

    load_sensor.tare_sync(10); // load_sensor.tare();
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

    // hx711_dt.listen(Event::FallingEdge);
    // hx711_sck.unlisten();
    // load_sensor.tare(10);
    // println!("Tare = {}", load_sensor.get_offset());

    // critical_section::with(|cs| {
    // HX711_MUTEX.borrow_ref_mut(cs).replace(load_sensor);
    //     // DT_PIN_MUTEX.borrow_ref_mut(cs).replace(hx711_dt);
    //     //
    // });

    interrupt::enable(peripherals::Interrupt::GPIO, interrupt::Priority::Priority2).unwrap();

    loop {
        critical_section::with(|cs| {
            println!(
                "Last Reading = {}",
                HX711_READING_MUTEX.borrow(cs).get() // HX711_MUTEX.borrow_ref_mut(cs).as_mut().unwrap().get_last()
            );
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
            HX711_READING_MUTEX.borrow(cs).set(hx711.read());
        }
        hx711.clear_interrupt();
        // hx711.0.read();
        //     DT_PIN_MUTEX
        //         .borrow_ref_mut(cs)
        //         .borrow_mut()
        //         .as_mut()
        //         .unwrap()
        //         .clear_interrupt();
        // }
    });
}
