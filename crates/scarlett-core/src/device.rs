//! Device models and information

use serde::{Deserialize, Serialize};
use std::fmt;

/// Scarlett device generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceGeneration {
    Gen1,
    Gen2,
    Gen3,
    Gen4,
    Clarett,
    ClarettPlus,
    Vocaster,
}

/// Specific device models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceModel {
    // Gen 1
    Scarlett6i6Gen1,
    Scarlett8i6Gen1,
    Scarlett18i6Gen1,
    Scarlett18i8Gen1,
    Scarlett18i20Gen1,

    // Gen 2
    Scarlett6i6Gen2,
    Scarlett18i8Gen2,
    Scarlett18i20Gen2,

    // Gen 3
    ScarlettSoloGen3,
    Scarlett2i2Gen3,
    Scarlett4i4Gen3,
    Scarlett8i6Gen3,
    Scarlett18i8Gen3,
    Scarlett18i20Gen3,

    // Gen 4
    ScarlettSoloGen4,
    Scarlett2i2Gen4,
    Scarlett4i4Gen4,
    Scarlett16i16Gen4,
    Scarlett18i16Gen4,
    Scarlett18i20Gen4,

    // Clarett USB
    Clarett2PreUsb,
    Clarett4PreUsb,
    Clarett8PreUsb,

    // Clarett+
    Clarett2PrePlus,
    Clarett4PrePlus,
    Clarett8PrePlus,

    // Vocaster
    VocasterOne,
    VocasterTwo,
}

impl DeviceModel {
    /// Get the device generation
    pub fn generation(&self) -> DeviceGeneration {
        match self {
            Self::Scarlett6i6Gen1 | Self::Scarlett8i6Gen1 | Self::Scarlett18i6Gen1
            | Self::Scarlett18i8Gen1 | Self::Scarlett18i20Gen1 => DeviceGeneration::Gen1,

            Self::Scarlett6i6Gen2 | Self::Scarlett18i8Gen2 | Self::Scarlett18i20Gen2
                => DeviceGeneration::Gen2,

            Self::ScarlettSoloGen3 | Self::Scarlett2i2Gen3 | Self::Scarlett4i4Gen3
            | Self::Scarlett8i6Gen3 | Self::Scarlett18i8Gen3 | Self::Scarlett18i20Gen3
                => DeviceGeneration::Gen3,

            Self::ScarlettSoloGen4 | Self::Scarlett2i2Gen4 | Self::Scarlett4i4Gen4
            | Self::Scarlett16i16Gen4 | Self::Scarlett18i16Gen4 | Self::Scarlett18i20Gen4
                => DeviceGeneration::Gen4,

            Self::Clarett2PreUsb | Self::Clarett4PreUsb | Self::Clarett8PreUsb
                => DeviceGeneration::Clarett,

            Self::Clarett2PrePlus | Self::Clarett4PrePlus | Self::Clarett8PrePlus
                => DeviceGeneration::ClarettPlus,

            Self::VocasterOne | Self::VocasterTwo => DeviceGeneration::Vocaster,
        }
    }

    /// Get the USB Product ID for this device
    pub fn product_id(&self) -> u16 {
        match self {
            // Gen 1 (PIDs from Linux kernel driver)
            Self::Scarlett6i6Gen1 => 0x8203,
            Self::Scarlett8i6Gen1 => 0x8204,
            Self::Scarlett18i6Gen1 => 0x8201,
            Self::Scarlett18i8Gen1 => 0x8202,
            Self::Scarlett18i20Gen1 => 0x8200,

            // Gen 2
            Self::Scarlett6i6Gen2 => 0x8211,
            Self::Scarlett18i8Gen2 => 0x8210,
            Self::Scarlett18i20Gen2 => 0x820C,

            // Gen 3
            Self::ScarlettSoloGen3 => 0x8215,
            Self::Scarlett2i2Gen3 => 0x8214,
            Self::Scarlett4i4Gen3 => 0x8213,
            Self::Scarlett8i6Gen3 => 0x8212,
            Self::Scarlett18i8Gen3 => 0x8217,
            Self::Scarlett18i20Gen3 => 0x8218,

            // Gen 4
            Self::ScarlettSoloGen4 => 0x8223,
            Self::Scarlett2i2Gen4 => 0x8222,
            Self::Scarlett4i4Gen4 => 0x8221,
            Self::Scarlett16i16Gen4 => 0x8220,
            Self::Scarlett18i16Gen4 => 0x821F,
            Self::Scarlett18i20Gen4 => 0x821E,

            // Clarett USB
            Self::Clarett2PreUsb => 0x8206,
            Self::Clarett4PreUsb => 0x8207,
            Self::Clarett8PreUsb => 0x8208,

            // Clarett+
            Self::Clarett2PrePlus => 0x820A,
            Self::Clarett4PrePlus => 0x820B,
            Self::Clarett8PrePlus => 0x820C,

            // Vocaster
            Self::VocasterOne => 0x8209,
            Self::VocasterTwo => 0x8219,
        }
    }

    /// Get the friendly name of the device
    pub fn name(&self) -> &'static str {
        match self {
            Self::Scarlett6i6Gen1 => "Scarlett 6i6 (1st Gen)",
            Self::Scarlett8i6Gen1 => "Scarlett 8i6 (1st Gen)",
            Self::Scarlett18i6Gen1 => "Scarlett 18i6 (1st Gen)",
            Self::Scarlett18i8Gen1 => "Scarlett 18i8 (1st Gen)",
            Self::Scarlett18i20Gen1 => "Scarlett 18i20 (1st Gen)",

            Self::Scarlett6i6Gen2 => "Scarlett 6i6 (2nd Gen)",
            Self::Scarlett18i8Gen2 => "Scarlett 18i8 (2nd Gen)",
            Self::Scarlett18i20Gen2 => "Scarlett 18i20 (2nd Gen)",

            Self::ScarlettSoloGen3 => "Scarlett Solo (3rd Gen)",
            Self::Scarlett2i2Gen3 => "Scarlett 2i2 (3rd Gen)",
            Self::Scarlett4i4Gen3 => "Scarlett 4i4 (3rd Gen)",
            Self::Scarlett8i6Gen3 => "Scarlett 8i6 (3rd Gen)",
            Self::Scarlett18i8Gen3 => "Scarlett 18i8 (3rd Gen)",
            Self::Scarlett18i20Gen3 => "Scarlett 18i20 (3rd Gen)",

            Self::ScarlettSoloGen4 => "Scarlett Solo (4th Gen)",
            Self::Scarlett2i2Gen4 => "Scarlett 2i2 (4th Gen)",
            Self::Scarlett4i4Gen4 => "Scarlett 4i4 (4th Gen)",
            Self::Scarlett16i16Gen4 => "Scarlett 16i16 (4th Gen)",
            Self::Scarlett18i16Gen4 => "Scarlett 18i16 (4th Gen)",
            Self::Scarlett18i20Gen4 => "Scarlett 18i20 (4th Gen)",

            Self::Clarett2PreUsb => "Clarett 2Pre USB",
            Self::Clarett4PreUsb => "Clarett 4Pre USB",
            Self::Clarett8PreUsb => "Clarett 8Pre USB",

            Self::Clarett2PrePlus => "Clarett+ 2Pre",
            Self::Clarett4PrePlus => "Clarett+ 4Pre",
            Self::Clarett8PrePlus => "Clarett+ 8Pre",

            Self::VocasterOne => "Vocaster One",
            Self::VocasterTwo => "Vocaster Two",
        }
    }

    /// Try to identify a device model from USB Product ID
    pub fn from_product_id(pid: u16) -> Option<Self> {
        match pid {
            0x8200 => Some(Self::Scarlett18i20Gen1),
            0x8201 => Some(Self::Scarlett18i6Gen1),
            0x8202 => Some(Self::Scarlett18i8Gen1),
            0x8203 => Some(Self::Scarlett6i6Gen1),
            0x8204 => Some(Self::Scarlett8i6Gen1),

            0x820C => Some(Self::Scarlett18i20Gen2),
            0x8210 => Some(Self::Scarlett18i8Gen2),
            0x8211 => Some(Self::Scarlett6i6Gen2),

            0x8212 => Some(Self::Scarlett8i6Gen3),
            0x8213 => Some(Self::Scarlett4i4Gen3),
            0x8214 => Some(Self::Scarlett2i2Gen3),
            0x8215 => Some(Self::ScarlettSoloGen3),
            0x8217 => Some(Self::Scarlett18i8Gen3),
            0x8218 => Some(Self::Scarlett18i20Gen3),

            0x821E => Some(Self::Scarlett18i20Gen4),
            0x821F => Some(Self::Scarlett18i16Gen4),
            0x8220 => Some(Self::Scarlett16i16Gen4),
            0x8221 => Some(Self::Scarlett4i4Gen4),
            0x8222 => Some(Self::Scarlett2i2Gen4),
            0x8223 => Some(Self::ScarlettSoloGen4),

            0x8206 => Some(Self::Clarett2PreUsb),
            0x8207 => Some(Self::Clarett4PreUsb),
            0x8208 => Some(Self::Clarett8PreUsb),

            0x820A => Some(Self::Clarett2PrePlus),
            0x820B => Some(Self::Clarett4PrePlus),

            0x8209 => Some(Self::VocasterOne),
            0x8219 => Some(Self::VocasterTwo),

            _ => None,
        }
    }
}

impl fmt::Display for DeviceModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub model: DeviceModel,
    pub serial_number: String,
    pub firmware_version: Option<String>,
    pub usb_path: String,
}

impl DeviceInfo {
    pub fn new(model: DeviceModel, serial_number: String, usb_path: String) -> Self {
        Self {
            model,
            serial_number,
            firmware_version: None,
            usb_path,
        }
    }
}

/// Trait for device operations
pub trait Device: Send + Sync {
    /// Get device information
    fn info(&self) -> &DeviceInfo;

    /// Check if device is connected
    fn is_connected(&self) -> bool;

    /// Get the number of input channels
    fn num_inputs(&self) -> usize;

    /// Get the number of output channels
    fn num_outputs(&self) -> usize;

    /// Get the number of mixer inputs
    fn num_mixer_inputs(&self) -> usize;

    /// Has hardware mixer
    fn has_mixer(&self) -> bool;

    /// Has routing matrix
    fn has_routing(&self) -> bool;
}
