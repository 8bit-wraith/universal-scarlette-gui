//! USB device detection and hotplug

use scarlett_core::{DeviceInfo, DeviceModel, Error, Result, FOCUSRITE_VENDOR_ID};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Hotplug event
#[derive(Debug, Clone)]
pub enum HotplugEvent {
    /// Device connected
    Connected(DeviceInfo),
    /// Device disconnected
    Disconnected(String), // USB path
}

/// Device detector
pub struct DeviceDetector {
    event_tx: mpsc::UnboundedSender<HotplugEvent>,
}

impl DeviceDetector {
    /// Create a new device detector
    pub fn new() -> (Self, mpsc::UnboundedReceiver<HotplugEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        (Self { event_tx }, event_rx)
    }

    /// Scan for connected Scarlett devices
    pub fn scan_devices(&self) -> Result<Vec<DeviceInfo>> {
        info!("🔍 Scanning for Focusrite Scarlett devices...");
        let mut devices = Vec::new();

        let device_list = nusb::list_devices()
            .map_err(|e| Error::Usb(format!("Failed to list USB devices: {}", e)))?;

        let mut total_devices = 0;
        let mut focusrite_count = 0;
        for device_info in device_list {
            total_devices += 1;
            // Log all USB devices in debug mode
            debug!(
                "USB Device: VID=0x{:04x}, PID=0x{:04x}",
                device_info.vendor_id(),
                device_info.product_id()
            );

            if device_info.vendor_id() == FOCUSRITE_VENDOR_ID {
                focusrite_count += 1;
                info!(
                    "🎵 Found Focusrite device: VID=0x{:04x}, PID=0x{:04x}",
                    device_info.vendor_id(),
                    device_info.product_id()
                );

                if let Some(model) = DeviceModel::from_product_id(device_info.product_id()) {
                    info!(
                        "✅ Recognized device: {} (VID: 0x{:04x}, PID: 0x{:04x})",
                        model.name(),
                        device_info.vendor_id(),
                        device_info.product_id()
                    );

                    // Get serial number
                    let serial = device_info
                        .serial_number()
                        .unwrap_or("Unknown")
                        .to_string();

                    // Create USB path identifier
                    let usb_path = format!(
                        "usb-{:03}-{:03}",
                        device_info.bus_number(),
                        device_info.device_address()
                    );

                    info!("   Serial: {}, Path: {}", serial, usb_path);

                    let device = DeviceInfo::new(model, serial, usb_path);
                    devices.push(device);
                } else {
                    warn!(
                        "❌ Unsupported Focusrite device (PID: 0x{:04x}) - please report this!",
                        device_info.product_id()
                    );
                }
            }
        }

        info!("📋 Scanned {} total USB devices", total_devices);

        if focusrite_count == 0 {
            info!("💡 No Focusrite devices found. Make sure your device is connected and powered on.");
            info!("   Looking for Vendor ID: 0x{:04x} (Focusrite)", FOCUSRITE_VENDOR_ID);
        } else {
            info!("🎵 Found {} Focusrite device(s)", focusrite_count);
        }

        info!("✨ Scan complete: {} Scarlett device(s) ready", devices.len());
        Ok(devices)
    }

    /// Start hotplug monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        info!("Starting hotplug monitoring");

        // Note: nusb 0.1 doesn't have built-in hotplug support yet
        // We'll implement polling for now, and can upgrade to proper
        // hotplug callbacks when nusb adds support

        let event_tx = self.event_tx.clone();
        let mut current_devices: Vec<DeviceInfo> = Vec::new();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

            loop {
                interval.tick().await;

                // Scan for devices
                let devices = match scan_devices_internal() {
                    Ok(d) => d,
                    Err(e) => {
                        warn!("Error scanning devices: {}", e);
                        continue;
                    }
                };

                // Check for new devices
                for device in &devices {
                    if !current_devices.iter().any(|d| d.usb_path == device.usb_path) {
                        info!("Device connected: {}", device.model);
                        let _ = event_tx.send(HotplugEvent::Connected(device.clone()));
                    }
                }

                // Check for removed devices
                for device in &current_devices {
                    if !devices.iter().any(|d| d.usb_path == device.usb_path) {
                        info!("Device disconnected: {}", device.model);
                        let _ = event_tx.send(HotplugEvent::Disconnected(device.usb_path.clone()));
                    }
                }

                current_devices = devices;
            }
        });

        Ok(())
    }
}

impl Default for DeviceDetector {
    fn default() -> Self {
        Self::new().0
    }
}

/// Internal function to scan for devices
fn scan_devices_internal() -> Result<Vec<DeviceInfo>> {
    let mut devices = Vec::new();

    let device_list = nusb::list_devices()
        .map_err(|e| Error::Usb(format!("Failed to list USB devices: {}", e)))?;

    for device_info in device_list {
        if device_info.vendor_id() == FOCUSRITE_VENDOR_ID {
            if let Some(model) = DeviceModel::from_product_id(device_info.product_id()) {
                let serial = device_info
                    .serial_number()
                    .unwrap_or("Unknown")
                    .to_string();

                let usb_path = format!(
                    "usb-{:03}-{:03}",
                    device_info.bus_number(),
                    device_info.device_address()
                );

                let device = DeviceInfo::new(model, serial, usb_path);
                devices.push(device);
            }
        }
    }

    Ok(devices)
}
