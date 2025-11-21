//! USB device implementation
//!
//! Wires together device detection, USB transport, and protocol layers

use scarlett_core::{Device, DeviceInfo, DeviceGeneration, Result};
use crate::direct_usb_transport::DirectUsbTransport;
use crate::gen4_fcp::FcpProtocol;
use crate::gen3_protocol::Scarlett2Protocol;
use nusb::Device as NusbDevice;

/// USB device wrapper that combines transport + protocol
pub struct UsbDevice {
    info: DeviceInfo,
    device_type: DeviceType,
    connected: bool,
}

/// Device type with protocol-specific state
enum DeviceType {
    /// Gen 4 "big" devices using FCP protocol
    Gen4Fcp {
        protocol: FcpProtocol,
    },
    /// Gen 2/3 devices using Scarlett2 protocol
    Gen2Or3 {
        protocol: Scarlett2Protocol,
    },
}

impl UsbDevice {
    /// Open and initialize a device
    pub fn open(info: DeviceInfo, nusb_device: NusbDevice) -> Result<Self> {
        tracing::info!("Opening device: {} ({})", info.model.name(), info.serial_number);

        let generation = info.model.generation();

        let device_type = match generation {
            DeviceGeneration::Gen4 => {
                // Gen 4 "big" devices use FCP
                tracing::info!("Initializing Gen 4 FCP protocol");

                // Create USB transport
                let transport = DirectUsbTransport::new(nusb_device, 0)?;

                // Create FCP protocol handler (boxing the transport)
                let protocol = FcpProtocol::new(Box::new(transport));

                DeviceType::Gen4Fcp { protocol }
            }
            DeviceGeneration::Gen2 | DeviceGeneration::Gen3 => {
                // Gen 2/3 use Scarlett2 protocol
                tracing::info!("Initializing Gen 2/3 Scarlett2 protocol");

                let protocol = Scarlett2Protocol::new(nusb_device);

                DeviceType::Gen2Or3 { protocol }
            }
            _ => {
                return Err(scarlett_core::Error::Protocol(
                    format!("Device generation {:?} not yet supported", generation)
                ));
            }
        };

        Ok(Self {
            info,
            device_type,
            connected: true,
        })
    }

    /// Initialize device (send INIT commands, etc.)
    pub fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing device: {}", self.info.model.name());

        match &mut self.device_type {
            DeviceType::Gen4Fcp { protocol } => {
                // Send FCP initialization commands
                tracing::debug!("Sending FCP INIT commands");
                let (resp1, resp2) = protocol.init()?;

                tracing::debug!("INIT_1 response: {} bytes", resp1.len());
                tracing::debug!("INIT_2 response: {} bytes", resp2.len());

                tracing::info!("Gen 4 device initialized successfully");
            }
            DeviceType::Gen2Or3 { .. } => {
                // Gen 2/3 initialization (TODO)
                tracing::info!("Gen 2/3 initialization not yet implemented");
            }
        }

        Ok(())
    }

    /// Get access to Gen 4 FCP protocol
    pub fn fcp_protocol(&mut self) -> Option<&mut FcpProtocol> {
        match &mut self.device_type {
            DeviceType::Gen4Fcp { protocol } => Some(protocol),
            _ => None,
        }
    }

    /// Get access to Gen 2/3 Scarlett2 protocol
    pub fn scarlett2_protocol(&mut self) -> Option<&mut Scarlett2Protocol> {
        match &mut self.device_type {
            DeviceType::Gen2Or3 { protocol } => Some(protocol),
            _ => None,
        }
    }
}

impl Device for UsbDevice {
    fn info(&self) -> &DeviceInfo {
        &self.info
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn num_inputs(&self) -> usize {
        use scarlett_core::DeviceModel::*;
        match self.info.model {
            Scarlett2i2Gen3 | Scarlett2i2Gen4 => 2,
            Scarlett4i4Gen3 | Scarlett4i4Gen4 => 4,
            Scarlett6i6Gen2 => 6,
            Scarlett8i6Gen3 => 8,
            Scarlett18i8Gen2 | Scarlett18i8Gen3 => 18,
            Scarlett18i20Gen2 | Scarlett18i20Gen3 | Scarlett18i20Gen4 => 18,
            Scarlett16i16Gen4 => 16,
            Scarlett18i16Gen4 => 18,
            _ => 0,
        }
    }

    fn num_outputs(&self) -> usize {
        use scarlett_core::DeviceModel::*;
        match self.info.model {
            Scarlett2i2Gen3 | Scarlett2i2Gen4 => 2,
            Scarlett4i4Gen3 | Scarlett4i4Gen4 => 4,
            Scarlett6i6Gen2 => 6,
            Scarlett8i6Gen3 => 6,
            Scarlett18i8Gen2 | Scarlett18i8Gen3 => 8,
            Scarlett18i20Gen2 | Scarlett18i20Gen3 | Scarlett18i20Gen4 => 20,
            Scarlett16i16Gen4 => 16,
            Scarlett18i16Gen4 => 16,
            _ => 0,
        }
    }

    fn num_mixer_inputs(&self) -> usize {
        use scarlett_core::DeviceModel::*;
        match self.info.model {
            Scarlett18i20Gen2 | Scarlett18i20Gen3 | Scarlett18i20Gen4 => 25,
            Scarlett4i4Gen3 | Scarlett4i4Gen4 => 8,
            Scarlett8i6Gen3 => 18,
            Scarlett18i8Gen3 => 20,
            Scarlett16i16Gen4 => 18,
            Scarlett18i16Gen4 => 20,
            _ => 0,
        }
    }

    fn has_mixer(&self) -> bool {
        // Solo and 2i2 don't have mixers
        !matches!(
            self.info.model,
            scarlett_core::DeviceModel::ScarlettSoloGen3
                | scarlett_core::DeviceModel::Scarlett2i2Gen3
                | scarlett_core::DeviceModel::ScarlettSoloGen4
                | scarlett_core::DeviceModel::Scarlett2i2Gen4
        )
    }

    fn has_routing(&self) -> bool {
        // Most devices have routing except Solo and 2i2
        self.has_mixer()
    }
}
