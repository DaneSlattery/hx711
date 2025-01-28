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
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Event, Level, Pull};
use esp_hal::interrupt::InterruptConfigurable;
use esp_hal::{handler, ram};
use esp_println::println;

use esp_hal::{
    delay::Delay,
    gpio::{Input, Io, Output},
    main,
};

use loadcell::hx711::HX711;
use loadcell::LoadCell;

type SckPin<'a> = Output<'a>;
type DTPin<'a> = Input<'a>;

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

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());

    let peripherals = esp_hal::init(config);

    let mut io = Io::new(peripherals.IO_MUX);
    io.set_interrupt_handler(handler);

    // setup the pins
    let hx711_sck = Output::new(peripherals.GPIO5, Level::Low);
    let hx711_dt = Input::new(peripherals.GPIO4, Pull::None);
    let delay = Delay::new();

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
