//! USB device implementation

use scarlett_core::{Device, DeviceInfo, Error, Result};
use crate::protocol::{Protocol, create_protocol};

/// USB device implementation
pub struct UsbDevice {
    info: DeviceInfo,
    protocol: Box<dyn Protocol>,
    connected: bool,
}

impl UsbDevice {
    /// Open a device
    pub fn open(info: DeviceInfo) -> Result<Self> {
        tracing::info!("Opening device: {}", info.model);

        // Create protocol handler for this device
        let protocol = create_protocol(info.model.generation());

        Ok(Self {
            info,
            protocol,
            connected: true,
        })
    }

    /// Get mutable access to protocol
    pub fn protocol_mut(&mut self) -> &mut dyn Protocol {
        self.protocol.as_mut()
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
        // TODO: Return actual number based on device model
        match self.info.model {
            scarlett_core::DeviceModel::Scarlett2i2Gen3 => 2,
            scarlett_core::DeviceModel::Scarlett4i4Gen3 => 4,
            scarlett_core::DeviceModel::Scarlett18i20Gen3 => 18,
            _ => 0,
        }
    }

    fn num_outputs(&self) -> usize {
        // TODO: Return actual number based on device model
        match self.info.model {
            scarlett_core::DeviceModel::Scarlett2i2Gen3 => 2,
            scarlett_core::DeviceModel::Scarlett4i4Gen3 => 4,
            scarlett_core::DeviceModel::Scarlett18i20Gen3 => 20,
            _ => 0,
        }
    }

    fn num_mixer_inputs(&self) -> usize {
        // TODO: Return actual number based on device model
        match self.info.model {
            scarlett_core::DeviceModel::Scarlett18i20Gen3 => 25,
            scarlett_core::DeviceModel::Scarlett4i4Gen3 => 8,
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
