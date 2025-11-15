//! Focusrite Control Protocol (FCP) for Gen 4 devices
//!
//! Gen 4 "big" devices (16i16, 18i16, 18i20) use the FCP protocol
//! for configuration and control.

use scarlett_core::{Error, Result};
use std::fmt;

/// FCP Protocol Version
pub const FCP_PROTOCOL_VERSION: u8 = 1;

/// FCP Magic bytes
pub const FCP_MAGIC_REQUEST: u8 = 0x53;
pub const FCP_MAGIC_RESPONSE: u8 = 0x73;

/// Maximum payload length (2MB)
pub const MAX_PAYLOAD_LENGTH: usize = 2 * 1024 * 1024;

/// FCP Error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i16)]
pub enum FcpErrorCode {
    InvalidMagic = 1,
    InvalidCommand = 2,
    InvalidLength = 3,
    InvalidHash = 4,
    InvalidUsbId = 5,
    Config = 6,
    Fcp = 7,
    Timeout = 8,
    Read = 9,
    Write = 10,
    NotLeapfrog = 11,
    InvalidState = 12,
}

impl FcpErrorCode {
    pub fn from_i16(val: i16) -> Option<Self> {
        match val {
            1 => Some(Self::InvalidMagic),
            2 => Some(Self::InvalidCommand),
            3 => Some(Self::InvalidLength),
            4 => Some(Self::InvalidHash),
            5 => Some(Self::InvalidUsbId),
            6 => Some(Self::Config),
            7 => Some(Self::Fcp),
            8 => Some(Self::Timeout),
            9 => Some(Self::Read),
            10 => Some(Self::Write),
            11 => Some(Self::NotLeapfrog),
            12 => Some(Self::InvalidState),
            _ => None,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::InvalidMagic => "Invalid magic byte",
            Self::InvalidCommand => "Invalid command",
            Self::InvalidLength => "Invalid length",
            Self::InvalidHash => "Invalid hash",
            Self::InvalidUsbId => "Invalid USB ID",
            Self::Config => "Configuration error",
            Self::Fcp => "FCP error",
            Self::Timeout => "Timeout",
            Self::Read => "Read error",
            Self::Write => "Write error",
            Self::NotLeapfrog => "Not leapfrog",
            Self::InvalidState => "Invalid state",
        }
    }
}

impl fmt::Display for FcpErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

/// FCP Request types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum FcpRequestType {
    Reboot = 0x0001,
    ConfigErase = 0x0002,
    AppFirmwareErase = 0x0003,
    AppFirmwareUpdate = 0x0004,
    EspFirmwareUpdate = 0x0005,
}

/// FCP Response types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FcpResponseType {
    Version = 0x00,
    Success = 0x01,
    Error = 0x02,
    Progress = 0x03,
}

impl FcpResponseType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(Self::Version),
            0x01 => Some(Self::Success),
            0x02 => Some(Self::Error),
            0x03 => Some(Self::Progress),
            _ => None,
        }
    }
}

/// FCP Message Header (6 bytes, packed)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct FcpMessageHeader {
    pub magic: u8,
    pub msg_type: u8,
    pub payload_length: u32,  // Little-endian
}

impl FcpMessageHeader {
    pub fn new_request(msg_type: u8, payload_length: u32) -> Self {
        Self {
            magic: FCP_MAGIC_REQUEST,
            msg_type,
            payload_length,
        }
    }

    pub fn new_response(msg_type: u8, payload_length: u32) -> Self {
        Self {
            magic: FCP_MAGIC_RESPONSE,
            msg_type,
            payload_length,
        }
    }

    pub fn to_bytes(&self) -> [u8; 6] {
        let mut bytes = [0u8; 6];
        bytes[0] = self.magic;
        bytes[1] = self.msg_type;
        // Copy payload_length manually to avoid packed field reference
        let payload_len = self.payload_length;
        bytes[2..6].copy_from_slice(&payload_len.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 6 {
            return Err(Error::Protocol("Header too short".to_string()));
        }

        let magic = bytes[0];
        let msg_type = bytes[1];
        let payload_length = u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);

        Ok(Self {
            magic,
            msg_type,
            payload_length,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.magic != FCP_MAGIC_REQUEST && self.magic != FCP_MAGIC_RESPONSE {
            return Err(Error::Protocol(format!(
                "Invalid magic byte: 0x{:02x}",
                self.magic
            )));
        }

        let payload_len = self.payload_length;
        if payload_len as usize > MAX_PAYLOAD_LENGTH {
            return Err(Error::Protocol(format!(
                "Payload too large: {} bytes",
                payload_len
            )));
        }

        Ok(())
    }
}

/// FCP Version Message
#[derive(Debug, Clone)]
pub struct FcpVersionMessage {
    pub header: FcpMessageHeader,
    pub version: u8,
}

impl FcpVersionMessage {
    pub fn new(version: u8) -> Self {
        Self {
            header: FcpMessageHeader::new_response(FcpResponseType::Version as u8, 1),
            version,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.header.to_bytes().to_vec();
        bytes.push(self.version);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 7 {
            return Err(Error::Protocol("Version message too short".to_string()));
        }

        let header = FcpMessageHeader::from_bytes(&bytes[0..6])?;
        header.validate()?;

        let version = bytes[6];

        Ok(Self { header, version })
    }
}

/// FCP Progress Message
#[derive(Debug, Clone)]
pub struct FcpProgressMessage {
    pub header: FcpMessageHeader,
    pub percent: u8,
}

impl FcpProgressMessage {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 7 {
            return Err(Error::Protocol("Progress message too short".to_string()));
        }

        let header = FcpMessageHeader::from_bytes(&bytes[0..6])?;
        header.validate()?;

        let percent = bytes[6];

        Ok(Self { header, percent })
    }
}

/// FCP Error Message
#[derive(Debug, Clone)]
pub struct FcpErrorMessage {
    pub header: FcpMessageHeader,
    pub error_code: i16,
}

impl FcpErrorMessage {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            return Err(Error::Protocol("Error message too short".to_string()));
        }

        let header = FcpMessageHeader::from_bytes(&bytes[0..6])?;
        header.validate()?;

        let error_code = i16::from_le_bytes([bytes[6], bytes[7]]);

        Ok(Self { header, error_code })
    }

    pub fn error_code_enum(&self) -> Option<FcpErrorCode> {
        FcpErrorCode::from_i16(self.error_code)
    }
}

/// FCP Success Message (just the header)
#[derive(Debug, Clone)]
pub struct FcpSuccessMessage {
    pub header: FcpMessageHeader,
}

impl FcpSuccessMessage {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 6 {
            return Err(Error::Protocol("Success message too short".to_string()));
        }

        let header = FcpMessageHeader::from_bytes(&bytes[0..6])?;
        header.validate()?;

        Ok(Self { header })
    }
}

/// FCP Response enum
#[derive(Debug, Clone)]
pub enum FcpResponse {
    Version(FcpVersionMessage),
    Success(FcpSuccessMessage),
    Error(FcpErrorMessage),
    Progress(FcpProgressMessage),
}

impl FcpResponse {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 6 {
            return Err(Error::Protocol("Response too short".to_string()));
        }

        let header = FcpMessageHeader::from_bytes(&bytes[0..6])?;
        header.validate()?;

        let response_type = FcpResponseType::from_u8(header.msg_type)
            .ok_or_else(|| Error::Protocol(format!("Unknown response type: 0x{:02x}", header.msg_type)))?;

        match response_type {
            FcpResponseType::Version => {
                Ok(FcpResponse::Version(FcpVersionMessage::from_bytes(bytes)?))
            }
            FcpResponseType::Success => {
                Ok(FcpResponse::Success(FcpSuccessMessage::from_bytes(bytes)?))
            }
            FcpResponseType::Error => {
                Ok(FcpResponse::Error(FcpErrorMessage::from_bytes(bytes)?))
            }
            FcpResponseType::Progress => {
                Ok(FcpResponse::Progress(FcpProgressMessage::from_bytes(bytes)?))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_serialization() {
        let header = FcpMessageHeader::new_request(0x01, 100);
        let bytes = header.to_bytes();
        let decoded = FcpMessageHeader::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.magic, FCP_MAGIC_REQUEST);
        assert_eq!(decoded.msg_type, 0x01);
        assert_eq!(decoded.payload_length, 100);
    }

    #[test]
    fn test_version_message() {
        let msg = FcpVersionMessage::new(FCP_PROTOCOL_VERSION);
        let bytes = msg.to_bytes();
        let decoded = FcpVersionMessage::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.version, FCP_PROTOCOL_VERSION);
    }
}
