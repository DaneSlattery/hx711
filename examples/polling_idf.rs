// currently does not build in in this no-std environment. copy the code elsewhere
use esp_idf_svc::hal::{delay, gpio::PinDriver, peripherals::Peripherals};
use loadcell::{hx711::HX711, LoadCell};

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let dt = PinDriver::input(peripherals.pins.gpio1).unwrap();
    let sck = PinDriver::output(peripherals.pins.gpio10).unwrap();
    let mut load_sensor = HX711::new(sck, dt, delay::FreeRtos);

    load_sensor.tare(16);
    load_sensor.set_scale(1.0);

    loop {
        if load_sensor.is_ready() {
            let reading = load_sensor.read_scaled();
            log::info!("Last Reading = {:?}", reading);
        }

        delay::FreeRtos::delay_ms(1000u32);
    }
}
