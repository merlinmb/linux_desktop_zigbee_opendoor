# Linux Mint Zigbee Door/Window Sensor Monitor Overlay

A lightweight desktop overlay application for Linux Mint Cinnamon that monitors Zigbee contact sensors (doors, windows, gates) via MQTT. Displays animated "RoboEyes" when all sensors are closed, and a scrolling list of open contacts when any door is open.

## Features

- **MQTT Subscription** to Zigbee2MQTT topics for any number of contact sensors
- **Transparent Desktop Overlay** positioned at top-left (or configurable position)
- **Two Display Modes:**
  - Animated RoboEyes when all contacts are closed
  - Scrolling list of open contacts with last-seen timestamp
- **Always-On-Top Window** stays visible over other applications
- **Dark Theme UI** with minimal, non-intrusive design
- **Settings UI** for:
  - MQTT broker configuration
  - Adding/removing sensor subscriptions
  - Brightness and scroll speed adjustment
  - Display options
- **Persistent Configuration** stored in `~/.config/opendoor-monitor/config.toml`
- **Keyboard Shortcut** (Ctrl+Comma) to toggle settings

## Requirements

### On Linux Mint (Target Platform)
- Linux Mint 20.x or later (Cinnamon desktop environment)
- X11 session (Wayland support pending)
- MQTT broker (e.g., Mosquitto) with Zigbee2MQTT publishing contact sensor payloads

### For Development
- Node.js 18+ and npm
- Rust 1.70+ (for building Tauri backend)
- Tauri CLI (installed via npm)

## Installation

### From AppImage (Recommended)
Download the latest `.AppImage` from GitHub releases:

```bash
wget https://github.com/merlinmb/linux_desktop_zigbee_opendoor/releases/download/v1.0.0/opendoor-monitor-1.0.0.AppImage
chmod +x opendoor-monitor-1.0.0.AppImage
./opendoor-monitor-1.0.0.AppImage
```

The app runs as a system overlay and can be launched from your application menu or via command line.

### From Source (Development)

```bash
git clone https://github.com/merlinmb/linux_desktop_zigbee_opendoor
cd linux_desktop_zigbee_opendoor

# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Build production AppImage
npm run tauri:build
```

## First-Run Setup

1. **Launch the app** — it opens as a small transparent overlay at the top-left
2. **Press Ctrl+Comma** to open Settings
3. **Configure MQTT:**
   - Enter your broker IP/hostname
   - Set port (default 1883)
   - Set a unique client name
4. **Add Subscriptions:**
   - Enter Zigbee2MQTT topic path (e.g., `zigbee2mqtt/mcmdhome/Front Door`)
   - Enter friendly name (e.g., `Front door`)
   - Click "Add"
5. **Adjust Display Settings:**
   - Brightness (0-255)
   - Scroll interval between contacts (ms)
   - Click "Save"

## Configuration

Configuration is stored in TOML format at:
```
~/.config/opendoor-monitor/config.toml
```

### Example Config
```toml
[mqtt]
broker = "192.168.1.55"
port = 1883
client_name = "opendoor_monitor_linux"

[display]
brightness = 100
flip_screen = false
scroll_interval_ms = 1750

[window]
x = 0
y = 0
always_on_top = true
transparency = 255

[subscriptions]
"zigbee2mqtt/mcmdhome/Front Door Contact" = "Front door"
"zigbee2mqtt/mcmdhome/Garage Door Contact" = "Garage"
"zigbee2mqtt/mcmdhome/Utility Door Contact" = "Utility room"
```

## Project Structure

```
├── src/
│   ├── main.tsx                  # React entry point
│   ├── App.tsx                   # Main app component
│   ├── components/
│   │   ├── RoboEyes.tsx          # Animated eyes (Canvas-based)
│   │   ├── ContactsList.tsx      # Scrolling open contacts
│   │   ├── StatusBar.tsx         # MQTT status + clock
│   │   └── SettingsModal.tsx     # Configuration UI
│   ├── hooks/
│   ├── lib/
│   │   ├── api.ts               # Tauri command wrappers
│   │   └── types.ts             # TypeScript interfaces
│   └── styles/
│       └── App.css              # Dark theme styling
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # Tauri app entry
│   │   ├── config.rs            # Config loading/saving
│   │   ├── state.rs             # Global app state
│   │   ├── mqtt.rs              # MQTT connection manager
│   │   ├── window.rs            # X11 window hints
│   │   └── commands/            # Tauri command handlers
│   ├── tauri.conf.json          # Window configuration
│   └── Cargo.toml               # Rust dependencies
├── package.json
└── README.md
```

## Usage

### Display States

**RoboEyes (All Doors Closed)**
- Animated eyes with random blinking and movement
- Indicates system is monitoring and all doors are secure

**Contact List (Any Door Open)**
- Shows count of open contacts
- Scrolls through each open contact with name and last-seen time
- Red indicator for open status

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+Comma | Toggle Settings modal |

### Window Management

The overlay is always on top and positioned at the desktop top-left. To move it:
1. Edit `~/.config/opendoor-monitor/config.toml`
2. Change `[window]` section `x` and `y` values
3. Restart the app

## MQTT Integration

The app expects Zigbee2MQTT format messages:

```json
{
  "contact": true,
  "battery": 98,
  "last_seen": "2025-08-26T14:32:00Z"
}
```

- `contact: true` = door/window is closed
- `contact: false` = door/window is open

## Build & Distribution

### Build AppImage (Linux)
```bash
npm run tauri:build
# Output: src-tauri/target/release/bundle/appimage/
```

### Build Deb Package (Optional)
```bash
npm run tauri:build -- --target deb
```

### Release to GitHub
```bash
git tag v1.0.0
git push origin v1.0.0
# GitHub Actions automatically builds and releases AppImage
```

## Troubleshooting

### App doesn't appear on screen
- Check X11 is running: `echo $XDG_SESSION_TYPE` should output `x11`
- If using Wayland, switch to X11 at login screen (Cinnamon settings)

### MQTT connection fails
- Verify broker is running and accessible: `telnet <broker-ip> 1883`
- Check firewall allows port 1883
- Verify topic names in subscriptions match Zigbee2MQTT output

### Config file corrupted
- Delete `~/.config/opendoor-monitor/config.toml` to reset to defaults

## Development

### Hot Reload (Dev Mode)
```bash
npm run tauri:dev
# Automatically reloads frontend on file changes
# Restart required for Rust backend changes
```

### Build Rust Only
```bash
cd src-tauri
cargo build --release
```

### Run Tests
```bash
cargo test -p opendoor-monitor
```

## Architecture

**Frontend:** React 18 + TypeScript + CSS
- Components for RoboEyes animation, contact list, settings
- Tauri API wrappers for backend communication
- Dark theme optimized for desktop overlay

**Backend:** Rust + Tauri 1.5
- Configuration management (TOML format)
- Global app state with MQTT connection status
- Tauri commands expose backend functionality to frontend
- X11 window hints for dock-style overlay

**MQTT:** rumqttc async client
- Non-blocking subscriptions
- Async/await tokio runtime integration
- Placeholder implementation ready for full event handling

## License

MIT

## Credits

Original ESP32 firmware concept adapted for Linux desktop environment.
RoboEyes animation inspired by Adafruit's RoboEyes library.
