use anyhow::Result;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    Brightness,
    Volume,
    Microphone,
    Wifi,
    Bluetooth,
}

impl fmt::Display for Setting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Brightness => "Brightness",
                Self::Volume => "Volume",
                Self::Microphone => "Microphone",
                Self::Wifi => "Wi-Fi",
                Self::Bluetooth => "Bluetooth",
            }
        )
    }
}

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub fn perform(s: Setting, a: &str, v: Option<&str>) -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        linux::perform(s, a, v)
    }
    #[cfg(target_os = "macos")]
    {
        macos::perform(s, a, v)
    }
    #[cfg(target_os = "windows")]
    {
        windows::perform(s, a, v)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        anyhow::bail!("This operating system is not supported.")
    }
}
pub fn status(s: Setting) -> String {
    #[cfg(target_os = "linux")]
    {
        linux::status(s)
    }
    #[cfg(target_os = "macos")]
    {
        macos::status(s)
    }
    #[cfg(target_os = "windows")]
    {
        windows::status(s)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        format!("{s}: unsupported platform")
    }
}
pub fn overall_status() -> String {
    [
        Setting::Brightness,
        Setting::Volume,
        Setting::Microphone,
        Setting::Wifi,
        Setting::Bluetooth,
    ]
    .iter()
    .map(|s| status(*s))
    .collect::<Vec<_>>()
    .join("\n")
}
pub fn wifi_networks() -> Result<Vec<String>> {
    #[cfg(target_os = "linux")]
    {
        linux::wifi_networks()
    }
    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("Wi-Fi network browsing is currently implemented on Linux only.")
    }
}
pub fn bluetooth_devices(scan: bool) -> Result<Vec<String>> {
    #[cfg(target_os = "linux")]
    {
        linux::bluetooth_devices(scan)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = scan;
        anyhow::bail!("Bluetooth device browsing is currently implemented on Linux only.")
    }
}
pub fn connect_wifi(ssid: &str) -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        linux::connect_wifi(ssid)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = ssid;
        anyhow::bail!("Wi-Fi connection is not implemented on this platform.")
    }
}
pub fn connect_bluetooth(address: &str) -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        linux::connect_bluetooth(address)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = address;
        anyhow::bail!("Bluetooth connection is not implemented on this platform.")
    }
}
