// #[doc = r"
// ESP32 Specific implementation for use with interrupts
// "]
use crate::hx711::{transmute, GainMode, HX711, HX711_DELAY_TIME_US, HX711_MAXIMUM, HX711_MINIMUM};
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

// impl<SckPin, DTPin, Delay> HX711<SckPin, DTPin, Delay>
// where
//     SckPin: OutputPin + embedded_hal::digital::v2::OutputPin<Error = Infallible>,
//     DTPin: InputPin + embedded_hal::digital::v2::InputPin<Error = Infallible>,
//     Delay: embedded_hal::blocking::delay::DelayUs<u32>,
// {
//     pub fn new(mut sck_pin: SckPin, mut dt_pin: DTPin, delay: Delay) -> Self {
//         // let _wrapped = _loadcell_hx711::new(sck_pin, _dt_pin, delay);
//         dt_pin.listen(Event::FallingEdge);
//         sck_pin.set_low().unwrap();

//         Self {
//             sck_pin,
//             dt_pin,
//             delay,
//             last_reading: 0,
//             gain_mode: GainMode::A64,
//             offset: 0,
//             scale: Some(0.0), // wrapped: _loadcell_hx711::new(sck_pin, _dt_pin, delay),
//         }
//     }

//     pub fn is_ready(&self) -> bool {
//         self.dt_pin.is_low().unwrap()
//     }

//     pub fn read_hx711_bit(&mut self, hx711_delay_time_us: u32) -> bool {
//         self.sck_pin.set_high().unwrap();
//         self.delay.delay_us(hx711_delay_time_us);

//         // read dt pin
//         let mut pin_state = true;
//         if self.dt_pin.is_low().unwrap() {
//             pin_state = false;
//         }

//         self.sck_pin.set_low().unwrap();
//         self.delay.delay_us(hx711_delay_time_us);

//         // return
//         pin_state
//     }

//     pub fn toggle_sck_bit(&mut self, hx711_delay_time_us: u32) {
//         self.sck_pin.set_high().unwrap();
//         self.delay.delay_us(hx711_delay_time_us);
//         self.sck_pin.set_low().unwrap();
//         self.delay.delay_us(hx711_delay_time_us);
//     }

//     pub fn set_gain_mode(&mut self, gain_mode: GainMode) {
//         self.gain_mode = gain_mode;
//     }

//     pub fn get_gain_mode(&self) -> GainMode {
//         self.gain_mode
//     }
// }

// impl<SckPin, DTPin, Delay> LoadCell for HX711<SckPin, DTPin, Delay>
// where
//     SckPin: OutputPin + embedded_hal::digital::v2::OutputPin<Error = Infallible>,
//     DTPin: InputPin + embedded_hal::digital::v2::InputPin<Error = Infallible>,
//     Delay: embedded_hal::blocking::delay::DelayUs<u32>,
// {
//     type Offset = u32;
//     type Scale = Option<f32>;

//     fn read(&mut self) -> i32 {
//         if !self.is_ready() {
//             // if let UseMode::INTERRUPT = self.use_mode {
//             //     self.dt_pin.clear_interrupt();
//             // }

//             return HX711_MINIMUM;
//         }

//         //data ready
//         let mut current_bit: u32;
//         let mut value: u32 = 0;
//         // read in data bits
//         for _ in 0..24 {
//             current_bit = self.read_hx711_bit(HX711_DELAY_TIME_US) as u32;
//             // bits arrive MSB first
//             value = (value << 1) | current_bit;
//         }
//         // send gain mode for next reading
//         let current_gain_mode = self.gain_mode as u8;
//         for _ in 0..current_gain_mode {
//             self.toggle_sck_bit(HX711_DELAY_TIME_US);
//         }

//         /* msb padding, if the 24 bit number is negative.
//          */
//         if value & 0b10000000_00000000_00000000 >= 1 {
//             // negative, fill with 1s
//             value |= 0b11111111_00000000_00000000_00000000_u32;
//         }
//         let mut signed = unsafe { transmute::<u32, i32>(value) };
//         // saturation
//         if signed < HX711_MINIMUM {
//             signed = HX711_MINIMUM;
//         } else if signed > HX711_MAXIMUM {
//             signed = HX711_MAXIMUM;
//         }
//         self.last_reading = signed;

//         // if let UseMode::INTERRUPT = self.use_mode {
//         //     self.dt_pin.clear_interrupt();
//         // }
//         // return
//         signed
//     }

//     fn get_offset(&self) -> Self::Offset {
//         self.offset as Self::Offset
//     }

//     fn get_scale(&self) -> Self::Scale {
//         self.scale as Self::Scale
//     }

//     fn read_scaled(&mut self) -> Self::Scale {
//         let raw = self.read();
//         self.scale.map(|x| raw as f32 * x)
//     }

//     fn set_scale(&mut self, scale: Self::Scale) {
//         self.scale = scale;
//     }

//     fn tare(&mut self, num_samples: usize) {
//         // use Interrupt;
//         self.deactivate_interrupt();
//         let mut current;
//         let mut average: f32 = 0.0;
//         for n in 1..=num_samples {
//             while !self.is_ready() {
//                 self.delay.delay_us(5000u32);
//                 // self.delay.delay_ms(5u32);
//             }
//             current = self.read() as f32;
//             // println!("Current = {current} average = {average}");
//             self.delay.delay_us(10u32 * 1000);
//             average += (current - average) / (n as f32);
//         }

//         self.offset = average as i32;

//         // self.activate_interrupt();
//     }
// }

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
