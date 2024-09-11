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

use embedded_hal::digital::{ErrorType, InputPin};
use esp_backtrace as _;
use esp_hal::gpio::{Event, Level, Pull};
use esp_hal::system::SystemControl;
use esp_println::println;

use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{Gpio4, Gpio5, Input, Io, Output},
    peripherals::Peripherals,
    prelude::*,
};

use loadcell::hx711::HX711;
use loadcell::LoadCell;

type SckPin<'a> = Output<'a, Gpio5>; // Gpio5<Output<PushPull>>;
type DTPin<'a> = Input<'a, Gpio4>; //Gpio4<Input<Floating>>;
type EspHX711<'a> = HX711<SckPin<'a>, &'a DTPinWrapper<'a>, Delay>;
// mutex to access during interrupt and in main
static HX711_READING_MUTEX: Mutex<Cell<i32>> = Mutex::new(Cell::new(0));
// mutex to access during interrupt and in main
static HX711_MUTEX: Mutex<RefCell<Option<EspHX711>>> = Mutex::new(RefCell::new(None));

struct DTPinWrapper<'a> {
    inner: Mutex<RefCell<Option<DTPin<'a>>>>,
}

impl InputPin for &DTPinWrapper<'_> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        critical_section::with(|cs| self.inner.borrow_ref_mut(cs).as_mut().unwrap().is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        critical_section::with(|cs| self.inner.borrow_ref_mut(cs).as_mut().unwrap().is_low())
    }
}

impl<'a> ErrorType for DTPinWrapper<'a> {
    type Error = <DTPin<'a> as ErrorType>::Error;
}

impl InputPin for DTPinWrapper<'_> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        critical_section::with(|cs| self.inner.borrow_ref_mut(cs).as_mut().unwrap().is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        critical_section::with(|cs| self.inner.borrow_ref_mut(cs).as_mut().unwrap().is_low())
    }
}
// static HX711_DT_MUTEX: Mutex<RefCell<Option<DTPin>>> = Mutex::new(RefCell::new(None));
// // mutex to access during interrupt and in main
static HX711_DT_MUTEX: DTPinWrapper = DTPinWrapper {
    inner: Mutex::new(RefCell::new(None)),
};

#[entry]
fn main() -> ! {
    let periph = Peripherals::take();
    let system = SystemControl::new(periph.SYSTEM);

    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    let mut io = Io::new(periph.GPIO, periph.IO_MUX);
    io.set_interrupt_handler(handler);

    // setup the pins
    let hx711_sck = Output::new(io.pins.gpio5, Level::Low);
    let hx711_dt = Input::new(io.pins.gpio4, Pull::None);

    critical_section::with(|cs| {
        HX711_DT_MUTEX.inner.borrow_ref_mut(cs).replace(hx711_dt);
        let mut load_sensor = HX711::new(hx711_sck, &HX711_DT_MUTEX, delay);

        HX711_DT_MUTEX
            .inner
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .listen(Event::FallingEdge);

        load_sensor.tare(20);
        println!("Tare = {}", load_sensor.get_offset());

        HX711_MUTEX.borrow_ref_mut(cs).replace(load_sensor);
    });

    loop {
        critical_section::with(|cs| {
            println!("Last Reading = {}", HX711_READING_MUTEX.borrow(cs).get());
        });

        delay.delay_millis(50u32);
    }
}

#[handler]
#[ram]
fn handler() {
    critical_section::with(|cs| {
        let mut bind = HX711_MUTEX.borrow_ref_mut(cs);
        let hx711 = bind.borrow_mut().as_mut().unwrap();
        if hx711.is_ready() {
            HX711_READING_MUTEX.borrow(cs).set(hx711.read().unwrap());
        }

        HX711_DT_MUTEX
            .inner
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}
