// List all available Scarlett firmware files
use std::fs;
use std::path::Path;
use scarlett_usb::firmware::FirmwareFile;
use scarlett_core::DeviceModel;

fn pid_to_model(pid: u16) -> &'static str {
    match DeviceModel::from_product_id(pid) {
        Some(model) => model.name(),
        None => "Unknown Device",
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Scarlett Firmware Files Available\n");
    println!("{:<40} {:>8} {:>10} {:>10}", "Device", "PID", "Version", "Size");
    println!("{}", "=".repeat(72));

    let firmware_dir = Path::new("scarlett2-firmware/firmware");

    if !firmware_dir.exists() {
        eprintln!("Firmware directory not found: {}", firmware_dir.display());
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(firmware_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "bin")
                .unwrap_or(false)
        })
        .collect();

    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();

        match FirmwareFile::from_file(&path) {
            Ok(firmware) => {
                let model = pid_to_model(firmware.header.usb_pid);
                println!(
                    "{:<40} 0x{:04X} {:10} {:>7} KB",
                    model,
                    firmware.header.usb_pid,
                    firmware.header.firmware_version,
                    firmware.data.len() / 1024
                );
            }
            Err(e) => {
                eprintln!("Failed to parse {}: {}", path.display(), e);
            }
        }
    }

    println!();
    Ok(())
}
