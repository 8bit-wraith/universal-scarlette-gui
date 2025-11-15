//! Scarlett USB Communication Layer
//!
//! Direct USB communication with Focusrite Scarlett devices.

pub mod detection;
pub mod protocol;
pub mod device_impl;
pub mod gen3_protocol;
pub mod gen4_fcp;

pub use detection::{DeviceDetector, HotplugEvent};
pub use device_impl::UsbDevice;

use scarlett_core::{Error, Result};

/// Initialize USB subsystem
pub fn init() -> Result<()> {
    tracing::info!("Initializing USB subsystem");
    Ok(())
}
