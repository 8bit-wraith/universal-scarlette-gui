//! Scarlett Firmware File Format
//!
//! Handles parsing and validation of Scarlett firmware update files.
//! Based on scarlett2-firmware.c from the Linux tools.

use scarlett_core::{Error, Result};
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Magic string at the start of all Scarlett firmware files
pub const FIRMWARE_MAGIC: &[u8; 8] = b"SCARLETT";

/// Scarlett firmware file header (52 bytes, packed)
/// All multi-byte integers are stored in BIG-ENDIAN format
#[derive(Debug, Clone)]
pub struct FirmwareHeader {
    /// Magic string "SCARLETT"
    pub magic: [u8; 8],
    /// USB Vendor ID (0x1235 for Focusrite) - Big-endian
    pub usb_vid: u16,
    /// USB Product ID (device-specific) - Big-endian
    pub usb_pid: u16,
    /// Firmware version number - Big-endian
    pub firmware_version: u32,
    /// Length of firmware data in bytes - Big-endian
    pub firmware_length: u32,
    /// SHA-256 hash of firmware data
    pub sha256: [u8; 32],
}

impl FirmwareHeader {
    /// Size of the header in bytes
    pub const SIZE: usize = 52;

    /// Parse header from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(Error::Protocol(format!(
                "Firmware header too short: {} bytes (expected {})",
                bytes.len(),
                Self::SIZE
            )));
        }

        // Extract magic string
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);

        // Validate magic
        if &magic != FIRMWARE_MAGIC {
            return Err(Error::Protocol(format!(
                "Invalid firmware magic: expected 'SCARLETT', got '{}'",
                String::from_utf8_lossy(&magic)
            )));
        }

        // Parse fields (all big-endian)
        let usb_vid = u16::from_be_bytes([bytes[8], bytes[9]]);
        let usb_pid = u16::from_be_bytes([bytes[10], bytes[11]]);
        let firmware_version = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let firmware_length = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);

        // Extract SHA-256 hash
        let mut sha256 = [0u8; 32];
        sha256.copy_from_slice(&bytes[20..52]);

        Ok(Self {
            magic,
            usb_vid,
            usb_pid,
            firmware_version,
            firmware_length,
            sha256,
        })
    }

    /// Read header from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path.as_ref())
            .map_err(|e| Error::Io(e))?;

        let mut header_bytes = [0u8; Self::SIZE];
        file.read_exact(&mut header_bytes)
            .map_err(|e| Error::Io(e))?;

        Self::from_bytes(&header_bytes)
    }
}

/// Complete firmware file with header and data
#[derive(Debug, Clone)]
pub struct FirmwareFile {
    /// Firmware header
    pub header: FirmwareHeader,
    /// Firmware binary data
    pub data: Vec<u8>,
}

impl FirmwareFile {
    /// Read and validate complete firmware file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();

        tracing::info!("Reading firmware file: {}", path_ref.display());

        // Read header
        let header = FirmwareHeader::from_file(path_ref)?;

        tracing::debug!(
            "Firmware header: VID=0x{:04x}, PID=0x{:04x}, version={}, length={} bytes",
            header.usb_vid,
            header.usb_pid,
            header.firmware_version,
            header.firmware_length
        );

        // Read entire file
        let mut file = File::open(path_ref)
            .map_err(|e| Error::Io(e))?;

        let mut all_data = Vec::new();
        file.read_to_end(&mut all_data)
            .map_err(|e| Error::Io(e))?;

        // Validate file size
        let expected_size = FirmwareHeader::SIZE + header.firmware_length as usize;
        if all_data.len() != expected_size {
            return Err(Error::Protocol(format!(
                "Firmware file size mismatch: got {} bytes, expected {} (header) + {} (data) = {}",
                all_data.len(),
                FirmwareHeader::SIZE,
                header.firmware_length,
                expected_size
            )));
        }

        // Extract firmware data (everything after header)
        let data = all_data[FirmwareHeader::SIZE..].to_vec();

        // Verify SHA-256 hash
        tracing::debug!("Verifying firmware SHA-256 hash...");
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let computed_hash = hasher.finalize();

        if computed_hash.as_slice() != &header.sha256 {
            return Err(Error::Protocol(
                "Firmware SHA-256 hash mismatch! File may be corrupted.".to_string()
            ));
        }

        tracing::info!("Firmware file validated successfully");

        Ok(Self { header, data })
    }

    /// Validate that firmware is compatible with a specific device
    pub fn validate_for_device(&self, vid: u16, pid: u16) -> Result<()> {
        if self.header.usb_vid != vid {
            return Err(Error::Protocol(format!(
                "Firmware VID mismatch: firmware is for 0x{:04x}, device is 0x{:04x}",
                self.header.usb_vid, vid
            )));
        }

        if self.header.usb_pid != pid {
            return Err(Error::Protocol(format!(
                "Firmware PID mismatch: firmware is for 0x{:04x}, device is 0x{:04x}",
                self.header.usb_pid, pid
            )));
        }

        Ok(())
    }

    /// Get firmware version
    pub fn version(&self) -> u32 {
        self.header.firmware_version
    }

    /// Get firmware data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get firmware length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if firmware is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_size() {
        // 8 (magic) + 2 (vid) + 2 (pid) + 4 (version) + 4 (length) + 32 (sha256) = 52
        assert_eq!(FirmwareHeader::SIZE, 52);
    }

    #[test]
    fn test_invalid_magic() {
        let mut bytes = [0u8; FirmwareHeader::SIZE];
        bytes[0..8].copy_from_slice(b"NOTMAGIC");

        let result = FirmwareHeader::from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_header() {
        let mut bytes = [0u8; FirmwareHeader::SIZE];

        // Set magic
        bytes[0..8].copy_from_slice(FIRMWARE_MAGIC);

        // Set VID (0x1235 big-endian)
        bytes[8] = 0x12;
        bytes[9] = 0x35;

        // Set PID (0x821D big-endian)
        bytes[10] = 0x82;
        bytes[11] = 0x1D;

        let header = FirmwareHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header.usb_vid, 0x1235);
        assert_eq!(header.usb_pid, 0x821D);
    }
}
