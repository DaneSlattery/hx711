// #[doc = r"
// ESP32 Specific implementation for use with interrupts
// "]
use crate::hx711::HX711;
use crate::LoadCell;

use core::convert::Infallible;

use esp32_hal::gpio::{Event, InputPin, OutputPin};

#[cfg(feature = "esp32_interrupt")]
pub trait Interrupt: LoadCell {
    /// Tare the loadcell synchronously only, disabling interrupts,
    /// and if they were previously enabled, re-enabling them after
    /// the blocking tare.
    fn tare_sync(&mut self, num_samples: usize);

    fn disable_interrupt(&mut self);

    fn enable_interrupt(&mut self);

    fn clear_interrupt(&mut self);
}

#[cfg(feature = "esp32_interrupt")]
impl<SckPin, DTPin, Delay> Interrupt for HX711<SckPin, DTPin, Delay>
where
    SckPin: OutputPin + embedded_hal::digital::v2::OutputPin<Error = Infallible>,
    DTPin: InputPin + embedded_hal::digital::v2::InputPin<Error = Infallible>,
    Delay: embedded_hal::blocking::delay::DelayUs<u32>,
{
    fn tare_sync(&mut self, num_samples: usize) {
        let mut was_listening = false;
        if self.dt_pin.is_listening() {
            was_listening = true;
        }
        self.disable_interrupt();
        <HX711<SckPin, DTPin, Delay> as LoadCell>::tare(self, num_samples);
        if was_listening {
            self.enable_interrupt();
        }
    }

    fn clear_interrupt(&mut self) {
        self.dt_pin.clear_interrupt();
    }

    fn disable_interrupt(&mut self) {
        self.dt_pin.unlisten();
    }

    fn enable_interrupt(&mut self) {
        self.dt_pin.listen(Event::FallingEdge);
    }
}
