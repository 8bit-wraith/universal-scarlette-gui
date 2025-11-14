//! USB protocol definitions and constants

/// Protocol-specific constants for different device generations
pub mod gen1 {
    // Gen 1 uses simpler ALSA mixer controls
    // Most control is done via USB control transfers
}

pub mod gen2 {
    // Scarlett2 USB Protocol constants
    // Based on Linux kernel driver: sound/usb/mixer_scarlett_gen2.c

    pub const CMD_OFFSET: u16 = 0x0000;
    pub const FLASH_SEGMENT_ID_SETTINGS: u8 = 0;
    pub const FLASH_SEGMENT_ID_FIRMWARE: u8 = 1;
}

pub mod gen3 {
    // Gen 3 uses similar protocol to Gen 2
    pub use super::gen2::*;
}

pub mod gen4 {
    // Gen 4 uses FCP (Focusrite Control Protocol)
    pub const FCP_PROTOCOL_VERSION: u8 = 1;
    pub const FCP_MAGIC_REQUEST: u8 = 0x53;
    pub const FCP_MAGIC_RESPONSE: u8 = 0x73;

    // Request types
    pub const REQUEST_REBOOT: u16 = 0x0001;
    pub const REQUEST_CONFIG_ERASE: u16 = 0x0002;
    pub const REQUEST_APP_FIRMWARE_ERASE: u16 = 0x0003;
    pub const REQUEST_APP_FIRMWARE_UPDATE: u16 = 0x0004;
    pub const REQUEST_ESP_FIRMWARE_UPDATE: u16 = 0x0005;

    // Response types
    pub const RESPONSE_VERSION: u8 = 0x00;
    pub const RESPONSE_SUCCESS: u8 = 0x01;
    pub const RESPONSE_ERROR: u8 = 0x02;
    pub const RESPONSE_PROGRESS: u8 = 0x03;
}

/// USB Control transfer parameters
#[derive(Debug, Clone, Copy)]
pub struct UsbControl {
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
}

impl UsbControl {
    pub const fn new(request_type: u8, request: u8, value: u16, index: u16) -> Self {
        Self {
            request_type,
            request,
            value,
            index,
        }
    }
}
