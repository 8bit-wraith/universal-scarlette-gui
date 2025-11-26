//! Scarlett Gen 2/3 USB Protocol
//!
//! Gen 2 and Gen 3 devices use the "Scarlett2" USB protocol which communicates
//! via USB vendor-specific control transfers

use scarlett_core::{Error, Result};
use nusb::{Device, transfer::RequestBuffer};
use std::time::Duration;

/// USB Control transfer parameters for Scarlett2 protocol
pub const USB_REQUEST_TYPE_CLASS: u8 = 0x21;  // Class-specific, Host-to-Device
pub const USB_REQUEST_TYPE_VENDOR_IN: u8 = 0xC0;  // Vendor-specific, Device-to-Host
pub const USB_REQUEST_TYPE_VENDOR_OUT: u8 = 0x40;  // Vendor-specific, Host-to-Device

/// USB Interface for audio control
pub const USB_AUDIO_CONTROL_INTERFACE: u8 = 0;

/// Scarlett2 USB Request codes
pub const SCARLETT2_USB_CMD_INIT: u8 = 0x00;
pub const SCARLETT2_USB_CMD_REQ: u8 = 0x02;
pub const SCARLETT2_USB_CMD_RESP: u8 = 0x03;

/// Scarlett2 Protocol Commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Scarlett2Command {
    /// Get meter levels
    GetMeterLevels = 0x1001,
    /// Get configuration
    GetConfig = 0x1002,
    /// Set configuration
    SetConfig = 0x1003,
    /// Get mixer values
    GetMixer = 0x3001,
    /// Set mixer values
    SetMixer = 0x3002,
    /// Get routing
    GetRouting = 0x3101,
    /// Set routing
    SetRouting = 0x3102,
}

/// Scarlett2 USB Protocol Handler
pub struct Scarlett2Protocol {
    device: Device,
    sequence: u8,
}

impl Scarlett2Protocol {
    /// Create a new protocol handler
    pub fn new(device: Device) -> Self {
        Self {
            device,
            sequence: 0,
        }
    }

    /// Initialize the device
    pub fn init(&mut self) -> Result<()> {
        tracing::debug!("Initializing Scarlett2 protocol");

        // Send INIT command
        let data = vec![SCARLETT2_USB_CMD_INIT, 0x00];

        self.control_write(0x00, 0x00, &data)?;

        Ok(())
    }

    /// Send a command and receive response
    pub fn send_command(&mut self, cmd: Scarlett2Command, data: &[u8]) -> Result<Vec<u8>> {
        tracing::debug!("Sending Scarlett2 command: {:?}", cmd);

        self.sequence = self.sequence.wrapping_add(1);

        // Build request packet
        let mut request = Vec::new();
        request.push(SCARLETT2_USB_CMD_REQ);
        request.push(self.sequence);
        request.extend_from_slice(&(cmd as u16).to_le_bytes());
        request.extend_from_slice(&(data.len() as u16).to_le_bytes());
        request.extend_from_slice(data);

        // Send request
        self.control_write(0x00, 0x00, &request)?;

        // Receive response
        let response = self.control_read(0x00, 0x00, 1024)?;

        // Validate response
        if response.len() < 4 {
            return Err(Error::Protocol("Response too short".to_string()));
        }

        if response[0] != SCARLETT2_USB_CMD_RESP {
            return Err(Error::Protocol(format!(
                "Invalid response command: 0x{:02x}",
                response[0]
            )));
        }

        if response[1] != self.sequence {
            return Err(Error::Protocol(format!(
                "Sequence mismatch: expected {}, got {}",
                self.sequence, response[1]
            )));
        }

        // Extract payload (skip command byte, sequence, and length)
        let payload_len = u16::from_le_bytes([response[2], response[3]]) as usize;

        if response.len() < 4 + payload_len {
            return Err(Error::Protocol("Response payload truncated".to_string()));
        }

        Ok(response[4..4 + payload_len].to_vec())
    }

    /// Get meter levels
    pub fn get_meter_levels(&mut self) -> Result<Vec<i32>> {
        let response = self.send_command(Scarlett2Command::GetMeterLevels, &[])?;

        // Parse meter levels (each is a 32-bit signed integer)
        let mut levels = Vec::new();
        for chunk in response.chunks_exact(4) {
            let level = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            levels.push(level);
        }

        Ok(levels)
    }

    /// Get mixer volume for a specific input
    pub fn get_mixer_volume(&mut self, input_index: u16) -> Result<u16> {
        let data = input_index.to_le_bytes();
        let response = self.send_command(Scarlett2Command::GetMixer, &data)?;

        if response.len() < 2 {
            return Err(Error::Protocol("Mixer response too short".to_string()));
        }

        Ok(u16::from_le_bytes([response[0], response[1]]))
    }

    /// Set mixer volume for a specific input
    pub fn set_mixer_volume(&mut self, input_index: u16, volume: u16) -> Result<()> {
        let mut data = Vec::new();
        data.extend_from_slice(&input_index.to_le_bytes());
        data.extend_from_slice(&volume.to_le_bytes());

        self.send_command(Scarlett2Command::SetMixer, &data)?;

        Ok(())
    }

    /// Low-level USB control write
    fn control_write(&self, value: u16, index: u16, data: &[u8]) -> Result<()> {
        tracing::trace!(
            "USB control write: value=0x{:04x}, index=0x{:04x}, len={}",
            value, index, data.len()
        );

        // TODO: Implement actual USB control transfer using nusb
        // For now, this is a placeholder

        // let result = self.device.control_out(
        //     USB_REQUEST_TYPE_VENDOR_OUT,
        //     0x00,  // request
        //     value,
        //     index,
        //     data,
        //     Duration::from_millis(1000),
        // )?;

        Ok(())
    }

    /// Low-level USB control read
    fn control_read(&self, value: u16, index: u16, length: usize) -> Result<Vec<u8>> {
        tracing::trace!(
            "USB control read: value=0x{:04x}, index=0x{:04x}, len={}",
            value, index, length
        );

        // TODO: Implement actual USB control transfer using nusb
        // For now, return empty vec as placeholder

        // let mut buffer = vec![0u8; length];
        // let result = self.device.control_in(
        //     USB_REQUEST_TYPE_VENDOR_IN,
        //     0x00,  // request
        //     value,
        //     index,
        //     &mut buffer,
        //     Duration::from_millis(1000),
        // )?;

        // Ok(buffer[..result].to_vec())

        Ok(Vec::new())
    }
}

/// Convert raw meter level to dB
pub fn meter_level_to_db(level: i32) -> f32 {
    if level <= 0 {
        -127.0
    } else {
        // Scarlett meters are in 8.24 fixed point format
        // Convert to dB (20 * log10(level / 2^24))
        20.0 * ((level as f64) / 16777216.0).log10() as f32
    }
}

/// Convert dB to mixer volume value (0-65535)
pub fn db_to_mixer_volume(db: f32) -> u16 {
    if db <= -127.0 {
        0
    } else {
        let linear = 10.0_f32.powf(db / 20.0);
        (linear * 65535.0).min(65535.0) as u16
    }
}

/// Convert mixer volume value to dB
pub fn mixer_volume_to_db(volume: u16) -> f32 {
    if volume == 0 {
        -127.0
    } else {
        20.0 * ((volume as f32) / 65535.0).log10()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_conversions() {
        // 0 dB should be around max volume
        let vol = db_to_mixer_volume(0.0);
        assert!(vol > 60000);

        // -6 dB should be about half
        let vol = db_to_mixer_volume(-6.0);
        assert!(vol > 30000 && vol < 35000);

        // Very negative dB should be 0
        let vol = db_to_mixer_volume(-130.0);
        assert_eq!(vol, 0);
    }

    #[test]
    fn test_volume_roundtrip() {
        let original_db = -12.0;
        let volume = db_to_mixer_volume(original_db);
        let converted_db = mixer_volume_to_db(volume);

        // Should be within 0.5 dB
        assert!((converted_db - original_db).abs() < 0.5);
    }
}
