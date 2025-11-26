//! Scarlett Core Library
//!
//! Core types, traits, and protocols for Focusrite Scarlett USB audio interfaces.

pub mod device;
pub mod protocol;
pub mod routing;
pub mod mixer;
pub mod error;

pub use device::{Device, DeviceInfo, DeviceGeneration, DeviceModel};
pub use error::{Error, Result};

/// Focusrite USB Vendor ID
pub const FOCUSRITE_VENDOR_ID: u16 = 0x1235;

/// Maximum number of channels supported
pub const MAX_CHANNELS: usize = 92;
