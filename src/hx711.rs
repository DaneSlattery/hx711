use core::mem::transmute;

use esp_println::println;
use hal::gpio::Pin;
use hal::gpio::{
    Event, Floating, GpioPin, GpioProperties, Input, IsInputPin, IsOutputPin, Output, PushPull,
};
use hal::prelude::_embedded_hal_digital_v2_InputPin;
use hal::prelude::_embedded_hal_digital_v2_OutputPin;
use hal::prelude::{_embedded_hal_blocking_delay_DelayMs, _embedded_hal_blocking_delay_DelayUs};
use hal::Delay;

pub const HX711_MINIMUM: i32 = -(2i32.saturating_pow(24 - 1));
pub const HX711_MAXIMUM: i32 = 2i32.saturating_pow(24 - 1) - 1;
const HX711_DELAY_TIME_US: u32 = 1;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum GainMode {
    A128 = 1, // extra pulses
    B32 = 2,
    A64 = 3,
}
impl GainMode {
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

pub enum UseMode {
    INTERRUPT,
    SYNC,
}

pub struct HX711<const SCK_PIN: u8, const DT_PIN: u8>
where
    GpioPin<Output<PushPull>, SCK_PIN>: GpioProperties,
    <GpioPin<Output<PushPull>, SCK_PIN> as GpioProperties>::PinType: IsOutputPin,
    GpioPin<Input<Floating>, DT_PIN>: GpioProperties,
    <GpioPin<Input<Floating>, DT_PIN> as GpioProperties>::PinType: IsInputPin,
{
    sck_pin: GpioPin<Output<PushPull>, SCK_PIN>,
    dt_pin: GpioPin<Input<Floating>, DT_PIN>,
    delay: Delay,
    last_reading: i32,
    gain_mode: GainMode,
    offset: i32, // tare
    scale: f32,  // calibration value,
    use_mode: UseMode,
}

impl<const SCK_PIN: u8, const DT_PIN: u8> HX711<SCK_PIN, DT_PIN>
where
    GpioPin<Output<PushPull>, SCK_PIN>: GpioProperties,
    <GpioPin<Output<PushPull>, SCK_PIN> as GpioProperties>::PinType: IsOutputPin,
    GpioPin<Input<Floating>, DT_PIN>: GpioProperties,
    <GpioPin<Input<Floating>, DT_PIN> as GpioProperties>::PinType: IsInputPin,
{
    pub fn new(
        mut sck_pin: GpioPin<Output<PushPull>, SCK_PIN>,
        mut dt_pin: GpioPin<Input<Floating>, DT_PIN>,
        delay: &Delay,
    ) -> Self {
        sck_pin.set_low().unwrap();
        dt_pin.listen(Event::FallingEdge);
        Self {
            sck_pin,
            dt_pin,
            delay: *delay,
            last_reading: 0,
            gain_mode: GainMode::A128,
            offset: 0,
            scale: 1.0,
            use_mode: UseMode::INTERRUPT,
        }
    }
    /// listen for interrupt events
    pub fn activate_interrupt(&mut self) {
        // self.sck_pin.set_low().unwrap();
        self.use_mode = UseMode::INTERRUPT;
        // if let UseMode::INTERRUPT = self.use_mode {
        // } else {
        //     panic!("Hx711 Peripheral not in interrupt mode.")
        // }
        if self.dt_pin.is_listening() {
            self.dt_pin.unlisten();
        }
        self.dt_pin.listen(Event::FallingEdge);
    }

    /// listen for interrupt events
    pub fn deactivate_interrupt(&mut self) {
        // self.sck_pin.set_low().unwrap();
        if let UseMode::INTERRUPT = self.use_mode {
        } else {
            panic!("Hx711 Peripheral not in interrupt mode.")
        }
        if self.dt_pin.is_listening() {
            self.dt_pin.unlisten();
        }
        self.use_mode = UseMode::SYNC;
        // self.dt_pin.listen(Event::FallingEdge);
    }

    pub fn is_ready(&self) -> bool {
        // if the dt pin is low, device is ready for read
        self.dt_pin.is_low().unwrap()
    }

    pub fn read(&mut self) -> i32 {
        if !self.is_ready() {
            if let UseMode::INTERRUPT = self.use_mode {
                self.dt_pin.clear_interrupt();
            }

            return HX711_MINIMUM;
        }

        //data ready
        let mut current_bit: u32;
        let mut value: u32 = 0;
        // read in data bits
        for _ in 0..24 {
            current_bit = self.read_hx711_bit(HX711_DELAY_TIME_US) as u32;
            // bits arrive MSB first
            value = (value << 1) | current_bit;
        }
        // send gain mode for next reading
        let current_gain_mode = self.gain_mode.discriminant();
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
        self.last_reading = signed;

        if let UseMode::INTERRUPT = self.use_mode {
            self.dt_pin.clear_interrupt();
        }
        // return
        signed
    }

    /// Get last reading, offset relative to the tare, and scaled.
    pub fn get_last(&self) -> f32 {
        (self.last_reading - self.offset) as f32 / self.scale
    }

    pub fn get_last_raw(&self) -> i32 {
        self.last_reading
    }

    pub fn get_gain_mode(&self) -> GainMode {
        self.gain_mode
    }

    pub fn set_gain_mode(&mut self, new_mode: GainMode) {
        self.gain_mode = new_mode;
    }

    /// zero the scale by taking 10 manual measurements, blocking.
    pub fn tare(&mut self) {
        self.deactivate_interrupt();
        let mut current = 0.0;
        let mut average: f32 = 0.0;
        for n in 1..=10 {
            while !self.is_ready() {
                self.delay.delay_ms(5u32);
            }
            current = self.read() as f32;
            println!("Current = {current} average = {average}");
            self.delay.delay_ms(10u32);
            average += (current - average) / (n as f32);
        }

        self.offset = average as i32;

        self.activate_interrupt();
    }
    pub fn get_scale(&self) -> f32 {
        return self.scale;
    }
    pub fn get_offset(&self) -> i32 {
        return self.offset;
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
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

    pub fn toggle_sck_bit(&mut self, hx711_delay_time_us: u32) {
        self.sck_pin.set_high().unwrap();
        self.delay.delay_us(hx711_delay_time_us);
        self.sck_pin.set_low().unwrap();
        self.delay.delay_us(hx711_delay_time_us);
    }
}
