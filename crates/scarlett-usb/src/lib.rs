//! Scarlett USB Communication Layer
//!
//! Direct USB communication with Focusrite Scarlett devices.
//! Supports multiple transport types (direct USB, USB/IP).

pub mod detection;
pub mod protocol;
pub mod device_impl;
pub mod gen3_protocol;
pub mod gen4_fcp;
pub mod transport;
pub mod direct_usb_transport;

pub use detection::{DeviceDetector, HotplugEvent};
pub use device_impl::UsbDevice;
pub use transport::{UsbTransport, TransportType, ControlTransfer, Direction};
pub use direct_usb_transport::DirectUsbTransport;

use scarlett_core::{Error, Result};

/// Initialize USB subsystem
pub fn init() -> Result<()> {
    tracing::info!("Initializing USB subsystem");
    Ok(())
}
