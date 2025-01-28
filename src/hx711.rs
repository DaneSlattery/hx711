//! HX711 Load Cell Amplifier
use core::fmt;

use core::fmt::Display;
use core::mem::transmute;

use embedded_hal::delay::DelayNs;

use embedded_hal::digital::{InputPin, OutputPin};

use crate::LoadCell;

/// The HX711 has different amplifier gain settings.
/// The choice of gain settings is controlled by writing a fixed number of
/// extra pulses after a read.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum GainMode {
    /// Amplification gain of 128 on channel A.
    A128 = 1, // extra pulses
    /// Amplification gain of 32 on channel B.
    B32 = 2,
    /// Amplification gain of 64 on channel A.
    A64 = 3,
}

/// The absolute minimum readings. A smaller value should be clamped.
pub const HX711_MINIMUM: i32 = -(2i32.saturating_pow(24 - 1));
/// The absolute maximum readings. A greater value should be clamped.
pub const HX711_MAXIMUM: i32 = 2i32.saturating_pow(24 - 1) - 1;

const HX711_DELAY_TIME_US: u32 = 1;

const HX711_TARE_DELAY_TIME_US: u32 = 5000;
const HX711_TARE_SLEEP_TIME_US: u32 = 10000;

/// Device driver for the HX711 load cell amplifier.
pub struct HX711<SckPin, DTPin, Delay> {
    sck_pin: SckPin,
    dt_pin: DTPin,
    delay: Delay,
    last_reading: i32,
    gain_mode: GainMode,
    offset: i32, // tare
    scale: f32,  // calibration value,
}

#[derive(Debug)]
/// Error returned when reading from the device before it has a new reading available.
pub struct NotReadyError;

impl Display for NotReadyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Device not ready to read")
    }
}

impl<SckPin, DTPin, Delay, ESCK, EDT> HX711<SckPin, DTPin, Delay>
where
    SckPin: OutputPin<Error = ESCK>,
    DTPin: InputPin<Error = EDT>,
    Delay: DelayNs,
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
    pub fn is_ready(&mut self) -> bool {
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
        critical_section::with(|_| {
            self.sck_pin.set_high().unwrap();
            self.delay.delay_us(hx711_delay_time_us);
            self.sck_pin.set_low().unwrap();
            self.delay.delay_us(hx711_delay_time_us);
        });
    }

    /// Set the gain mode for the next reading.
    pub fn set_gain_mode(&mut self, gain_mode: GainMode) {
        self.gain_mode = gain_mode;
    }

    /// Get the gain mode.
    pub fn get_gain_mode(&self) -> GainMode {
        self.gain_mode
    }

    /// The reading must run in a critical section to prevent other interrupts from altering the SCK timing.
    /// If an interrupt occurs during the time the SCK signal is high, it will stretch the length of the clock pulse.
    /// If the total pulse time exceeds 60 us, this will cause the HX711 to enter power down mode during the middle of
    /// the read sequence. While the device will wake up when PD_SCK goes low again, the reset starts a new conversion
    /// cycle which forces DT high until that cycle is completed. The result is that all subsequent bits read by
    /// this function will read back as 1, corrupting the returned value.
    fn read_bits(&mut self) -> i32 {
        // read in data bits
        let mut value = critical_section::with(|_| {
            let mut value: u32 = 0;
            let mut current_bit: u32;
            for _ in 0..24 {
                current_bit = self.read_hx711_bit(HX711_DELAY_TIME_US) as u32;
                // bits arrive MSB first
                value = (value << 1) | current_bit;
            }
            value
        });
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
        signed = signed.clamp(HX711_MINIMUM, HX711_MAXIMUM);

        signed
    }
}

impl<SckPin, DTPin, Delay, ESCK, EDT> LoadCell for HX711<SckPin, DTPin, Delay>
where
    SckPin: OutputPin<Error = ESCK>,
    DTPin: InputPin<Error = EDT>,
    Delay: DelayNs,
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

    fn set_offset(&mut self, offset: Self::Offset) {
        self.offset = offset;
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
