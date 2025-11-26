// Test opening and initializing a Scarlett device
use scarlett_usb::{DeviceDetector, UsbDevice};
use scarlett_core::DeviceModel;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("Scarlett Device Initialization Test\n");
    println!("Scanning for connected devices...\n");

    // Create detector
    let (detector, _hotplug_rx) = DeviceDetector::new();

    // Get list of devices
    let devices = detector.scan_devices()?;

    if devices.is_empty() {
        println!("❌ No Scarlett devices found!");
        println!("\nMake sure your Scarlett interface is:");
        println!("  1. Connected via USB");
        println!("  2. Powered on");
        println!("  3. Not in use by other software (close Focusrite Control)");
        return Ok(());
    }

    println!("Found {} device(s):\n", devices.len());

    for (i, info) in devices.iter().enumerate() {
        println!("{}. {} ({})", i + 1, info.model.name(), info.serial_number);
        println!("   USB: {:04x}:{:04x}", info.vendor_id, info.product_id);
        println!("   Generation: {:?}", info.model.generation());
        println!();
    }

    // Try to open the first device
    println!("Attempting to open device: {}\n", devices[0].model.name());

    // Get the nusb device handle
    let nusb_info = nusb::list_devices()?
        .find(|d| {
            d.vendor_id() == devices[0].vendor_id &&
            d.product_id() == devices[0].product_id
        })
        .ok_or("Device disappeared")?;

    let nusb_device = nusb_info.open()?;

    println!("✅ USB device opened\n");

    // Create our device wrapper
    println!("Creating device wrapper...");
    let mut device = UsbDevice::open(devices[0].clone(), nusb_device)?;
    println!("✅ Device wrapper created\n");

    // Try to initialize
    println!("Initializing device (sending INIT commands)...");
    match device.initialize() {
        Ok(()) => {
            println!("✅ Device initialized successfully!\n");

            // If it's a Gen 4 device, try some FCP commands
            if devices[0].model.generation() == scarlett_core::DeviceGeneration::Gen4 {
                println!("Testing Gen 4 FCP protocol:");

                if let Some(fcp) = device.fcp_protocol() {
                    println!("  ✅ FCP protocol accessible");

                    // Try to read device info
                    println!("  Attempting to read device info...");
                    // TODO: Add FCP commands to read device info
                } else {
                    println!("  ❌ FCP protocol not available");
                }
            }

            println!("\n🎉 SUCCESS! Your {} is responding to commands!", devices[0].model.name());
            println!("\nNext steps:");
            println!("  - Implement volume control");
            println!("  - Implement meter reading");
            println!("  - Implement routing matrix");
            println!("  - Connect to keyboard hotkeys");
        }
        Err(e) => {
            println!("❌ Initialization failed: {}\n", e);
            println!("This could mean:");
            println!("  - The device is in use by Focusrite Control (close it)");
            println!("  - The USB parameters need adjustment");
            println!("  - The FCP protocol sequence is incorrect");
            return Err(e.into());
        }
    }

    Ok(())
}
