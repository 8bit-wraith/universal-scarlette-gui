//! Scarlett GUI - Main Application

use scarlett_config::ConfigManager;
use scarlett_hotkeys::{HotkeyManager, VolumeCommand};
use scarlett_usb::{DeviceDetector, HotplugEvent, UsbDevice};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use tracing_subscriber;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting Scarlett GUI");

    // Initialize USB subsystem
    scarlett_usb::init()?;

    // Create configuration manager
    let config = ConfigManager::new()?;
    let prefs = config.load_preferences().unwrap_or_default();
    info!("Loaded preferences");

    // Create device detector
    let (detector, mut hotplug_rx) = DeviceDetector::new();

    // Create hotkey manager
    let (hotkey_mgr, mut volume_rx) = HotkeyManager::new();

    // Create UI
    let ui = MainWindow::new()?;

    // Store current devices
    let current_devices = Arc::new(Mutex::new(Vec::new()));

    // Initial device scan
    {
        let devices = detector.scan_devices()?;
        let mut current = current_devices.lock().await;
        *current = devices.clone();

        // Update UI with devices
        let device_items: Vec<DeviceItem> = devices
            .iter()
            .map(|d| DeviceItem {
                name: d.model.name().into(),
                serial: d.serial_number.clone().into(),
                status: "Connected".into(),
            })
            .collect();

        ui.set_devices(std::rc::Rc::new(slint::VecModel::from(device_items)).into());

        if devices.is_empty() {
            ui.set_status_text("No Focusrite Scarlett devices found".into());
        } else {
            ui.set_status_text(format!("Found {} device(s)", devices.len()).into());
        }
    }

    // Start hotplug monitoring
    detector.start_monitoring().await?;
    info!("Started hotplug monitoring");

    // Start keyboard hotkey capture (if enabled)
    if prefs.enable_hotkeys {
        match hotkey_mgr.start().await {
            Ok(_) => info!("Keyboard volume control enabled"),
            Err(e) => warn!("Could not enable keyboard volume control: {}", e),
        }
    }

    // Handle scan button
    let ui_handle = ui.as_weak();
    let detector_clone = Arc::new(detector);
    let current_devices_clone = current_devices.clone();
    ui.on_scan_devices(move || {
        let ui = ui_handle.unwrap();
        let detector = detector_clone.clone();
        let current_devices = current_devices_clone.clone();

        slint::spawn_local(async move {
            match detector.scan_devices() {
                Ok(devices) => {
                    let mut current = current_devices.lock().await;
                    *current = devices.clone();

                    let device_items: Vec<DeviceItem> = devices
                        .iter()
                        .map(|d| DeviceItem {
                            name: d.model.name().into(),
                            serial: d.serial_number.clone().into(),
                            status: "Connected".into(),
                        })
                        .collect();

                    ui.set_devices(std::rc::Rc::new(slint::VecModel::from(device_items)).into());

                    if devices.is_empty() {
                        ui.set_status_text("No Focusrite Scarlett devices found".into());
                    } else {
                        ui.set_status_text(format!("Found {} device(s)", devices.len()).into());
                    }
                }
                Err(e) => {
                    error!("Failed to scan devices: {}", e);
                    ui.set_status_text(format!("Error: {}", e).into());
                }
            }
        })
        .unwrap();
    });

    // Handle device selection
    let ui_handle = ui.as_weak();
    ui.on_select_device(move |index| {
        let ui = ui_handle.unwrap();
        info!("Selected device at index {}", index);
        // TODO: Open device control window
    });

    // Handle routing button
    let ui_handle = ui.as_weak();
    ui.on_open_routing(move || {
        let ui = ui_handle.unwrap();
        info!("Opening routing window");
        // TODO: Open routing window
    });

    // Handle mixer button
    let ui_handle = ui.as_weak();
    ui.on_open_mixer(move || {
        let ui = ui_handle.unwrap();
        info!("Opening mixer window");
        // TODO: Open mixer window
    });

    // Handle levels button
    let ui_handle = ui.as_weak();
    ui.on_open_levels(move || {
        let ui = ui_handle.unwrap();
        info!("Opening levels window");
        // TODO: Open levels window
    });

    // Spawn task to handle hotplug events
    let ui_weak = ui.as_weak();
    tokio::spawn(async move {
        while let Some(event) = hotplug_rx.recv().await {
            match event {
                HotplugEvent::Connected(device_info) => {
                    info!("Device connected: {}", device_info.model);
                    // TODO: Update UI
                }
                HotplugEvent::Disconnected(path) => {
                    info!("Device disconnected: {}", path);
                    // TODO: Update UI
                }
            }
        }
    });

    // Spawn task to handle volume commands
    tokio::spawn(async move {
        while let Some(cmd) = volume_rx.recv().await {
            match cmd {
                VolumeCommand::VolumeUp => {
                    info!("Volume up");
                    // TODO: Increase device volume
                }
                VolumeCommand::VolumeDown => {
                    info!("Volume down");
                    // TODO: Decrease device volume
                }
                VolumeCommand::Mute => {
                    info!("Mute toggle");
                    // TODO: Toggle device mute
                }
            }
        }
    });

    // Run UI event loop
    ui.run()?;

    // Save preferences on exit
    config.save_preferences(&prefs)?;
    info!("Scarlett GUI exiting");

    Ok(())
}
