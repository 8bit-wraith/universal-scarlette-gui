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

/// FCP Opcode categories (upper 12 bits)
pub const FCP_OPCODE_CATEGORY_INIT: u16 = 0x0;
pub const FCP_OPCODE_CATEGORY_METER: u16 = 0x1;
pub const FCP_OPCODE_CATEGORY_MIX: u16 = 0x2;
pub const FCP_OPCODE_CATEGORY_MUX: u16 = 0x3;
pub const FCP_OPCODE_CATEGORY_FLASH: u16 = 0x4;
pub const FCP_OPCODE_CATEGORY_SYNC: u16 = 0x5;
pub const FCP_OPCODE_CATEGORY_ESP_DFU: u16 = 0x6;
pub const FCP_OPCODE_CATEGORY_DATA: u16 = 0x7;

/// FCP Opcodes (category << 12 | command)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum FcpOpcode {
    // Init category
    Init1 = (FCP_OPCODE_CATEGORY_INIT << 12) | 0x000,
    CapRead = (FCP_OPCODE_CATEGORY_INIT << 12) | 0x001,
    Init2 = (FCP_OPCODE_CATEGORY_INIT << 12) | 0x002,
    Reboot = (FCP_OPCODE_CATEGORY_INIT << 12) | 0x003,

    // Meter category
    MeterInfo = (FCP_OPCODE_CATEGORY_METER << 12) | 0x000,
    MeterRead = (FCP_OPCODE_CATEGORY_METER << 12) | 0x001,

    // Mix category
    MixInfo = (FCP_OPCODE_CATEGORY_MIX << 12) | 0x000,
    MixRead = (FCP_OPCODE_CATEGORY_MIX << 12) | 0x001,
    MixWrite = (FCP_OPCODE_CATEGORY_MIX << 12) | 0x002,

    // Mux (routing) category
    MuxInfo = (FCP_OPCODE_CATEGORY_MUX << 12) | 0x000,
    MuxRead = (FCP_OPCODE_CATEGORY_MUX << 12) | 0x001,
    MuxWrite = (FCP_OPCODE_CATEGORY_MUX << 12) | 0x002,

    // Flash category
    FlashInfo = (FCP_OPCODE_CATEGORY_FLASH << 12) | 0x000,
    FlashSegmentInfo = (FCP_OPCODE_CATEGORY_FLASH << 12) | 0x001,
    FlashErase = (FCP_OPCODE_CATEGORY_FLASH << 12) | 0x002,
    FlashEraseProgress = (FCP_OPCODE_CATEGORY_FLASH << 12) | 0x003,
    FlashWrite = (FCP_OPCODE_CATEGORY_FLASH << 12) | 0x004,
    FlashRead = (FCP_OPCODE_CATEGORY_FLASH << 12) | 0x005,

    // Sync category
    SyncRead = (FCP_OPCODE_CATEGORY_SYNC << 12) | 0x004,

    // ESP DFU category
    EspDfuStart = (FCP_OPCODE_CATEGORY_ESP_DFU << 12) | 0x000,
    EspDfuWrite = (FCP_OPCODE_CATEGORY_ESP_DFU << 12) | 0x001,

    // Data category
    DataRead = (FCP_OPCODE_CATEGORY_DATA << 12) | 0x000,
    DataWrite = (FCP_OPCODE_CATEGORY_DATA << 12) | 0x001,
    DataNotify = (FCP_OPCODE_CATEGORY_DATA << 12) | 0x002,
    DevmapInfo = (FCP_OPCODE_CATEGORY_DATA << 12) | 0x00c,
    DevmapRead = (FCP_OPCODE_CATEGORY_DATA << 12) | 0x00d,
}

impl FcpOpcode {
    pub fn from_u16(val: u16) -> Option<Self> {
        match val {
            0x0000 => Some(Self::Init1),
            0x0001 => Some(Self::CapRead),
            0x0002 => Some(Self::Init2),
            0x0003 => Some(Self::Reboot),
            0x1000 => Some(Self::MeterInfo),
            0x1001 => Some(Self::MeterRead),
            0x2000 => Some(Self::MixInfo),
            0x2001 => Some(Self::MixRead),
            0x2002 => Some(Self::MixWrite),
            0x3000 => Some(Self::MuxInfo),
            0x3001 => Some(Self::MuxRead),
            0x3002 => Some(Self::MuxWrite),
            0x4000 => Some(Self::FlashInfo),
            0x4001 => Some(Self::FlashSegmentInfo),
            0x4002 => Some(Self::FlashErase),
            0x4003 => Some(Self::FlashEraseProgress),
            0x4004 => Some(Self::FlashWrite),
            0x4005 => Some(Self::FlashRead),
            0x5004 => Some(Self::SyncRead),
            0x6000 => Some(Self::EspDfuStart),
            0x6001 => Some(Self::EspDfuWrite),
            0x7000 => Some(Self::DataRead),
            0x7001 => Some(Self::DataWrite),
            0x7002 => Some(Self::DataNotify),
            0x700c => Some(Self::DevmapInfo),
            0x700d => Some(Self::DevmapRead),
            _ => None,
        }
    }
}

/// FCP Protocol Handler
///
/// Communicates with Gen 4 devices using the Focusrite Control Protocol.
/// On macOS, this bypasses the Linux kernel driver and communicates directly
/// via USB vendor-specific control transfers.
pub struct FcpProtocol {
    transport: Box<dyn crate::transport::UsbTransport>,
    initialized: bool,
}

impl FcpProtocol {
    /// Create a new FCP protocol handler
    pub fn new(transport: Box<dyn crate::transport::UsbTransport>) -> Self {
        Self {
            transport,
            initialized: false,
        }
    }

    /// Initialize the FCP protocol
    /// Must be called before sending any commands
    pub fn init(&mut self) -> Result<(Vec<u8>, Vec<u8>)> {
        use crate::transport::{ControlTransfer, Direction};

        tracing::info!("Initializing FCP protocol");

        // Step 0: Send INIT_1 command
        let step0_resp = self.send_command(FcpOpcode::Init1, &[], 24)?;
        tracing::debug!("FCP Init Step 0 complete: {} bytes", step0_resp.len());

        // Step 2: Send INIT_2 command
        let step2_resp = self.send_command(FcpOpcode::Init2, &[], 84)?;
        tracing::debug!("FCP Init Step 2 complete: {} bytes", step2_resp.len());

        // Extract firmware version from step2_resp[8..12]
        if step2_resp.len() >= 12 {
            let firmware_version = u32::from_le_bytes([
                step2_resp[8], step2_resp[9], step2_resp[10], step2_resp[11]
            ]);
            tracing::info!("Device firmware version: {}", firmware_version);
        }

        self.initialized = true;
        Ok((step0_resp, step2_resp))
    }

    /// Send an FCP command via USB vendor control transfer
    ///
    /// On Linux, this goes through hwdep ioctl, but on macOS we use direct USB.
    /// Based on Scarlett2 protocol pattern (Gen 3), using vendor-specific control transfers.
    /// These parameters may need adjustment based on actual hardware testing.
    fn send_command(&self, opcode: FcpOpcode, request_data: &[u8], response_size: usize) -> Result<Vec<u8>> {
        use crate::transport::ControlTransfer;

        tracing::trace!("FCP command: {:?}, req_len={}, resp_len={}", opcode, request_data.len(), response_size);

        // Build request packet: opcode (u16 LE) + length (u16 LE) + data
        let mut request = Vec::new();
        request.extend_from_slice(&(opcode as u16).to_le_bytes());
        request.extend_from_slice(&(request_data.len() as u16).to_le_bytes());
        request.extend_from_slice(request_data);

        tracing::debug!("FCP request packet: {} bytes total", request.len());

        // Send command via vendor-specific control transfer
        // Based on Gen 3 Scarlett2 protocol pattern:
        // - Request type 0x40 = vendor-specific, host-to-device
        // - Request 0x00 seems to be standard for Scarlett devices
        // - Value/Index typically 0x00
        let transfer_out = ControlTransfer::vendor_out(
            0x00,  // request (standard for Scarlett vendor commands)
            0x00,  // value
            0x00,  // index (interface number)
        );

        self.transport.control_out(&transfer_out, &request)?;

        // Only read response if we expect one
        if response_size == 0 {
            return Ok(Vec::new());
        }

        // Read response via vendor-specific IN transfer
        // Request type 0xC0 = vendor-specific, device-to-host
        let transfer_in = ControlTransfer::vendor_in(
            0x00,  // request (may need to be different, will test)
            0x00,  // value
            0x00,  // index
        );

        let mut response = vec![0u8; response_size];
        let actual = self.transport.control_in(&transfer_in, &mut response)?;
        response.truncate(actual);

        tracing::debug!("FCP response: {} bytes received", actual);

        Ok(response)
    }

    /// Read meter levels
    pub fn read_meters(&self, count: u16) -> Result<Vec<u32>> {
        if !self.initialized {
            return Err(Error::Protocol("FCP not initialized".to_string()));
        }

        // Build request: offset (u16), count (u16), pad (u32)
        let mut request = Vec::new();
        request.extend_from_slice(&0u16.to_le_bytes());  // offset = 0
        request.extend_from_slice(&count.to_le_bytes());
        request.extend_from_slice(&0u32.to_le_bytes());  // padding

        let response = self.send_command(FcpOpcode::MeterRead, &request, (count * 4) as usize)?;

        // Parse meter values (32-bit integers)
        let mut meters = Vec::new();
        for chunk in response.chunks_exact(4) {
            let value = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            meters.push(value);
        }

        Ok(meters)
    }

    /// Read mixer info (number of outputs and inputs)
    pub fn read_mix_info(&self) -> Result<(u8, u8)> {
        if !self.initialized {
            return Err(Error::Protocol("FCP not initialized".to_string()));
        }

        let response = self.send_command(FcpOpcode::MixInfo, &[], 8)?;

        if response.len() < 2 {
            return Err(Error::Protocol("Mix info response too short".to_string()));
        }

        Ok((response[0], response[1]))  // (num_outputs, num_inputs)
    }

    /// Read data value (1, 2, or 4 bytes)
    pub fn read_data(&self, offset: u32, size: u32) -> Result<i32> {
        if !self.initialized {
            return Err(Error::Protocol("FCP not initialized".to_string()));
        }

        let mut request = Vec::new();
        request.extend_from_slice(&offset.to_le_bytes());
        request.extend_from_slice(&size.to_le_bytes());

        let response = self.send_command(FcpOpcode::DataRead, &request, size as usize)?;

        if response.len() < size as usize {
            return Err(Error::Protocol("Data read response too short".to_string()));
        }

        // Parse based on size
        let value = match size {
            1 => i8::from_le_bytes([response[0]]) as i32,
            2 => i16::from_le_bytes([response[0], response[1]]) as i32,
            4 => i32::from_le_bytes([response[0], response[1], response[2], response[3]]),
            _ => return Err(Error::Protocol(format!("Invalid data size: {}", size))),
        };

        Ok(value)
    }

    /// Write data value (1, 2, or 4 bytes)
    pub fn write_data(&self, offset: u32, size: u32, value: i32) -> Result<()> {
        if !self.initialized {
            return Err(Error::Protocol("FCP not initialized".to_string()));
        }

        let mut request = Vec::new();
        request.extend_from_slice(&offset.to_le_bytes());
        request.extend_from_slice(&size.to_le_bytes());

        // Add value bytes based on size
        match size {
            1 => request.push(value as u8),
            2 => request.extend_from_slice(&(value as i16).to_le_bytes()),
            4 => request.extend_from_slice(&value.to_le_bytes()),
            _ => return Err(Error::Protocol(format!("Invalid data size: {}", size))),
        }

        self.send_command(FcpOpcode::DataWrite, &request, 0)?;

        Ok(())
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
