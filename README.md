<h1 align="center">CyberSettings</h1>

**CyberSettings** is a Rust command-line utility and keyboard-driven terminal UI for controlling common desktop settings. The current functional backend targets Linux systems using standard desktop utilities.

- **Package:** `cyber-settings`
- **Executable:** `cybersettings`
- **Version:** `0.1.0`
- **Rust edition:** 2024
- **License:** MIT

> Platform honesty: Linux is the implemented backend. macOS and Windows modules are placeholders that return an explicit unsupported message. The CI workflow compiles the code on all three operating systems; that is not a claim that the settings integrations are implemented there.

## Contents

- [Features](#features)
- [Screenshot](#screenshot)
- [Platform support](#platform-support)
- [Requirements](#requirements)
- [Build from source](#build-from-source)
- [Install](#install)
- [Uninstall](#uninstall)
- [CLI reference](#cli-reference)
- [TUI controls](#tui-controls)
- [GitHub Actions](#github-actions)
- [Troubleshooting](#troubleshooting)
- [Safety and privacy](#safety-and-privacy)
- [Project layout](#project-layout)
- [Roadmap](#roadmap)

## Features






## Screenshot (using <a href="https://github.com/zaynkhan100110/CyberShot">CyberShot</a>)


### Main Interface

![Main Interface](https://github.com/spoilpetal/Picture_Vault/blob/main/CyberSettings/CyberSettings_0.1.0/main_interface.png)

## Wifi SSID List (when enabled)

![Wifi List](https://github.com/spoilpetal/Picture_Vault/blob/main/CyberSettings/CyberSettings_0.1.0/wifi_scnanned.png)

### Wifi (when disable) 

![Wifi Disable](https://github.com/spoilpetal/Picture_Vault/blob/main/CyberSettings/CyberSettings_0.1.0/wifi_disable.png)  
 
### Bluetooth scaning

![Bluetooth scaning](https://github.com/spoilpetal/Picture_Vault/blob/main/CyberSettings/CyberSettings_0.1.0/bluetooth_scaning.png)

### Bluetooth (when enable unsure your system have bluetooth device/chip !) 

![Bluetooth enable](https://github.com/spoilpetal/Picture_Vault/blob/main/CyberSettings/CyberSettings_0.1.0/bluetooth_device_scaned.png)


### Bluetooth (when disable)

![Bluetooth disable](https://github.com/spoilpetal/Picture_Vault/blob/main/CyberSettings/CyberSettings_0.1.0/bluetooth_disable.png)
 


### Linux controls

| Setting | Current implementation | Operations |
|---|---|---|
| Brightness | `brightnessctl` | status, set percentage, up, down |
| Volume | PipeWire/WirePlumber `wpctl` | status, set percentage, up, down, mute, unmute, toggle |
| Microphone | PipeWire/WirePlumber `wpctl` | status, mute, unmute, toggle |
| Wi-Fi | NetworkManager `nmcli` | status, radio on/off, scan/list networks, interactive password connection |
| Bluetooth | BlueZ `bluetoothctl` | status, power on/off, scan/list devices, connect |

### Interface

- Launches the TUI when run without arguments.
- CLI flags and descriptive subcommands.
- Keyboard-only TUI; mouse interaction is not required.
- Live status refresh on the main TUI screen.
- Brightness and volume adjustments are clamped to 0–100%.
- Wi-Fi secured-network connection temporarily restores a normal terminal so `nmcli --ask` can request credentials.
- Bluetooth and Wi-Fi radio enable/disable controls are available on the main screen when the relevant row is selected, and on their device screens.
- The application does not automatically elevate privileges.

## Platform support

| Platform | Status | Notes |
|---|---|---|
| Linux | Implemented | Uses the tools listed below. Hardware, desktop session and permissions affect availability. |
| macOS | Not implemented | Backend returns an explicit unsupported message. |
| Windows | Not implemented | Backend returns an explicit unsupported message. |

The repository includes macOS and Windows backend placeholders and cross-platform compilation checks. Actual native setting control, device discovery and integration testing on those platforms remain future work.

## Requirements

### Rust build requirements

- Rust stable with Cargo.
- Rust 2024 edition support (Rust 1.85 or newer).

### Linux runtime tools

| Command | Package commonly providing it | Used for |
|---|---|---|
| `brightnessctl` | `brightnessctl` | Display brightness |
| `wpctl` | `wireplumber` | PipeWire audio |
| `nmcli` | `networkmanager` | Wi-Fi |
| `bluetoothctl` | `bluez` | Bluetooth |

Arch Linux:

```bash
sudo pacman -S --needed brightnessctl wireplumber networkmanager bluez
```

Debian/Ubuntu package names vary by release; commonly:

```bash
sudo apt install brightnessctl wireplumber network-manager bluez
```

Fedora package names commonly:

```bash
sudo dnf install brightnessctl wireplumber NetworkManager bluez
```

Ensure the corresponding services are enabled and running. For example, NetworkManager must manage the Wi-Fi adapter and the PipeWire/WirePlumber session must be active for audio operations.

### Permissions

CyberSettings does not call `sudo` itself. Backlight writes may require suitable device permissions or a correctly configured `brightnessctl` udev/ACL setup. NetworkManager and BlueZ permissions depend on the system's policy and active user session. Do not run the whole TUI as root just to work around an individual permission problem.

## Build from source

Clone the repository, then:

```bash
cargo fmt --all -- --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --release --locked
```

The release binary is:

```text
target/release/cybersettings
```

Run the TUI:

```bash
cargo run
# or
cargo run -- --tui
```

Check the version and CLI options:

```bash
cargo run -- --version
cargo run -- --help
```

## Install

Build the release binary first:

```bash
cargo build --release --locked
```

Then run the installer:

```bash
./install.sh
```

The installer interactively asks for:

1. **Scope:** current-user install (`~/.local/bin`, recommended) or system-wide install (`/usr/local/bin`, requires root).
2. **Command names:** `cybersettings` only, `cse` only, or both.
3. If a selected command name already exists in the chosen destination, it displays the existing path and asks before replacing it. Declining leaves that name untouched.

The installer copies the release binary into a versioned application directory (`~/.local/lib/cybersettings/0.1.0` for a user install or `/usr/local/lib/cybersettings/0.1.0` for a system install), then creates command symlinks in the selected `bin` directory. The installed command therefore does not depend on keeping this source checkout or its `target` directory in place.

Before installing each command name, the installer checks whether it is already resolvable in `PATH` and displays the discovered path. It asks whether to continue. Existing destination entries are shown and are never overwritten without confirmation; destination directories are always left untouched.

For system-wide installation:

```bash
sudo ./install.sh
```

If `~/.local/bin` is not in your `PATH`, the installer prints the line to add to your shell profile. Open a new terminal or reload that profile afterward.

## Uninstall

Run:

```bash
./uninstall.sh
```

The uninstaller asks whether to inspect the current-user or system-wide location. System-wide removal requires:

```bash
sudo ./uninstall.sh
```

It displays found command names and asks which ones to remove. It displays detected command names and asks which to remove. It removes only symlinks in the selected install directory, then offers to remove the versioned binary only when no remaining selected command link points to it. It asks before deleting the install manifest. It does not remove your source checkout, Cargo registry/cache, `target` build directory, other Cyber* projects, shell configuration, SSH keys, GPG keys, or personal files.

If you installed both a user-local and system-wide copy, run the uninstaller once for each scope. The script intentionally does not search the entire filesystem or delete arbitrary files named `cybersettings`.

## CLI reference

Run `cybersettings --help` for the authoritative options from the compiled binary.

### TUI

```bash
cybersettings
cybersettings --tui
```

### Overall status

```bash
cybersettings --status
cybersettings status
```

### Short options

```bash
cybersettings -b                 # brightness status
cybersettings -b 65              # set brightness to 65%
cybersettings -v                 # volume status
cybersettings -v mute            # mute volume
cybersettings -m toggle          # toggle microphone mute
cybersettings -w                 # Wi-Fi status
cybersettings -w off             # disable Wi-Fi radio
cybersettings -t on              # enable Bluetooth
cybersettings -s                 # overall status
```

### Descriptive subcommands

```bash
cybersettings brightness status
cybersettings brightness set 65
cybersettings brightness up
cybersettings brightness down

cybersettings volume status
cybersettings volume set 40
cybersettings volume up
cybersettings volume down
cybersettings volume mute
cybersettings volume unmute
cybersettings volume toggle

cybersettings microphone status
cybersettings microphone mute
cybersettings microphone unmute
cybersettings microphone toggle

cybersettings wifi status
cybersettings wifi on
cybersettings wifi off
cybersettings wifi networks
cybersettings wifi connect "SSID"

cybersettings bluetooth status
cybersettings bluetooth on
cybersettings bluetooth off
cybersettings bluetooth scan
cybersettings bluetooth devices
cybersettings bluetooth connect "AA:BB:CC:DD:EE:FF"
```

Wi-Fi connection through the TUI uses an interactive terminal password prompt. CLI `wifi connect` currently also uses `nmcli --ask`, which requires an interactive terminal.

## TUI controls

### Main screen

| Key | Action |
|---|---|
| Up / Down (also `k` / `j`) | Select a setting |
| Left / Right | Decrease/increase brightness or volume |
| Home / End | Set brightness or volume to 0% / 100% |
| Enter | Open Wi-Fi or Bluetooth device screen |
| `S` | Disable selected Wi-Fi/Bluetooth radio |
| `E` | Enable selected Wi-Fi/Bluetooth radio |
| `M` | Toggle microphone mute |
| `V` | Toggle volume mute |
| Space | Toggle microphone mute when selected |
| `R` | Refresh status |
| `?` | Show shortcut help |
| Esc / `Q` | Exit |

### Wi-Fi and Bluetooth screens

| Key | Action |
|---|---|
| Up / Down | Select network/device |
| Enter | Connect to selected network/device |
| `R` | Rescan/refresh list |
| `S` | Disable radio |
| `E` | Enable radio |
| Esc | Return to main screen |
| `Q` | Quit |

For secured Wi-Fi networks, CyberSettings leaves the alternate-screen TUI and disables raw keyboard mode before starting `nmcli --ask`. This is necessary for NetworkManager's password prompt to receive normal terminal input. After the connection attempt, press Enter when prompted to return to the TUI.

## GitHub Actions

`.github/workflows/ci.yml` runs on pushes to `main`/`master`, pull requests and manual dispatch. It performs:

- `cargo fmt --all -- --check`
- `cargo check --locked --all-targets`
- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo test --locked --all-targets`
- Compile checks on Ubuntu, macOS and Windows runners

A successful macOS/Windows compile does **not** mean those platform integrations work; see [Platform support](#platform-support).

## Troubleshooting

### `command not found`

Check that the selected installation directory is in `PATH`:

```bash
printf '%s\n' "$PATH"
ls -l ~/.local/bin/cybersettings ~/.local/bin/cse 2>/dev/null
```

### Brightness is unavailable or permission denied

```bash
brightnessctl
brightnessctl get
brightnessctl max
```

Check that the device exists and that your user session has permission to adjust it.

### Volume or microphone is unavailable

```bash
wpctl status
wpctl get-volume @DEFAULT_AUDIO_SINK@
wpctl get-volume @DEFAULT_AUDIO_SOURCE@
```

Make sure PipeWire and WirePlumber are running and that default sink/source devices exist.

### Wi-Fi scan or connection fails

```bash
nmcli radio wifi
nmcli device status
nmcli device wifi list
```

Make sure NetworkManager is active and manages the adapter. A secured network needs its correct password; `nmcli --ask` prompts interactively. Repeated password requests can indicate an incorrect password or authentication/connection-agent issue.

### Bluetooth fails

```bash
bluetoothctl show
bluetoothctl devices
rfkill list
```

Check that the adapter is present, powered and not blocked. BlueZ's scan may take several seconds.

### TUI terminal is not restored

CyberSettings uses terminal raw mode and an alternate screen. If a terminal is interrupted unexpectedly, run:

```bash
stty sane
reset
```

## Safety and privacy

- No telemetry or account sign-in is implemented.
- No settings command is automatically run with elevated privileges.
- Wi-Fi passwords are requested by NetworkManager interactively; CyberSettings does not put the password in its own command-line arguments.
- The installer asks before replacing an existing command path.
- The uninstaller removes only selected command symlinks and its own manifest; it does not perform broad filesystem deletion.
- Review shell scripts before running them, especially when using `sudo`.

## Project layout

```text
CyberSettings/
├── .github/
│   └── workflows/
│       └── ci.yml
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── platform/
│   │   ├── mod.rs
│   │   ├── linux.rs
│   │   ├── macos.rs
│   │   └── windows.rs
│   └── tui/
│       └── mod.rs
├── install.sh
├── uninstall.sh
├── Cargo.toml
├── Cargo.lock
├── LICENSE
└── README.md
```

## Roadmap

- Native macOS settings backend.
- Native Windows settings backend.
- Night-light / color-temperature controls.
- Keyboard backlight.
- Battery and power-profile information/control.
- Audio input/output device selection.
- Automated integration tests with mocked system commands.
- More robust Wi-Fi SSID parsing (including unusual SSIDs containing delimiters).

## License

MIT. See [LICENSE](LICENSE).
