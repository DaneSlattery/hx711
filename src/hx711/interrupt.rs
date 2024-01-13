//! ESP32 Specific implementation for use with interrupts

use crate::hx711::HX711;
use crate::LoadCell;

use core::convert::Infallible;

use esp32_hal::gpio::{Event, InputPin, OutputPin};

/// An extension of the `LoadCell` interface that can be used instead
/// of polling, rather listening on the DT pin for
/// pin change events.
#[cfg(feature = "esp32_interrupt")]
pub trait Interrupt: LoadCell {
    /// Tare the loadcell synchronously only, disabling interrupts,
    /// and if they were previously enabled, re-enabling them after
    /// the blocking tare.
    fn tare_sync(&mut self, num_samples: usize);

    /// Disable the pin change interrupt on the DT pin.
    fn disable_interrupt(&mut self);

    /// Listen for pin change events on the DT pin.
    fn enable_interrupt(&mut self);

    /// Clear the status interrupt bit for the DT pin.
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
