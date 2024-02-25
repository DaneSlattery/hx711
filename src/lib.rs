#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub mod hx711;

/// The generic load cell interface.
pub trait LoadCell {
    /// The offset used to zero the load cell.
    type Offset;
    /// The multiplier used for the scale sensitivity.
    type Scale;

    /// Returned when trying to read from the hx711 chip when it is not ready.
    type NotReadyError;

    /// Read the value from the load cell
    fn read(&mut self) -> Result<i32, Self::NotReadyError>;

    /// Read the value after applying scaling.
    /// Casts to the type of Scale.
    fn read_scaled(&mut self) -> Result<Self::Scale, Self::NotReadyError>;

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
