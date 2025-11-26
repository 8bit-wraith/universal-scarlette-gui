# Scarlett 18i20 Gen 4 Support

## Device Information

**Model**: Focusrite Scarlett 18i20 (4th Generation)
**USB IDs**: VID=0x1235, PID=0x821D

## Current Status

✅ **Device Detection**: Working
🚧 **USB Protocol**: In development
⏳ **Full Control**: Not yet implemented

## Known Working

- Device enumeration
- Serial number detection
- Model identification
- Hotplug detection

## To Be Implemented

1. **Gen 4 Protocol** (Uses FCP - Focusrite Control Protocol)
   - Device uses Unix socket communication on Linux
   - macOS will need direct USB implementation
   - Different from Gen 2/3 which use ALSA control transfers

2. **Audio Routing**
   - 18 inputs, 20 outputs
   - Complex routing matrix
   - DSP routing

3. **Mixer**
   - 25 mixer inputs
   - Stereo pair support
   - Per-channel volume/pan/mute

4. **Level Meters**
   - Real-time meter polling
   - Peak hold
   - 38 channels total

5. **Hardware Settings**
   - Sample rate control
   - Sync source selection
   - Phantom power
   - Air mode
   - Pad switches
   - Direct monitoring

## Protocol Notes

The Scarlett 18i20 Gen 4 uses the **FCP (Focusrite Control Protocol)** which is different from earlier generations:

- **Gen 1-3**: Direct ALSA mixer controls or Scarlett2 USB protocol
- **Gen 4 Big** (16i16, 18i16, 18i20): FCP via Unix socket (Linux) or direct USB (macOS)

This means the implementation will need to:
1. Parse the FCP protocol from the C source code
2. Implement direct USB communication for macOS
3. Create protocol translator for cross-platform support

## Testing Needed

Once protocol is implemented:
- [ ] Routing matrix
- [ ] Mixer controls
- [ ] Level meters
- [ ] Hardware settings
- [ ] Configuration save/load
- [ ] Firmware updates

## References

- Linux kernel driver: `sound/usb/mixer_scarlett_gen2.c`
- FCP implementation: Original C code in `fcp-shared.c`
- Device capabilities: See original ALSA GUI `iface-4th-gen-big.md`
