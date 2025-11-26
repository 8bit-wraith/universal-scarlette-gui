//! Test actual USB communication with connected Scarlett devices
//! This will attempt to:
//! 1. Detect all connected Scarlett devices
//! 2. Try to initialize communication with each one
//! 3. Read firmware version / device info
//! 4. Test basic commands

use scarlett_core::{DeviceModel, FOCUSRITE_VENDOR_ID};
use scarlett_usb::direct_usb_transport::DirectUsbTransport;
use scarlett_usb::gen4_fcp::FcpProtocol;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable debug logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("🔍 Scanning for Scarlett devices...\n");

    let device_list = nusb::list_devices()?;

    let mut scarlett_devices = Vec::new();

    for device_info in device_list {
        if device_info.vendor_id() == FOCUSRITE_VENDOR_ID {
            if let Some(model) = DeviceModel::from_product_id(device_info.product_id()) {
                scarlett_devices.push((device_info, model));
            }
        }
    }

    if scarlett_devices.is_empty() {
        println!("❌ No Scarlett devices found!");
        return Ok(());
    }

    println!("✅ Found {} device(s):\n", scarlett_devices.len());

    for (i, (device_info, model)) in scarlett_devices.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Device #{}: {}", i + 1, model.name());
        println!("  VID:    0x{:04X}", device_info.vendor_id());
        println!("  PID:    0x{:04X}", device_info.product_id());
        println!("  Serial: {}", device_info.serial_number().unwrap_or("Unknown"));
        println!("  Gen:    {:?}", model.generation());
        println!();

        // Try to communicate based on generation
        match model.generation() {
            scarlett_core::DeviceGeneration::Gen4 => {
                println!("🎛️  Attempting Gen 4 FCP communication...");
                test_gen4_fcp(&device_info)?;
            }
            scarlett_core::DeviceGeneration::Gen3 => {
                println!("🎛️  Gen 3 Scarlett2 protocol");
                println!("  📝 TODO: Implement Scarlett2 USB commands");
            }
            scarlett_core::DeviceGeneration::Gen2 => {
                println!("🎛️  Gen 2 protocol not yet implemented");
            }
            _ => {
                println!("⚠️  Unknown generation");
            }
        }

        println!();
    }

    Ok(())
}

fn test_gen4_fcp(device_info: &nusb::DeviceInfo) -> Result<(), Box<dyn std::error::Error>> {
    println!("  → Opening USB device...");

    // Open the nusb device
    let usb_device = device_info.open()?;

    println!("  → Finding vendor-specific interface (class 255)...");
    // Create DirectUsbTransport with vendor interface
    let transport = DirectUsbTransport::new_vendor_interface(usb_device)?;
    let interface_num = transport.interface_number();
    println!("  ✅ Found and claimed vendor interface {}", interface_num);

    println!("  → Creating FCP protocol handler...");
    // Create FCP protocol with the interface number
    let mut fcp = FcpProtocol::new_with_interface(Box::new(transport), interface_num);

    println!("  → Sending INIT command...");
    // Try initialization
    match fcp.init() {
        Ok(_) => {
            println!("  ✅ INIT successful!");

            // Try volume control!
            println!("\n  → Testing volume control on output 0 (Monitor)...");

            // Read current volume
            match fcp.get_volume(0) {
                Ok(vol) => println!("  📊 Current volume: {} dB", vol),
                Err(e) => println!("  ⚠️  Failed to read volume: {}", e),
            }

            // Read mute status
            match fcp.get_mute(0) {
                Ok(muted) => println!("  🔇 Mute status: {}", if muted { "MUTED" } else { "UNMUTED" }),
                Err(e) => println!("  ⚠️  Failed to read mute: {}", e),
            }

            // Optionally test volume change (commented out for safety)
            // println!("\n  → Testing volume adjustment (+1 dB)...");
            // match fcp.adjust_volume(0, 1) {
            //     Ok(new_vol) => println!("  ✅ New volume: {} dB", new_vol),
            //     Err(e) => println!("  ❌ Failed to adjust volume: {}", e),
            // }
        }
        Err(e) => println!("  ❌ INIT failed: {}", e),
    }

    Ok(())
}
