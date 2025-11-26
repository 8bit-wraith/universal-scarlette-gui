//! Linux keyboard event capture using evdev

use super::VolumeCommand;
use scarlett_core::Result;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

// TODO: Implement Linux keyboard capture using evdev
// This requires:
// 1. Find keyboard device in /dev/input/event*
// 2. Open device and read events
// 3. Filter for KEY_VOLUMEUP, KEY_VOLUMEDOWN, KEY_MUTE
// 4. Send VolumeCommand events when keys are pressed

pub async fn start_capture(command_tx: mpsc::UnboundedSender<VolumeCommand>) -> Result<()> {
    info!("Starting Linux keyboard event capture");

    tokio::spawn(async move {
        warn!("Linux keyboard capture not yet implemented");

        // TODO: Implementation will:
        // 1. Use evdev crate to enumerate input devices
        // 2. Find device with volume key capabilities
        // 3. Listen for key events
        // 4. Send commands via command_tx

        // For now, just keep task alive
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    Ok(())
}
