#![no_std]

pub mod hx711;

/// The generic load cell interface.
pub trait LoadCell {
    /// The offset used to zero the load cell.
    type Offset;
    /// The multiplier used for the scale sensitivity.
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

    /// Get the  scale.
    fn get_scale(&self) -> Self::Scale;
}
