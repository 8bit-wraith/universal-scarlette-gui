//! Direct USB Transport via nusb
//!
//! Local USB device communication using the nusb library.

use crate::transport::{BulkTransfer, ControlTransfer, UsbTransport};
use scarlett_core::{Error, Result};
use nusb::{Device, Interface};
use std::sync::Arc;
use tracing::{debug, trace};

/// Direct USB transport implementation using nusb
pub struct DirectUsbTransport {
    device: Arc<Device>,
    interface: Interface,
    interface_number: u8,
}

impl DirectUsbTransport {
    /// Create a new direct USB transport
    pub fn new(device: Device, interface_number: u8) -> Result<Self> {
        debug!("Claiming USB interface {}", interface_number);

        // Claim the interface for exclusive access
        let interface = device
            .claim_interface(interface_number)
            .map_err(|e| Error::Usb(format!("Failed to claim interface: {:?}", e)))?;

        Ok(Self {
            device: Arc::new(device),
            interface,
            interface_number,
        })
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

        // Parse request_type to determine control transfer parameters
        let control_type = match (transfer.request_type >> 5) & 0x03 {
            0 => nusb::transfer::ControlType::Standard,
            1 => nusb::transfer::ControlType::Class,
            2 => nusb::transfer::ControlType::Vendor,
            _ => return Err(Error::Usb("Invalid control type".to_string())),
        };

        let recipient = match transfer.request_type & 0x1F {
            0 => nusb::transfer::Recipient::Device,
            1 => nusb::transfer::Recipient::Interface,
            2 => nusb::transfer::Recipient::Endpoint,
            3 => nusb::transfer::Recipient::Other,
            _ => return Err(Error::Usb("Invalid recipient".to_string())),
        };

        // Perform the control transfer
        let future = self.interface.control_out(nusb::transfer::ControlOut {
            control_type,
            recipient,
            request: transfer.request,
            value: transfer.value,
            index: transfer.index,
            data,
        });

        // Block on the async operation
        let completion = futures::executor::block_on(future);

        // Check status
        completion.status
            .map_err(|e| Error::Usb(format!("Control OUT failed: {:?}", e)))?;

        trace!("Control OUT completed: {} bytes transferred", data.len());
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

        // Parse request_type to determine control transfer parameters
        let control_type = match (transfer.request_type >> 5) & 0x03 {
            0 => nusb::transfer::ControlType::Standard,
            1 => nusb::transfer::ControlType::Class,
            2 => nusb::transfer::ControlType::Vendor,
            _ => return Err(Error::Usb("Invalid control type".to_string())),
        };

        let recipient = match transfer.request_type & 0x1F {
            0 => nusb::transfer::Recipient::Device,
            1 => nusb::transfer::Recipient::Interface,
            2 => nusb::transfer::Recipient::Endpoint,
            3 => nusb::transfer::Recipient::Other,
            _ => return Err(Error::Usb("Invalid recipient".to_string())),
        };

        // Perform the control transfer
        let future = self.interface.control_in(nusb::transfer::ControlIn {
            control_type,
            recipient,
            request: transfer.request,
            value: transfer.value,
            index: transfer.index,
            length: buffer.len() as u16,
        });

        // Block on the async operation
        let completion = futures::executor::block_on(future);

        // Check status
        completion.status
            .map_err(|e| Error::Usb(format!("Control IN failed: {:?}", e)))?;

        // Copy data to buffer
        let actual_len = completion.data.len().min(buffer.len());
        buffer[..actual_len].copy_from_slice(&completion.data[..actual_len]);

        trace!("Control IN completed: {} bytes received", actual_len);
        Ok(actual_len)
    }

    fn bulk_out(&self, _transfer: &BulkTransfer, _data: &[u8]) -> Result<usize> {
        // TODO: Implement bulk transfers if needed
        // Most Scarlett devices use control transfers for communication
        // This is here for completeness and future expansion
        trace!("Bulk OUT not yet implemented");
        Err(Error::NotSupported("Bulk transfers not yet implemented".to_string()))
    }

    fn bulk_in(&self, _transfer: &BulkTransfer, _buffer: &mut [u8]) -> Result<usize> {
        // TODO: Implement bulk transfers if needed
        // Most Scarlett devices use control transfers for communication
        // This is here for completeness and future expansion
        trace!("Bulk IN not yet implemented");
        Err(Error::NotSupported("Bulk transfers not yet implemented".to_string()))
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

        DirectUsbTransport::new(device, self.interface_number)
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
