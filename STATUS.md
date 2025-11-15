# Scarlett Control - Rust Edition Status

## 🎉 Current Status: Device Detection Working!

Your **Scarlett 18i20 Gen 4** is now being detected and displays in the UI!

### ✅ What's Working Right Now

1. **Device Detection** - Your 18i20 Gen 4 shows up in the app
2. **Dark UI Theme** - Professional audio app aesthetic
3. **Hotplug Monitoring** - Detects when you connect/disconnect
4. **Device Information** - Shows model name and serial number
5. **Protocol Foundations** - Both Gen 3 and Gen 4 protocol structures are implemented

### 🚧 What's In Progress

**USB Communication Layer** - The actual USB communication to control the device

Currently, the protocol structures are defined but the actual USB transfers need to be implemented using the `nusb` library. This is the critical next step to make the app functional.

### 🎯 Immediate Next Steps (in order)

#### 1. Implement USB Control Transfers
**File**: `crates/scarlett-usb/src/gen4_fcp.rs`

Need to replace the placeholder USB functions with actual `nusb` calls:
```rust
// Current: Placeholder
fn control_write(&self, ...) -> Result<()> {
    // TODO: Implement
}

// Needed: Actual USB
fn control_write(&self, data: &[u8]) -> Result<()> {
    self.device.control_out(
        request_type,
        request,
        value,
        index,
        data,
        timeout
    )?;
    Ok(())
}
```

#### 2. Open USB Device Handle
**File**: `crates/scarlett-usb/src/device_impl.rs`

When opening a device, we need to:
1. Find the USB device by VID/PID
2. Open it with `nusb`
3. Claim the audio control interface
4. Store the device handle for protocol operations

#### 3. Implement Basic Volume Control
**Target**: Monitor output volume for keyboard integration

This is the highest priority feature for your use case!

Steps:
1. Figure out the FCP command for getting/setting volume
2. Implement in `gen4_fcp.rs`
3. Expose via `Device` trait
4. Connect to keyboard hotkey system

#### 4. Implement Level Meters
Real-time VU meters showing input/output levels

#### 5. Implement Routing Matrix
Configure audio routing between inputs/outputs

---

## 📋 Implementation Details

### Gen 4 Protocol (Your 18i20)

**Protocol**: FCP (Focusrite Control Protocol)

**What's Implemented**:
- ✅ FCP message structures (header, version, error, progress, success)
- ✅ Error code handling
- ✅ Message serialization/deserialization
- ✅ Type-safe response parsing

**What's Needed**:
- ❌ Actual USB control transfers (via nusb)
- ❌ Device command mapping (what commands control what?)
- ❌ Configuration state management

**Challenges**:
- Gen 4 "big" devices (18i20, 18i16, 16i16) use a complex protocol
- On Linux, this goes through a Unix socket to `fcp-server` daemon
- On macOS, we need **direct USB** communication
- Limited documentation - need to reverse engineer from C code

### Gen 3 Protocol (Your other 18i20)

**Protocol**: Scarlett2 USB Protocol

**What's Implemented**:
- ✅ Protocol command structures
- ✅ Meter level handling
- ✅ Mixer volume conversion (dB ↔ raw value)
- ✅ USB control transfer parameters

**What's Needed**:
- ❌ USB communication implementation
- ❌ Command/response handling
- ❌ Integration with device layer

**Advantages**:
- Simpler than Gen 4 FCP
- Well-documented in Linux kernel driver
- Good for learning and testing before tackling Gen 4

---

## 🔬 Technical Challenges

### Challenge 1: USB Device Access on macOS

**Problem**: macOS has strict USB device access policies

**Solution**:
1. Add USB device entitlements to Info.plist (when we create .app bundle)
2. Use `nusb` which handles macOS permissions correctly
3. May need to request user permission on first run

### Challenge 2: FCP Protocol Documentation

**Problem**: Gen 4 FCP protocol is not fully documented

**Strategy**:
1. Study the Linux C implementation (`fcp-shared.c`, `fcp-socket.c`)
2. Use USB packet sniffing (Wireshark) on Linux to observe traffic
3. Reverse engineer command structures from kernel driver
4. Test incrementally with real hardware

### Challenge 3: Concurrent USB Access

**Problem**: Focusrite's official software might conflict

**Solution**:
- On macOS: Close Focusrite Control before using our app
- Our app uses exclusive device access
- Eventually: Detect conflicts and warn user

### Challenge 4: Level Meter Performance

**Problem**: Need to poll meters at ~30-60 Hz for smooth VU meters

**Solution**:
- Use async polling with Tokio
- Batch meter reads
- Update UI via Slint reactive properties
- Limit update rate to match display refresh

---

## 🎹 Keyboard Volume Control Implementation

This feature makes your Mac's volume keys control the Focusrite!

### Architecture

```
Keyboard → HotkeyManager → Volume Command → Device Protocol → USB → Hardware
   F11/F12   (macOS CGEventTap)  (+/-/mute)   (FCP/Scarlett2)  (nusb)  (18i20)
```

### Steps

1. **Capture Keys** (`scarlett-hotkeys/src/macos.rs`)
   - Use `CGEventTap` to intercept media keys
   - Filter for volume up/down/mute
   - Send commands via channel

2. **Map to Device** (`scarlett-gui/src/main.rs`)
   - Receive volume commands
   - Call device.set_volume() or device.adjust_volume()
   - Update UI to reflect change

3. **Control Hardware** (Protocol layer)
   - Convert dB to device-specific value
   - Send USB command to hardware
   - Read back to confirm

### Configuration

User preferences (in `scarlett-config`):
- Enable/disable keyboard control
- Which output to control (Monitor, Headphones, etc.)
- Volume step size (default: 1 dB)
- Show OSD notification (optional)

---

## 📊 Feature Roadmap

### Phase 1: Basic Control (Current)
- [x] Device detection
- [x] Protocol structures
- [ ] USB communication ← **YOU ARE HERE**
- [ ] Basic volume control
- [ ] Keyboard integration

### Phase 2: Full Interface Control
- [ ] Routing matrix
- [ ] Full mixer (all 25 inputs for 18i20)
- [ ] Hardware settings (sample rate, sync, etc.)
- [ ] Configuration save/load

### Phase 3: Advanced Features
- [ ] Level meters with peak hold
- [ ] Firmware updates
- [ ] Multiple device support
- [ ] Presets/scenes

### Phase 4: Polish
- [ ] macOS app bundle (.app)
- [ ] DMG installer
- [ ] Auto-update system
- [ ] Comprehensive documentation

---

## 🧪 Testing Plan

### With Your Hardware

**Gen 4 (18i20)** - Primary focus
1. Device detection ✅
2. USB communication (next)
3. Volume control
4. Keyboard integration
5. Full routing/mixer

**Gen 3 (18i20)** - Secondary
1. Test simpler protocol first
2. Validate approach works
3. Use learnings for Gen 4

### Development Workflow

1. **Incremental Changes**
   - Implement one feature at a time
   - Test immediately with hardware
   - Commit working code frequently

2. **Debug Logging**
   - Run with `RUST_LOG=debug`
   - Check USB traffic in logs
   - Use `./run-debug.sh` script

3. **Safety**
   - Don't attempt firmware updates until thoroughly tested
   - Save device config before major changes
   - Keep Focusrite Control as backup

---

## 🚀 How to Help Development

1. **Test Device Detection**
   ```bash
   cargo run --release -p scarlett-gui
   ```
   Confirm your device shows up correctly ✅

2. **Provide USB Packet Captures** (Optional)
   If you have Wireshark on macOS:
   - Capture USB traffic while using Focusrite Control
   - Can help reverse engineer commands

3. **Test Features As They're Built**
   - Try each feature on your Gen 4
   - Report what works/doesn't work
   - Suggest UX improvements

4. **Document Your Workflow**
   - How do you currently use your 18i20?
   - What features are most important?
   - What routing/mixer setup do you use?

---

## 💡 Quick Wins We Can Implement Soon

1. **Show More Device Info** in UI
   - Firmware version (if we can read it)
   - Sample rate
   - Sync status

2. **Volume OSD**
   - Show volume level on screen when using keyboard
   - Match macOS style

3. **System Tray Icon**
   - Quick access to app
   - Show connection status
   - Toggle mute quickly

4. **Preset System**
   - Save common routing/mixer configs
   - Quick recall
   - Export/import

---

## 🎵 The Vision

Once complete, you'll have:

✨ **Your Mac's volume keys control your Focusrite** - No more fixed volume!
🎛️ **Full control over routing and mixing** - Better than Focusrite's software
📊 **Smooth, professional level meters** - Real-time visual feedback
🎨 **Beautiful dark UI** - Matches your audio workflow
🚀 **Fast and stable** - Rust means no crashes
🔌 **Works on both your devices** - Gen 3 and Gen 4

---

## 📞 Next Steps - Your Input Needed!

1. **Try the current build** - Does your device show up reliably?

2. **Priority features** - What do you want to work first?
   - Keyboard volume control?
   - Level meters?
   - Routing matrix?
   - Mixer control?

3. **Usage patterns** - How do you use your 18i20?
   - Recording setup?
   - Mixing setup?
   - What outputs do you monitor on?

4. **Testing time** - When can you test as features are built?

---

**Current Commits**:
- Initial Rust rewrite ✅
- Darker UI theme ✅
- 18i20 Gen 4 support ✅
- Protocol foundations ✅

**Next Commit**: USB communication implementation 🚧

Let me know which feature you'd like me to tackle next!
