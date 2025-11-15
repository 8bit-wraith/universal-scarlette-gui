//! USB Transport Abstraction Layer
//!
//! This module provides a transport-agnostic interface for USB communication,
//! allowing for multiple backends:
//! - Direct local USB (via nusb)
//! - USB/IP network transport (future)
//! - Mock transport for testing

use scarlett_core::{Error, Result};
use std::time::Duration;

/// USB Control Transfer Direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Host to Device
    Out,
    /// Device to Host
    In,
}

/// USB Control Transfer Request
#[derive(Debug, Clone)]
pub struct ControlTransfer {
    /// Request type (vendor, class, standard)
    pub request_type: u8,
    /// Specific request
    pub request: u8,
    /// Request value
    pub value: u16,
    /// Request index (often interface number)
    pub index: u16,
    /// Direction (In or Out)
    pub direction: Direction,
    /// Timeout for the transfer
    pub timeout: Duration,
}

impl ControlTransfer {
    /// Create a new control transfer request
    pub fn new(
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        direction: Direction,
    ) -> Self {
        Self {
            request_type,
            request,
            value,
            index,
            direction,
            timeout: Duration::from_secs(1),
        }
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Create vendor-specific OUT transfer
    pub fn vendor_out(request: u8, value: u16, index: u16) -> Self {
        Self::new(0x40, request, value, index, Direction::Out)
    }

    /// Create vendor-specific IN transfer
    pub fn vendor_in(request: u8, value: u16, index: u16) -> Self {
        Self::new(0xC0, request, value, index, Direction::In)
    }

    /// Create class-specific OUT transfer
    pub fn class_out(request: u8, value: u16, index: u16) -> Self {
        Self::new(0x21, request, value, index, Direction::Out)
    }

    /// Create class-specific IN transfer
    pub fn class_in(request: u8, value: u16, index: u16) -> Self {
        Self::new(0xA1, request, value, index, Direction::In)
    }
}

/// USB Bulk Transfer Request
#[derive(Debug, Clone)]
pub struct BulkTransfer {
    /// Endpoint address
    pub endpoint: u8,
    /// Direction
    pub direction: Direction,
    /// Timeout
    pub timeout: Duration,
}

/// USB Transport trait - abstraction over different transport methods
pub trait UsbTransport: Send + Sync {
    /// Perform a control transfer OUT (host to device)
    fn control_out(&self, transfer: &ControlTransfer, data: &[u8]) -> Result<usize>;

    /// Perform a control transfer IN (device to host)
    fn control_in(&self, transfer: &ControlTransfer, buffer: &mut [u8]) -> Result<usize>;

    /// Perform a bulk transfer OUT
    fn bulk_out(&self, transfer: &BulkTransfer, data: &[u8]) -> Result<usize>;

    /// Perform a bulk transfer IN
    fn bulk_in(&self, transfer: &BulkTransfer, buffer: &mut [u8]) -> Result<usize>;

    /// Check if transport is connected
    fn is_connected(&self) -> bool;

    /// Get transport type name (for debugging/display)
    fn transport_name(&self) -> &'static str;
}

/// Transport type selector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    /// Direct local USB via nusb
    DirectUsb,
    /// USB/IP network transport
    #[allow(dead_code)]
    UsbIp,
    /// Mock transport for testing
    #[allow(dead_code)]
    Mock,
}

impl TransportType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DirectUsb => "Direct USB",
            Self::UsbIp => "USB/IP",
            Self::Mock => "Mock",
        }
    }
}

/// Helper functions for common transfer patterns
pub mod helpers {
    use super::*;

    /// Perform a simple vendor OUT transfer with data
    pub fn vendor_write(
        transport: &dyn UsbTransport,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
    ) -> Result<()> {
        let transfer = ControlTransfer::vendor_out(request, value, index);
        transport.control_out(&transfer, data)?;
        Ok(())
    }

    /// Perform a simple vendor IN transfer
    pub fn vendor_read(
        transport: &dyn UsbTransport,
        request: u8,
        value: u16,
        index: u16,
        length: usize,
    ) -> Result<Vec<u8>> {
        let transfer = ControlTransfer::vendor_in(request, value, index);
        let mut buffer = vec![0u8; length];
        let actual = transport.control_in(&transfer, &mut buffer)?;
        buffer.truncate(actual);
        Ok(buffer)
    }

    /// Perform a class OUT transfer
    pub fn class_write(
        transport: &dyn UsbTransport,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
    ) -> Result<()> {
        let transfer = ControlTransfer::class_out(request, value, index);
        transport.control_out(&transfer, data)?;
        Ok(())
    }

    /// Perform a class IN transfer
    pub fn class_read(
        transport: &dyn UsbTransport,
        request: u8,
        value: u16,
        index: u16,
        length: usize,
    ) -> Result<Vec<u8>> {
        let transfer = ControlTransfer::class_in(request, value, index);
        let mut buffer = vec![0u8; length];
        let actual = transport.control_in(&transfer, &mut buffer)?;
        buffer.truncate(actual);
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock transport for testing
    struct MockTransport {
        connected: bool,
    }

    impl UsbTransport for MockTransport {
        fn control_out(&self, _transfer: &ControlTransfer, data: &[u8]) -> Result<usize> {
            Ok(data.len())
        }

        fn control_in(&self, _transfer: &ControlTransfer, buffer: &mut [u8]) -> Result<usize> {
            // Fill with test data
            for (i, byte) in buffer.iter_mut().enumerate() {
                *byte = (i % 256) as u8;
            }
            Ok(buffer.len())
        }

        fn bulk_out(&self, _transfer: &BulkTransfer, data: &[u8]) -> Result<usize> {
            Ok(data.len())
        }

        fn bulk_in(&self, _transfer: &BulkTransfer, buffer: &mut [u8]) -> Result<usize> {
            Ok(buffer.len())
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        fn transport_name(&self) -> &'static str {
            "Mock"
        }
    }

    #[test]
    fn test_control_transfer_builder() {
        let transfer = ControlTransfer::vendor_out(0x01, 0x1234, 0x5678);
        assert_eq!(transfer.request_type, 0x40);
        assert_eq!(transfer.request, 0x01);
        assert_eq!(transfer.value, 0x1234);
        assert_eq!(transfer.index, 0x5678);
        assert_eq!(transfer.direction, Direction::Out);
    }

    #[test]
    fn test_mock_transport() {
        let transport = MockTransport { connected: true };

        // Test control OUT
        let transfer = ControlTransfer::vendor_out(0x01, 0x00, 0x00);
        let result = transport.control_out(&transfer, &[1, 2, 3, 4]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);

        // Test control IN
        let transfer = ControlTransfer::vendor_in(0x02, 0x00, 0x00);
        let mut buffer = vec![0u8; 16];
        let result = transport.control_in(&transfer, &mut buffer);
        assert!(result.is_ok());

        // Test connection
        assert!(transport.is_connected());
        assert_eq!(transport.transport_name(), "Mock");
    }

    #[test]
    fn test_helpers() {
        let transport = MockTransport { connected: true };

        // Test vendor write
        let result = helpers::vendor_write(&transport, 0x01, 0x00, 0x00, &[1, 2, 3]);
        assert!(result.is_ok());

        // Test vendor read
        let result = helpers::vendor_read(&transport, 0x02, 0x00, 0x00, 10);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 10);
    }
}
