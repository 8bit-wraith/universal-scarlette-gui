//! Direct USB Transport via nusb
//!
//! Local USB device communication using the nusb library.

use crate::transport::{BulkTransfer, ControlTransfer, UsbTransport};
use scarlett_core::{Error, Result};
use nusb::Device;
use std::sync::Arc;
use tracing::{debug, trace, warn};

/// Direct USB transport implementation using nusb
pub struct DirectUsbTransport {
    device: Arc<Device>,
    interface_number: u8,
}

impl DirectUsbTransport {
    /// Create a new direct USB transport
    pub fn new(device: Device, interface_number: u8) -> Self {
        Self {
            device: Arc::new(device),
            interface_number,
        }
    }

}

impl UsbTransport for DirectUsbTransport {
    fn control_out(&self, transfer: &ControlTransfer, data: &[u8]) -> Result<usize> {
        trace!(
            "USB control OUT: type=0x{:02x}, req=0x{:02x}, val=0x{:04x}, idx=0x{:04x}, len={}",
            transfer.request_type,
            transfer.request,
            transfer.value,
            transfer.index,
            data.len()
        );

        // TODO: Implement actual nusb 0.1.x control transfer
        // The nusb 0.1.x API is different from what I initially implemented
        // Need to study the nusb documentation and examples

        warn!("USB control OUT not yet implemented - returning success stub");
        debug!("Would send {} bytes", data.len());
        Ok(data.len())
    }

    fn control_in(&self, transfer: &ControlTransfer, buffer: &mut [u8]) -> Result<usize> {
        trace!(
            "USB control IN: type=0x{:02x}, req=0x{:02x}, val=0x{:04x}, idx=0x{:04x}, len={}",
            transfer.request_type,
            transfer.request,
            transfer.value,
            transfer.index,
            buffer.len()
        );

        // TODO: Implement actual nusb 0.1.x control transfer
        // The nusb 0.1.x API is different from what I initially implemented
        // Need to study the nusb documentation and examples

        warn!("USB control IN not yet implemented - returning empty stub");
        Ok(0)
    }

    fn bulk_out(&self, transfer: &BulkTransfer, data: &[u8]) -> Result<usize> {
        trace!(
            "USB bulk OUT: ep=0x{:02x}, len={}",
            transfer.endpoint,
            data.len()
        );

        // TODO: Implement actual nusb 0.1.x bulk transfer

        warn!("USB bulk OUT not yet implemented - returning success stub");
        Ok(data.len())
    }

    fn bulk_in(&self, transfer: &BulkTransfer, buffer: &mut [u8]) -> Result<usize> {
        trace!(
            "USB bulk IN: ep=0x{:02x}, len={}",
            transfer.endpoint,
            buffer.len()
        );

        // TODO: Implement actual nusb 0.1.x bulk transfer

        warn!("USB bulk IN not yet implemented - returning empty stub");
        Ok(0)
    }

    fn is_connected(&self) -> bool {
        // TODO: Properly check if device is still connected
        // For now, assume it's connected
        true
    }

    fn transport_name(&self) -> &'static str {
        "Direct USB"
    }
}

/// Builder for DirectUsbTransport
pub struct DirectUsbTransportBuilder {
    interface_number: u8,
}

impl DirectUsbTransportBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            interface_number: 0,
        }
    }

    /// Set the interface number to use
    pub fn interface(mut self, number: u8) -> Self {
        self.interface_number = number;
        self
    }

    /// Build the transport with a device
    pub fn build(self, device: Device) -> Result<DirectUsbTransport> {
        debug!(
            "Creating DirectUsbTransport for interface {}",
            self.interface_number
        );

        Ok(DirectUsbTransport::new(device, self.interface_number))
    }
}

impl Default for DirectUsbTransportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let builder = DirectUsbTransportBuilder::new().interface(1);
        assert_eq!(builder.interface_number, 1);
    }
}
