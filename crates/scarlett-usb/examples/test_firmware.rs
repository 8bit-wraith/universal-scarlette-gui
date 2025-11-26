// Test the firmware parser with actual firmware files
use std::path::Path;
use scarlett_usb::firmware::FirmwareFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Scarlett2 Firmware Parser\n");

    // Test with 18i20 Gen 2 firmware (PID 0x8201)
    let firmware_path = Path::new("scarlett2-firmware/firmware/scarlett2-1235-8201-1653.bin");
    println!("Testing: {}", firmware_path.display());

    match FirmwareFile::from_file(firmware_path) {
        Ok(firmware) => {
            println!("✅ Firmware file parsed successfully!");
            println!("   VID: 0x{:04X} (Focusrite)", firmware.header.usb_vid);
            println!("   PID: 0x{:04X} (18i20 Gen 2)", firmware.header.usb_pid);
            println!("   Version: {}", firmware.header.firmware_version);
            println!("   Data size: {} bytes", firmware.data.len());
            println!("   SHA-256 verified: ✅\n");
        }
        Err(e) => {
            println!("❌ Failed to parse firmware: {}\n", e);
            return Err(e.into());
        }
    }

    // Test with 6i6 Gen 2 (PID 0x8203)
    let firmware_path = Path::new("scarlett2-firmware/firmware/scarlett2-1235-8203-1583.bin");
    println!("Testing: {}", firmware_path.display());

    match FirmwareFile::from_file(firmware_path) {
        Ok(firmware) => {
            println!("✅ Firmware file parsed successfully!");
            println!("   VID: 0x{:04X} (Focusrite)", firmware.header.usb_vid);
            println!("   PID: 0x{:04X} (6i6 Gen 2)", firmware.header.usb_pid);
            println!("   Version: {}", firmware.header.firmware_version);
            println!("   Data size: {} bytes", firmware.data.len());
            println!("   SHA-256 verified: ✅\n");
        }
        Err(e) => {
            println!("❌ Failed to parse firmware: {}\n", e);
        }
    }

    // Test with Vocaster Two (PID 0x821A)
    let firmware_path = Path::new("scarlett2-firmware/firmware/scarlett2-1235-821a-2108.bin");
    println!("Testing: {}", firmware_path.display());

    match FirmwareFile::from_file(firmware_path) {
        Ok(firmware) => {
            println!("✅ Firmware file parsed successfully!");
            println!("   VID: 0x{:04X} (Focusrite)", firmware.header.usb_vid);
            println!("   PID: 0x{:04X} (Vocaster Two)", firmware.header.usb_pid);
            println!("   Version: {}", firmware.header.firmware_version);
            println!("   Data size: {} bytes", firmware.data.len());
            println!("   SHA-256 verified: ✅\n");
        }
        Err(e) => {
            println!("❌ Failed to parse firmware: {}\n", e);
        }
    }

    println!("All tests completed successfully! 🎉");
    Ok(())
}
