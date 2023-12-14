#[cfg(feature = "default")]
use core::fmt;

use core::fmt::Display;
use core::mem::transmute;

#[cfg(feature = "esp32_interrupt")]
pub mod interrupt;
#[cfg(feature = "esp32_interrupt")]
pub use interrupt::*;

use embedded_hal::blocking::delay::DelayUs;

use embedded_hal::digital::v2::{InputPin, OutputPin};

#[cfg(feature = "default")]
use crate::LoadCell;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum GainMode {
    A128 = 1, // extra pulses
    B32 = 2,
    A64 = 3,
}

pub const HX711_MINIMUM: i32 = -(2i32.saturating_pow(24 - 1));
pub const HX711_MAXIMUM: i32 = 2i32.saturating_pow(24 - 1) - 1;
const HX711_DELAY_TIME_US: u32 = 1;

const HX711_TARE_DELAY_TIME_US: u32 = 5000;
const HX711_TARE_SLEEP_TIME_US: u32 = 10000;

pub struct HX711<SckPin, DTPin, Delay>
where
    SckPin: OutputPin,
    DTPin: InputPin,
    Delay: DelayUs<u32>,
{
    sck_pin: SckPin,
    dt_pin: DTPin,
    delay: Delay,
    last_reading: i32,
    gain_mode: GainMode,
    offset: i32, // tare
    scale: f32,  // calibration value,
}

#[derive(Debug)]
pub struct NotReadyError;

impl Display for NotReadyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Device not ready to read")
    }
}

#[cfg(feature = "default")]
impl<SckPin, DTPin, Delay, ESCK, EDT> HX711<SckPin, DTPin, Delay>
where
    SckPin: OutputPin<Error = ESCK>,
    DTPin: InputPin<Error = EDT>,
    Delay: DelayUs<u32>,
    EDT: fmt::Debug,
    ESCK: fmt::Debug,
{
    /// Constructs a new hx711 driver, taking ownership of the pins.
    pub fn new(mut sck_pin: SckPin, dt_pin: DTPin, delay: Delay) -> Self {
        sck_pin.set_low().unwrap();
        Self {
            sck_pin,
            dt_pin,
            delay,
            last_reading: 0,
            gain_mode: GainMode::A64,
            offset: 0,
            scale: 1.0,
        }
    }

    /// Returns true if the load cell amplifier has a value ready to be read.
    pub fn is_ready(&self) -> bool {
        self.dt_pin.is_low().unwrap()
    }

    fn read_hx711_bit(&mut self, hx711_delay_time_us: u32) -> bool {
        self.sck_pin.set_high().unwrap();
        self.delay.delay_us(hx711_delay_time_us);

        // read dt pin
        let mut pin_state = true;
        if self.dt_pin.is_low().unwrap() {
            pin_state = false;
        }

        self.sck_pin.set_low().unwrap();
        self.delay.delay_us(hx711_delay_time_us);

        // return
        pin_state
    }

    fn toggle_sck_bit(&mut self, hx711_delay_time_us: u32) {
        self.sck_pin.set_high().unwrap();
        self.delay.delay_us(hx711_delay_time_us);
        self.sck_pin.set_low().unwrap();
        self.delay.delay_us(hx711_delay_time_us);
    }

    /// Set the gain mode for the next reading.
    pub fn set_gain_mode(&mut self, gain_mode: GainMode) {
        self.gain_mode = gain_mode;
    }

    /// Get the gain mode.
    pub fn get_gain_mode(&self) -> GainMode {
        self.gain_mode
    }

    fn read_bits(&mut self) -> i32 {
        let mut current_bit: u32;
        let mut value: u32 = 0;
        // read in data bits
        for _ in 0..24 {
            current_bit = self.read_hx711_bit(HX711_DELAY_TIME_US) as u32;
            // bits arrive MSB first
            value = (value << 1) | current_bit;
        }
        // send gain mode for next reading
        let current_gain_mode = self.gain_mode as u8;
        for _ in 0..current_gain_mode {
            self.toggle_sck_bit(HX711_DELAY_TIME_US);
        }

        /* msb padding, if the 24 bit number is negative.
         */
        if value & 0b10000000_00000000_00000000 >= 1 {
            // negative, fill with 1s
            value |= 0b11111111_00000000_00000000_00000000_u32;
        }
        let mut signed = unsafe { transmute::<u32, i32>(value) };
        // saturation
        if signed < HX711_MINIMUM {
            signed = HX711_MINIMUM;
        } else if signed > HX711_MAXIMUM {
            signed = HX711_MAXIMUM;
        }

        signed
    }
}

#[cfg(feature = "default")]
impl<SckPin, DTPin, Delay, ESCK, EDT> LoadCell for HX711<SckPin, DTPin, Delay>
where
    SckPin: OutputPin<Error = ESCK>,
    DTPin: InputPin<Error = EDT>,
    Delay: DelayUs<u32>,
    ESCK: fmt::Debug,
    EDT: fmt::Debug,
{
    type Offset = i32;
    type Scale = f32;

    type NotReadyError = NotReadyError;

    fn read(&mut self) -> Result<i32, Self::NotReadyError> {
        // TODO: change this to return an option or error if the device is not ready.
        if !self.is_ready() {
            return Err(NotReadyError);
        }
        let signed = self.read_bits();

        self.last_reading = signed - self.offset;

        Ok(self.last_reading)
    }

    fn get_offset(&self) -> Self::Offset {
        self.offset as Self::Offset
    }

    fn get_scale(&self) -> Self::Scale {
        self.scale as Self::Scale
    }

    fn read_scaled(&mut self) -> Result<Self::Scale, NotReadyError> {
        let raw = self.read()?;
        Ok(raw as f32 * self.scale)
    }

    fn set_scale(&mut self, scale: Self::Scale) {
        self.scale = scale;
    }

    fn tare(&mut self, num_samples: usize) {
        let mut current;
        let mut average: f32 = 0.0;
        for n in 1..=num_samples {
            while !self.is_ready() {
                self.delay.delay_us(HX711_TARE_DELAY_TIME_US);
            }
            current = self.read_bits() as f32;
            self.delay.delay_us(HX711_TARE_SLEEP_TIME_US);
            average += (current - average) / (n as f32);
        }

        self.offset = average as Self::Offset;
    }
}
