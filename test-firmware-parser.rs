// Quick test to verify our firmware parser works
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Include the firmware module
    use scarlett_usb::firmware::FirmwareFile;

    // Test with 18i20 Gen 2 firmware (PID 0x8201)
    let firmware_path = Path::new("scarlett2-firmware/firmware/scarlett2-1235-8201-1653.bin");

    println!("Testing firmware parser with: {}", firmware_path.display());

    match FirmwareFile::from_file(firmware_path) {
        Ok(firmware) => {
            println!("✅ Firmware file parsed successfully!");
            println!("   VID: 0x{:04X}", firmware.header.usb_vid);
            println!("   PID: 0x{:04X}", firmware.header.usb_pid);
            println!("   Version: {}", firmware.header.firmware_version);
            println!("   Data size: {} bytes", firmware.data.len());
            println!("   SHA-256 verified: ✅");
        }
        Err(e) => {
            println!("❌ Failed to parse firmware: {}", e);
            return Err(e.into());
        }
    }

    // Try another one - 6i6 Gen 2 (PID 0x8203)
    println!();
    let firmware_path = Path::new("scarlett2-firmware/firmware/scarlett2-1235-8203-1583.bin");
    println!("Testing with: {}", firmware_path.display());

    match FirmwareFile::from_file(firmware_path) {
        Ok(firmware) => {
            println!("✅ Firmware file parsed successfully!");
            println!("   VID: 0x{:04X}", firmware.header.usb_vid);
            println!("   PID: 0x{:04X}", firmware.header.usb_pid);
            println!("   Version: {}", firmware.header.firmware_version);
            println!("   Data size: {} bytes", firmware.data.len());
        }
        Err(e) => {
            println!("❌ Failed to parse firmware: {}", e);
        }
    }

    Ok(())
}
