use super::Setting;
use anyhow::{Context, Result, anyhow, bail};
use std::process::Command;

fn output(program: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(program).args(args).output().with_context(|| {
        format!("Cannot start '{program}'. Install it and make sure it is in PATH.")
    })?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();

        bail!(
            "{program} failed: {}",
            if err.is_empty() {
                "operation unsuccessful"
            } else {
                &err
            }
        );
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn brightness_percent() -> Result<u8> {
    let cur: f64 = output("brightnessctl", &["get"])?
        .parse()
        .context("Invalid brightness value")?;

    let max: f64 = output("brightnessctl", &["max"])?
        .parse()
        .context("Invalid brightness maximum")?;

    if max <= 0.0 {
        bail!("Brightness device reported an invalid maximum.");
    }

    Ok(((cur / max * 100.0).round() as i32).clamp(0, 100) as u8)
}

fn audio_percent(target: &str) -> Result<(u8, bool)> {
    let text = output("wpctl", &["get-volume", target])?;

    let raw = text
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow!("Could not parse wpctl volume output: {text}"))?;

    let value: f64 = raw.parse().context("Could not parse audio volume")?;

    Ok((
        ((value * 100.0).round() as i32).clamp(0, 100) as u8,
        text.contains("[MUTED]"),
    ))
}

fn audio_status(target: &str, label: &str) -> Result<String> {
    let (percentage, muted) = audio_percent(target)?;

    Ok(format!(
        "{label}: {percentage}% ({})",
        if muted { "muted" } else { "unmuted" }
    ))
}

fn wifi_status() -> Result<String> {
    let radio = output("nmcli", &["radio", "wifi"])?;

    let devices = output(
        "nmcli",
        &[
            "-t",
            "-f",
            "DEVICE,TYPE,STATE,CONNECTION",
            "device",
            "status",
        ],
    )?;

    let active = devices
        .lines()
        .find(|line| line.contains(":wifi:connected:"))
        .and_then(|line| line.splitn(4, ':').nth(3))
        .filter(|connection| !connection.is_empty())
        .unwrap_or("not connected");

    Ok(format!("Wi-Fi: radio {radio}; {active}"))
}

fn bluetooth_status() -> Result<String> {
    let text = output("bluetoothctl", &["show"])?;

    let powered = text
        .lines()
        .find(|line| line.trim_start().starts_with("Powered:"))
        .map(|line| line.split(':').nth(1).unwrap_or("").trim())
        .unwrap_or("unknown");

    Ok(format!("Bluetooth: {powered}"))
}

pub fn status(setting: Setting) -> String {
    let result = match setting {
        Setting::Brightness => brightness_percent().map(|p| format!("Brightness: {p}%")),
        Setting::Volume => audio_status("@DEFAULT_AUDIO_SINK@", "Volume"),
        Setting::Microphone => audio_status("@DEFAULT_AUDIO_SOURCE@", "Microphone"),
        Setting::Wifi => wifi_status(),
        Setting::Bluetooth => bluetooth_status(),
    };

    result.unwrap_or_else(|error| format!("{setting}: unavailable ({error})"))
}

pub fn perform(setting: Setting, action: &str, value: Option<&str>) -> Result<String> {
    let action = action.to_ascii_lowercase();

    match setting {
        Setting::Brightness => {
            let target = match action.as_str() {
                "status" => return Ok(status(setting)),
                "up" => (brightness_percent()? as i16 + 5).clamp(0, 100) as u8,
                "down" => (brightness_percent()? as i16 - 5).clamp(0, 100) as u8,
                "set" => parse_percent(value, "brightness")?,
                _ => parse_percent(Some(&action), "brightness")?,
            };

            output("brightnessctl", &["set", &format!("{target}%")])?;

            Ok(status(setting))
        }

        Setting::Volume | Setting::Microphone => {
            let (target, label) = if setting == Setting::Volume {
                ("@DEFAULT_AUDIO_SINK@", "Volume")
            } else {
                ("@DEFAULT_AUDIO_SOURCE@", "Microphone")
            };

            match action.as_str() {
                "status" => return audio_status(target, label),

                "up" | "down" => {
                    let (current, _) = audio_percent(target)?;

                    let next = if action == "up" {
                        (current as i16 + 5).clamp(0, 100)
                    } else {
                        (current as i16 - 5).clamp(0, 100)
                    };

                    output(
                        "wpctl",
                        &["set-volume", target, &format!("{:.2}", next as f64 / 100.0)],
                    )?;
                }

                "mute" => {
                    output("wpctl", &["set-mute", target, "1"])?;
                }

                "unmute" => {
                    output("wpctl", &["set-mute", target, "0"])?;
                }

                "toggle" => {
                    output("wpctl", &["set-mute", target, "toggle"])?;
                }

                "set" => {
                    let percentage = parse_percent(value, "volume")?;

                    output(
                        "wpctl",
                        &[
                            "set-volume",
                            target,
                            &format!("{:.2}", percentage as f64 / 100.0),
                        ],
                    )?;
                }

                _ => {
                    let percentage = parse_percent(Some(&action), "volume")?;

                    output(
                        "wpctl",
                        &[
                            "set-volume",
                            target,
                            &format!("{:.2}", percentage as f64 / 100.0),
                        ],
                    )?;
                }
            }

            audio_status(target, label)
        }

        Setting::Wifi => match action.as_str() {
            "status" => wifi_status(),

            "on" => {
                output("nmcli", &["radio", "wifi", "on"])?;
                wifi_status()
            }

            "off" => {
                output("nmcli", &["radio", "wifi", "off"])?;
                wifi_status()
            }

            "networks" => Ok(wifi_networks()?.join("\n")),

            "connect" => connect_wifi(
                value.ok_or_else(|| anyhow!("Usage: cybersettings wifi connect <SSID>"))?,
            ),

            _ => bail!("Use Wi-Fi action: status, on, off, networks, connect <SSID>."),
        },

        Setting::Bluetooth => match action.as_str() {
            "status" => bluetooth_status(),

            "on" => {
                output("bluetoothctl", &["power", "on"])?;
                bluetooth_status()
            }

            "off" => {
                output("bluetoothctl", &["power", "off"])?;
                bluetooth_status()
            }

            "scan" | "devices" => Ok(bluetooth_devices(action == "scan")?.join("\n")),

            "connect" => connect_bluetooth(
                value.ok_or_else(|| anyhow!("Usage: cybersettings bluetooth connect <ADDRESS>"))?,
            ),

            _ => bail!("Use Bluetooth action: status, on, off, scan, devices, connect <ADDRESS>."),
        },
    }
}

fn parse_percent(value: Option<&str>, name: &str) -> Result<u8> {
    let number: u8 = value
        .ok_or_else(|| anyhow!("A percentage is required."))?
        .parse()
        .with_context(|| format!("{name} must be a whole number from 0 to 100"))?;

    if number > 100 {
        bail!("{name} must be between 0 and 100.");
    }

    Ok(number)
}

pub fn wifi_networks() -> Result<Vec<String>> {
    let raw = output(
        "nmcli",
        &[
            "-t",
            "--escape",
            "no",
            "-f",
            "SSID,SECURITY,SIGNAL",
            "device",
            "wifi",
            "list",
            "--rescan",
            "yes",
        ],
    )?;

    let mut rows = raw
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, ':');

            let ssid = parts.next()?.trim();

            if ssid.is_empty() {
                return None;
            }

            let security = parts.next().unwrap_or("open").trim();
            let signal = parts.next().unwrap_or("?").trim();

            Some(format!(
                "{ssid}  |  {}  |  {signal}% signal",
                if security.is_empty() {
                    "Open"
                } else {
                    security
                }
            ))
        })
        .collect::<Vec<_>>();

    rows.sort();
    rows.dedup();

    if rows.is_empty() {
        rows.push("No visible networks found. Press R to scan again.".into());
    }

    Ok(rows)
}

/// Connect to Wi-Fi using NetworkManager's interactive password prompt.
///
/// This function inherits stdin/stdout/stderr so nmcli --ask can
/// communicate with the user through the terminal.
pub fn connect_wifi(ssid: &str) -> Result<String> {
    if ssid.trim().is_empty() {
        bail!("SSID cannot be empty.");
    }

    let result = Command::new("nmcli")
        .args(["--ask", "device", "wifi", "connect", ssid])
        .status()
        .context("Could not start nmcli. Is NetworkManager installed?")?;

    if !result.success() {
        bail!("NetworkManager could not connect to Wi-Fi network '{ssid}'.");
    }

    Ok(format!("Connected to Wi-Fi network: {ssid}"))
}

pub fn bluetooth_devices(scan: bool) -> Result<Vec<String>> {
    if scan {
        let _ = Command::new("bluetoothctl")
            .args(["scan", "on"])
            .output()
            .context("Cannot start bluetoothctl. Install bluez.")?;

        std::thread::sleep(std::time::Duration::from_secs(4));
    }

    let raw = output("bluetoothctl", &["devices"])?;

    let mut rows = raw.lines().map(str::to_string).collect::<Vec<_>>();

    if rows.is_empty() {
        rows.push("No known devices. Press R to scan again.".into());
    }

    Ok(rows)
}

pub fn connect_bluetooth(address: &str) -> Result<String> {
    if address.trim().is_empty() {
        bail!("Bluetooth address is required.");
    }

    output("bluetoothctl", &["connect", address])?;

    Ok(format!("Connected to Bluetooth device {address}"))
}
