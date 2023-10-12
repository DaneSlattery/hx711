#![no_std]

pub mod hx711;

// pub mod hx711_interrupt;
pub trait LoadCell {
    type Offset;
    type Scale;
    /// Read the value from the load cell
    fn read(&mut self) -> i32;

    /// Read the value after applying scaling.
    /// Casts to the type of Scale.
    fn read_scaled(&mut self) -> Self::Scale;

    /// Zero the load cell offset by averaging `num_samples` readings
    fn tare(&mut self, num_samples: usize);

    /// Get the load cell offset.
    fn get_offset(&self) -> Self::Offset;

    /// Set the scale (AKA calibrate the scale).
    /// Use this to ensure that 1kg ~ 1kg
    fn set_scale(&mut self, scale: Self::Scale);

    /// Get the scale.
    fn get_scale(&self) -> Self::Scale;
}

// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
