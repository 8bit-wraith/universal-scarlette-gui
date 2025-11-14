# Rust Rewrite Implementation Plan

## Project: Universal Scarlett GUI (Rust + Slint)

### Overview
Rewrite the ALSA Scarlett Control Panel from C/GTK4 to Rust/Slint with direct USB access for cross-platform support (macOS + Linux).

### Key Goals
1. **Cross-Platform**: Direct USB communication bypassing ALSA/CoreAudio
2. **Modern UI**: Slint-based declarative UI matching GTK4 aesthetics
3. **System Integration**: Keyboard volume controls (up/down/mute) control Focusrite interface
4. **Safety**: Memory-safe Rust implementation
5. **Maintainability**: Clean architecture with clear abstractions

---

## Architecture

### Cargo Workspace Structure
```
scarlett-gui/
├── Cargo.toml (workspace)
├── crates/
│   ├── scarlett-core/       # Device models, traits, protocols
│   ├── scarlett-usb/        # Direct USB communication layer
│   ├── scarlett-hotkeys/    # System keyboard integration
│   ├── scarlett-config/     # Configuration persistence
│   └── scarlett-gui/        # Slint UI application
└── firmware/                # Firmware files (optional)
```

### Core Crates

#### 1. `scarlett-core`
**Purpose**: Device models, traits, and protocol definitions

**Key Components**:
- `Device` trait - Common interface for all Scarlett devices
- `DeviceInfo` - Model, generation, capabilities
- Protocol enums and constants
- Audio routing data structures
- Mixer state management
- Level meter data

**Dependencies**:
- `serde` - Serialization
- `thiserror` - Error handling
- `tracing` - Logging

#### 2. `scarlett-usb`
**Purpose**: Direct USB communication with Focusrite hardware

**Key Components**:
- USB device enumeration and detection
- Hotplug event handling
- Protocol implementation for all generations:
  - Gen 1 (6i6, 8i6, 18i6, 18i8, 18i20)
  - Gen 2 (6i6, 18i8, 18i20)
  - Gen 3 (Solo, 2i2, 4i4, 8i6, 18i8, 18i20)
  - Gen 4 (Solo, 2i2, 4i4, 16i16, 18i16, 18i20)
  - Clarett USB/+ (2Pre, 4Pre, 8Pre)
  - Vocaster (One, Two)
- USB control transfer abstraction
- Level meter polling
- Firmware update support

**Dependencies**:
- `nusb` - Modern USB library (cross-platform)
- `tokio` - Async runtime
- `scarlett-core` - Device models

**Key USB Details**:
- Vendor ID: 0x1235 (Focusrite)
- Product IDs: Device-specific (to be mapped)
- Control endpoint for configuration
- Bulk/Interrupt endpoint for level meters
- HID interface for some models

#### 3. `scarlett-hotkeys`
**Purpose**: System keyboard media key integration

**Key Components**:
- Platform-specific keyboard hook:
  - **macOS**: IOKit/CGEventTap for media keys
  - **Linux**: evdev for media key capture
- Volume control mapping to device monitor output
- Mute state synchronization
- OSD (On-Screen Display) integration

**Dependencies**:
- `rdev` - Cross-platform input listening
- **macOS**: `core-foundation`, `cocoa`
- **Linux**: `evdev`

**Implementation Details**:
- Intercept volume up/down/mute keys
- Map to Focusrite monitor output volume
- Provide visual feedback (system OSD or custom)
- Background service/daemon mode

#### 4. `scarlett-config`
**Purpose**: Configuration persistence and management

**Key Components**:
- Save/Load device configurations
- User preferences
- Platform-specific config locations:
  - **macOS**: `~/Library/Application Support/ScarlettGUI/`
  - **Linux**: `~/.config/scarlett-gui/`
- Configuration format: RON or TOML

**Dependencies**:
- `serde`
- `ron` or `toml`
- `directories` - Platform-specific paths

#### 5. `scarlett-gui`
**Purpose**: Slint-based user interface

**Key Components**:
- Main application window
- Device selection
- Routing matrix view
- Mixer console with:
  - Volume faders
  - Pan dials
  - Stereo pair linking
  - Level meters (VU style)
- Hardware settings panel
- Firmware update wizard
- Configuration management

**Dependencies**:
- `slint` - UI framework
- `tokio` - Async runtime
- `scarlett-core`, `scarlett-usb`, `scarlett-config`

---

## USB Protocol Implementation

### Research Strategy
1. **Analyze Linux Kernel Driver**:
   - Study `sound/usb/mixer_scarlett_gen2.c`
   - Extract USB command structures
   - Document protocol for each generation

2. **USB Sniffing** (if needed):
   - Use Wireshark/USBPcap
   - Capture Windows Control software traffic
   - Reverse engineer missing commands

3. **Incremental Implementation**:
   - Start with device detection
   - Implement read-only control queries
   - Add write operations
   - Add level meter streaming
   - Add firmware update

### Protocol Layers
```
┌─────────────────────────────────────┐
│   Application Layer (Rust API)      │
│   - get_routing(), set_volume()     │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│   Protocol Layer (Gen-specific)     │
│   - Gen1Protocol, Gen2Protocol...   │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│   USB Transport Layer               │
│   - control_transfer()              │
│   - bulk_transfer()                 │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│   nusb (USB Library)                │
└─────────────────────────────────────┘
```

---

## Slint UI Design

### Design System
- **Color Scheme**: Dark theme matching Focusrite branding
  - Primary: Focusrite Red (#E2231A or similar)
  - Background: Dark gray (#2B2B2B)
  - Surface: Lighter gray (#3C3C3C)
  - Text: White/Light gray
- **Typography**: Sans-serif, clean and modern
- **Components**:
  - Custom rotary dial (matching gtkdial aesthetics)
  - Linear faders with dB scale
  - VU meters with peak hold
  - Routing matrix with drag-and-drop

### Window Structure
1. **Main Window** - Device selection and overview
2. **Routing Window** - Audio routing matrix
3. **Mixer Window** - Mixing console
4. **Levels Window** - Real-time level meters
5. **Hardware Window** - Device-specific settings
6. **Firmware Window** - Update wizard

### Responsive Layout
- Adapt to different device capabilities
- Collapsible sections for complex interfaces
- Resizable windows with saved preferences

---

## Keyboard Volume Control Implementation

### macOS Implementation
```rust
// Use CGEventTap to intercept media keys
// NX_KEYTYPE_SOUND_UP, NX_KEYTYPE_SOUND_DOWN, NX_KEYTYPE_MUTE
// Map to Scarlett monitor output volume control
```

**Challenges**:
- Requires accessibility permissions
- Compete with system volume control
- Provide user preference to enable/disable

### Linux Implementation
```rust
// Use evdev to capture media keys
// KEY_VOLUMEUP, KEY_VOLUMEDOWN, KEY_MUTE
// Map to Scarlett output volume
```

**Challenges**:
- Requires udev rules for device access
- Different desktop environments (GNOME, KDE, etc.)

### Volume Mapping
- Linear or logarithmic volume curve
- dB to percentage conversion
- Step size configuration
- Mute state persistence

---

## Implementation Phases

### Phase 1: Project Setup (Current)
- [x] Create Cargo workspace
- [ ] Set up basic Slint UI skeleton
- [ ] Add core dependencies
- [ ] Create basic project structure

### Phase 2: USB Foundation
- [ ] Implement device detection (enumerate USB devices)
- [ ] Identify Scarlett devices by VID/PID
- [ ] Implement hotplug detection
- [ ] Create Device trait and basic models

### Phase 3: Protocol Implementation (Gen 3 Focus)
- [ ] Research Gen 3 protocol from kernel driver
- [ ] Implement read operations (get routing, get volume, etc.)
- [ ] Implement write operations (set routing, set volume, etc.)
- [ ] Test with Gen 3 hardware

### Phase 4: Basic UI
- [ ] Main window with device selection
- [ ] Simple volume control
- [ ] Basic routing view
- [ ] Connect UI to USB backend

### Phase 5: Keyboard Integration
- [ ] Platform detection
- [ ] macOS media key capture
- [ ] Linux media key capture
- [ ] Volume mapping logic
- [ ] User preferences for hotkey behavior

### Phase 6: Advanced Features
- [ ] Mixer window with stereo pairs
- [ ] Level meters with real-time polling
- [ ] Hardware settings panel
- [ ] Configuration save/load

### Phase 7: Multi-Generation Support
- [ ] Gen 1 protocol implementation
- [ ] Gen 2 protocol implementation
- [ ] Gen 4 protocol implementation
- [ ] Clarett protocol implementation
- [ ] Vocaster protocol implementation

### Phase 8: Firmware & Polish
- [ ] Firmware update functionality
- [ ] Error handling and recovery
- [ ] Performance optimization
- [ ] UI polish and animations
- [ ] Documentation

### Phase 9: Distribution
- [ ] macOS app bundle (.app)
- [ ] DMG installer for macOS
- [ ] Linux packages (deb, rpm, flatpak)
- [ ] Auto-update mechanism
- [ ] Installation documentation

---

## Technical Decisions

### Why Slint?
- **Cross-platform**: Native on macOS/Linux/Windows
- **Declarative**: Easy to design and maintain
- **Performance**: Compiled, GPU-accelerated
- **Rust-native**: First-class Rust integration
- **License**: Royalty-free for GPL projects

### Why nusb?
- **Modern**: Actively maintained, async-first
- **Cross-platform**: Works on macOS/Linux/Windows
- **Safe**: Rust-native with good error handling
- **Features**: Hotplug, control/bulk transfers, descriptor parsing

### Why Direct USB?
- **Cross-platform**: Works on macOS without ALSA
- **Control**: Full access to device capabilities
- **No dependencies**: No need for kernel drivers (beyond USB stack)
- **Flexible**: Can implement custom features

---

## Challenges & Solutions

### Challenge 1: USB Protocol Documentation
**Solution**: Reverse engineer from Linux kernel driver source code. The driver is open source and well-documented.

### Challenge 2: Level Meter Performance
**Solution**: Use async polling with tokio, update UI at 30-60 Hz max. Use Slint's reactive properties for smooth updates.

### Challenge 3: macOS Permissions
**Solution**:
- USB: Info.plist with USB device entries
- Keyboard: Request accessibility permissions with clear dialog
- Provide fallback mode without hotkey support

### Challenge 4: Multiple Device Support
**Solution**: Device registry pattern, spawn separate handlers per device, UI shows all connected devices.

### Challenge 5: Firmware Updates
**Solution**: Parse firmware files, verify checksums, implement sector-by-sector write with progress reporting. Use protocols from existing C code.

---

## Testing Strategy

### Unit Tests
- Protocol encoding/decoding
- Volume/dB conversions
- Routing matrix logic

### Integration Tests
- USB device mock for protocol testing
- Configuration save/load

### Hardware Testing
- Test with actual Focusrite devices
- Verify against Windows Control software behavior
- Test hotplug scenarios
- Test firmware updates (carefully!)

---

## Success Criteria

1. **Functional Parity**: Match all features of the C/GTK4 version
2. **Cross-Platform**: Works reliably on macOS and Linux
3. **Keyboard Integration**: Volume keys control Focusrite interface
4. **Performance**: UI responsive, level meters smooth
5. **Stability**: No crashes, proper error handling
6. **UX**: Intuitive, polished, professional appearance

---

## Timeline Estimate

- **Phase 1**: 1 day
- **Phase 2**: 3-5 days
- **Phase 3**: 1-2 weeks (protocol research intensive)
- **Phase 4**: 1 week
- **Phase 5**: 1 week
- **Phase 6**: 2 weeks
- **Phase 7**: 2-3 weeks (multiple device support)
- **Phase 8**: 1 week
- **Phase 9**: 1 week

**Total**: ~2-3 months for full implementation

---

## Next Steps

1. Initialize Cargo workspace
2. Add Slint UI skeleton
3. Implement USB device detection
4. Start Gen 3 protocol implementation
5. Create basic UI with volume control
6. Test on actual hardware

Let's build something awesome! 🎵🎛️
