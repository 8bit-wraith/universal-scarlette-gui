#!/bin/bash
# Run Scarlett GUI with debug logging

echo "🎵 Starting Scarlett GUI with debug logging..."
echo ""
echo "This will show detailed USB device detection information."
echo "If your device doesn't show up, check the output below."
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run with debug logging for scarlett modules only
RUST_LOG=scarlett_usb=debug,scarlett_gui=info,info cargo run --release -p scarlett-gui
