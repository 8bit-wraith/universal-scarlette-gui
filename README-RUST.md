# Scarlett Control - Rust Edition

> **Cross-platform** Focusrite Scarlett USB audio interface control panel built with Rust and Slint

This is a complete rewrite of the ALSA Scarlett Control Panel in Rust, designed to work on **macOS** and **Linux** through direct USB communication.

## Features

### Current Status 🚧

This is an **early-stage development** project. The foundation is in place, but core functionality is still being implemented.

**Completed:**
- ✅ Rust workspace structure with 5 crates
- ✅ Device model definitions for all Scarlett generations (Gen 1-4, Clarett, Vocaster)
- ✅ Basic Slint UI framework
- ✅ USB device detection infrastructure
- ✅ Configuration management foundation
- ✅ Keyboard hotkey integration framework

**In Progress:**
- 🔨 USB protocol implementation (direct USB control transfers)
- 🔨 Device-specific protocol handlers
- 🔨 Complete UI windows (routing, mixer, levels)

**Planned:**
- ⏳ Level meter real-time polling
- ⏳ Firmware update support
- ⏳ macOS keyboard volume control integration
- ⏳ Complete feature parity with C version

### Target Features (When Complete)

- **Cross-Platform**: Works on macOS and Linux via direct USB (no ALSA dependency)
- **Modern UI**: Beautiful Slint-based interface with Focusrite branding
- **System Integration**: Keyboard volume/mute keys control your Focusrite interface
- **Full Control**: Routing, mixer, level meters, hardware settings
- **Safe**: Memory-safe Rust implementation
- **Fast**: Native performance with GPU-accelerated UI

## Supported Devices

The same devices as the original ALSA Scarlett GUI:

- **Scarlett Gen 1**: 6i6, 8i6, 18i6, 18i8, 18i20
- **Scarlett Gen 2**: 6i6, 18i8, 18i20
- **Scarlett Gen 3**: Solo, 2i2, 4i4, 8i6, 18i8, 18i20
- **Scarlett Gen 4**: Solo, 2i2, 4i4, 16i16, 18i16, 18i20
- **Clarett USB**: 2Pre, 4Pre, 8Pre
- **Clarett+**: 2Pre, 4Pre, 8Pre
- **Vocaster**: One, Two

## Architecture

This project is organized as a Cargo workspace with 5 crates:

```
scarlett-gui/
├── crates/
│   ├── scarlett-core/       # Core types, traits, and protocols
│   ├── scarlett-usb/        # Direct USB communication layer
│   ├── scarlett-hotkeys/    # System keyboard integration
│   ├── scarlett-config/     # Configuration persistence
│   └── scarlett-gui/        # Slint UI application (main binary)
```

### Why Rust?

1. **Cross-Platform USB**: Direct USB access works on macOS without ALSA
2. **Memory Safety**: Eliminate entire classes of bugs
3. **Modern Tooling**: Cargo, great libraries, excellent error messages
4. **Performance**: Native speed with zero-cost abstractions
5. **Maintainability**: Strong type system catches bugs at compile time

### Why Slint?

1. **Cross-Platform**: Native on macOS, Linux, Windows
2. **Declarative**: Easy to design and maintain
3. **GPU Accelerated**: Smooth 60 FPS UI
4. **Rust-First**: Designed for Rust, not a FFI wrapper
5. **License**: Royalty-free for GPL projects

## Building

### Prerequisites

**macOS:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies (if needed)
brew install libusb
```

**Linux (Ubuntu/Debian):**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
sudo apt install libusb-1.0-0-dev pkg-config
sudo apt install libfontconfig1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev
```

### Build

```bash
# Clone the repository
git clone https://github.com/8bit-wraith/universal-scarlette-gui.git
cd universal-scarlette-gui

# Build all crates
cargo build --release

# Run the GUI
cargo run --release -p scarlett-gui
```

## Usage

### Running the Application

```bash
# Development mode (with debug logging)
RUST_LOG=debug cargo run -p scarlett-gui

# Release mode
cargo run --release -p scarlett-gui

# Or run the binary directly after building
./target/release/scarlett-gui
```

### Keyboard Volume Control

When enabled in preferences, your system volume/mute keys will control the Focusrite interface's monitor output volume.

**macOS:** Requires accessibility permissions (you'll be prompted)
**Linux:** Requires access to /dev/input (may need udev rules)

To disable keyboard control, uncheck "Enable Hotkeys" in the preferences.

## Development

### Project Structure

#### `scarlett-core`
Core abstractions and data structures:
- Device models and capabilities
- Protocol definitions
- Routing matrix
- Mixer state
- Error types

#### `scarlett-usb`
USB communication layer:
- Device detection and enumeration
- Hotplug monitoring
- Protocol implementations per generation
- USB control/bulk transfers

#### `scarlett-hotkeys`
System integration:
- Platform-specific keyboard event capture
- Volume command mapping
- macOS and Linux implementations

#### `scarlett-config`
Configuration management:
- User preferences
- Device configurations
- Save/load functionality
- Platform-specific config paths

#### `scarlett-gui`
Slint-based UI:
- Main application window
- Routing matrix view
- Mixer console
- Level meters
- Hardware settings

### Testing

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p scarlett-core

# Run with logging
RUST_LOG=debug cargo test
```

### Code Quality

```bash
# Check for errors without building
cargo check

# Run clippy (linter)
cargo clippy

# Format code
cargo fmt

# Run all checks
cargo check && cargo clippy && cargo fmt --check
```

## Roadmap

See [RUST_REWRITE_PLAN.md](RUST_REWRITE_PLAN.md) for the detailed implementation plan.

**Phase 1: Foundation** ✅ (Current)
- Project structure
- Basic UI skeleton
- Device models

**Phase 2: USB Communication** (Next)
- Device detection
- Hotplug support
- Protocol implementation

**Phase 3: Core Features**
- Routing matrix
- Mixer controls
- Level meters

**Phase 4: System Integration**
- Keyboard volume control
- Configuration persistence
- Preferences

**Phase 5: Polish**
- Firmware updates
- Error handling
- UI refinements
- Documentation

## Differences from C Version

### Advantages

1. **Cross-Platform**: Works on macOS natively (C version is Linux-only via ALSA)
2. **Memory Safe**: Rust eliminates buffer overflows, use-after-free, etc.
3. **Modern UI**: Slint provides a more modern look and GPU acceleration
4. **Keyboard Integration**: Volume keys control your interface (new feature!)
5. **Better Error Handling**: Rust's Result type makes errors explicit

### Trade-offs

1. **Development Time**: Rust has a learning curve
2. **Binary Size**: Rust binaries are larger than C (but still reasonable)
3. **Maturity**: C version is battle-tested, Rust version is new

## Contributing

This project is in early development. Contributions are welcome!

### Areas Needing Help

1. **Protocol Implementation**: Reverse engineering USB protocols from kernel drivers
2. **Testing**: Testing with actual hardware (especially Gen 1, 2, 4, Clarett, Vocaster)
3. **macOS Keyboard Integration**: Implementing CGEventTap for media keys
4. **UI Design**: Polishing the Slint interface
5. **Documentation**: Improving docs and examples

### How to Contribute

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## License

GPL-3.0-or-later (same as the original C version)

Copyright 2025 [Your Name]

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

## Credits

- **Original C Version**: [Geoffrey D. Bennett](https://github.com/geoffreybennett/alsa-scarlett-gui)
- **Rust Rewrite**: 8bit-wraith and contributors
- **Slint UI Framework**: [Slint](https://slint.dev/)
- **nusb Library**: [nusb](https://github.com/kevinmehall/nusb)

## Disclaimer

Focusrite, Scarlett, Clarett, and Vocaster are trademarks or registered trademarks of Focusrite Audio Engineering Limited. This software is not affiliated with or endorsed by Focusrite.

This is an unofficial, community-developed control panel for educational and interoperability purposes.

---

**Status**: 🚧 Early Development - Not ready for production use

**Feedback**: Please open an issue on GitHub if you encounter problems or have suggestions!
