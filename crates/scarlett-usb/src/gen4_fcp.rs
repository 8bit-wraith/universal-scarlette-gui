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
    seq_num: u16,  // Sequence number for Scarlett2 USB packets
    interface_num: u8,  // Interface number for control transfers
}

impl FcpProtocol {
    /// Create a new FCP protocol handler
    pub fn new(transport: Box<dyn crate::transport::UsbTransport>) -> Self {
        Self::new_with_interface(transport, 0)
    }

    /// Create a new FCP protocol handler with specific interface number
    pub fn new_with_interface(transport: Box<dyn crate::transport::UsbTransport>, interface_num: u8) -> Self {
        Self {
            transport,
            initialized: false,
            seq_num: 0,  // Start at 0, will increment on first use
            interface_num,
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

    /// Send an FCP command via USB class-specific control transfer
    ///
    /// Based on Linux kernel mixer_scarlett2.c driver (scarlett2_usb_tx/rx functions).
    /// Uses class-specific control transfers, not vendor-specific.
    fn send_command(&mut self, opcode: FcpOpcode, request_data: &[u8], response_size: usize) -> Result<Vec<u8>> {
        use crate::transport::ControlTransfer;

        // Increment sequence number (kernel starts at 1 for init)
        self.seq_num += 1;

        tracing::trace!("FCP command: {:?}, seq={}, req_len={}, resp_len={}", opcode, self.seq_num, request_data.len(), response_size);

        // Build Scarlett2 USB packet matching mixer_scarlett2.c
        // struct scarlett2_usb_packet:
        //   __le32 cmd;
        //   __le16 size;
        //   __le16 seq;
        //   __le32 error;
        //   __le32 pad;
        //   u8 data[];

        let mut request = Vec::new();
        request.extend_from_slice(&(opcode as u32).to_le_bytes());  // cmd (4 bytes)
        request.extend_from_slice(&(request_data.len() as u16).to_le_bytes());  // size (2 bytes)
        request.extend_from_slice(&(self.seq_num).to_le_bytes());  // seq (2 bytes)
        request.extend_from_slice(&0u32.to_le_bytes());  // error (4 bytes)
        request.extend_from_slice(&0u32.to_le_bytes());  // pad (4 bytes)
        request.extend_from_slice(request_data);  // data

        tracing::debug!("Scarlett2 USB packet: {} bytes total (16 byte header + {} data), seq={}", request.len(), request_data.len(), self.seq_num);

        // Send command via class-specific control transfer
        // From mixer_scarlett2.c:scarlett2_usb_tx()
        // USB_TYPE_CLASS | USB_RECIP_INTERFACE | USB_DIR_OUT = 0x21
        // Request = SCARLETT2_USB_CMD_REQ = 2
        let transfer_out = ControlTransfer::class_out(
            2,  // SCARLETT2_USB_CMD_REQ
            0,  // value
            self.interface_num as u16,  // index = interface number!
        );

        self.transport.control_out(&transfer_out, &request)?;

        // Only read response if we expect one
        if response_size == 0 {
            return Ok(Vec::new());
        }

        // Read response via class-specific IN transfer
        // From mixer_scarlett2.c:scarlett2_usb_rx()
        // USB_TYPE_CLASS | USB_RECIP_INTERFACE | USB_DIR_IN = 0xA1
        // Request = SCARLETT2_USB_CMD_RESP = 3
        let transfer_in = ControlTransfer::class_in(
            3,  // SCARLETT2_USB_CMD_RESP
            0,  // value
            self.interface_num as u16,  // index = interface number!
        );

        // Response includes 16-byte Scarlett2 header + data
        const HEADER_SIZE: usize = 16;
        let total_size = HEADER_SIZE + response_size;
        let mut response_buf = vec![0u8; total_size];
        let actual = self.transport.control_in(&transfer_in, &mut response_buf)?;

        if actual < HEADER_SIZE {
            return Err(Error::Protocol(format!(
                "Response too short: got {} bytes, need at least {} for header",
                actual, HEADER_SIZE
            )));
        }

        tracing::debug!("FCP response: {} bytes total ({} header + {} data)",
                       actual, HEADER_SIZE, actual - HEADER_SIZE);

        // TODO: Validate header (cmd, seq, size, error, pad) like kernel driver does

        // Extract just the data portion (skip 16-byte header)
        let data_len = actual - HEADER_SIZE;
        let response = response_buf[HEADER_SIZE..HEADER_SIZE + data_len].to_vec();

        Ok(response)
    }

    /// Read meter levels
    pub fn read_meters(&mut self, count: u16) -> Result<Vec<u32>> {
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
    pub fn read_mix_info(&mut self) -> Result<(u8, u8)> {
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
    pub fn read_data(&mut self, offset: u32, size: u32) -> Result<i32> {
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
    pub fn write_data(&mut self, offset: u32, size: u32, value: i32) -> Result<()> {
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

    /// Volume control constants
    /// Based on mixer_scarlett2.c
    pub const VOLUME_BIAS: i32 = 127;  // 0 dB = 127
    pub const VOLUME_MIN: i32 = 0;     // -127 dB
    pub const VOLUME_MAX: i32 = 127;   // 0 dB

    /// Configuration offsets (from mixer_scarlett2.c)
    const LINE_OUT_VOLUME_OFFSET: u32 = 0x34;
    const MUTE_SWITCH_OFFSET: u32 = 0x5c;

    /// Get volume for a specific output (0-based index)
    /// Returns volume in dB (-127 to 0)
    pub fn get_volume(&mut self, output_index: u8) -> Result<i32> {
        if !self.initialized {
            return Err(Error::Protocol("FCP not initialized".to_string()));
        }

        // Read 16-bit volume value from device
        let offset = Self::LINE_OUT_VOLUME_OFFSET + (output_index as u32 * 2);
        let raw_value = self.read_data(offset, 2)?;

        // Convert from device value to dB
        // Device stores: 0 = -127dB, 127 = 0dB
        let db = raw_value - Self::VOLUME_BIAS;

        tracing::debug!("Output {} volume: {} dB (raw={})", output_index, db, raw_value);
        Ok(db)
    }

    /// Set volume for a specific output (0-based index)
    /// volume_db: Volume in dB (-127 to 0)
    pub fn set_volume(&mut self, output_index: u8, volume_db: i32) -> Result<()> {
        if !self.initialized {
            return Err(Error::Protocol("FCP not initialized".to_string()));
        }

        // Clamp to valid range
        let volume_db = volume_db.clamp(-Self::VOLUME_BIAS, 0);

        // Convert dB to device value
        let device_value = volume_db + Self::VOLUME_BIAS;

        tracing::info!("Setting output {} volume to {} dB (raw={})", output_index, volume_db, device_value);

        // Write 16-bit volume value to device
        let offset = Self::LINE_OUT_VOLUME_OFFSET + (output_index as u32 * 2);
        self.write_data(offset, 2, device_value)?;

        Ok(())
    }

    /// Adjust volume by delta (in dB)
    pub fn adjust_volume(&mut self, output_index: u8, delta_db: i32) -> Result<i32> {
        let current = self.get_volume(output_index)?;
        let new_volume = (current + delta_db).clamp(-Self::VOLUME_BIAS, 0);
        self.set_volume(output_index, new_volume)?;
        Ok(new_volume)
    }

    /// Get mute status for a specific output
    pub fn get_mute(&mut self, output_index: u8) -> Result<bool> {
        if !self.initialized {
            return Err(Error::Protocol("FCP not initialized".to_string()));
        }

        // Read 8-bit mute value from device
        let offset = Self::MUTE_SWITCH_OFFSET + output_index as u32;
        let muted = self.read_data(offset, 1)?;

        Ok(muted != 0)
    }

    /// Set mute status for a specific output
    pub fn set_mute(&mut self, output_index: u8, muted: bool) -> Result<()> {
        if !self.initialized {
            return Err(Error::Protocol("FCP not initialized".to_string()));
        }

        tracing::info!("Setting output {} mute: {}", output_index, muted);

        // Write 8-bit mute value to device
        let offset = Self::MUTE_SWITCH_OFFSET + output_index as u32;
        self.write_data(offset, 1, if muted { 1 } else { 0 })?;

        Ok(())
    }

    /// Toggle mute for a specific output
    pub fn toggle_mute(&mut self, output_index: u8) -> Result<bool> {
        let current = self.get_mute(output_index)?;
        let new_state = !current;
        self.set_mute(output_index, new_state)?;
        Ok(new_state)
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
