use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "cybersettings",
    version,
    about = "CLI and keyboard-driven TUI for common system settings"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
    /// Brightness: status, up, down, or percentage
    #[arg(short='b', long, num_args=0..=1, default_missing_value="status")]
    pub brightness: Option<String>,
    /// Volume: status, up, down, mute, unmute, or percentage
    #[arg(short='v', long, num_args=0..=1, default_missing_value="status")]
    pub volume: Option<String>,
    /// Microphone: status, mute, unmute, toggle
    #[arg(short='m', long, num_args=0..=1, default_missing_value="status")]
    pub microphone: Option<String>,
    /// Wi-Fi: status, on, off
    #[arg(short='w', long, num_args=0..=1, default_missing_value="status")]
    pub wifi: Option<String>,
    /// Bluetooth: status, on, off
    #[arg(short='t', long, num_args=0..=1, default_missing_value="status")]
    pub bluetooth: Option<String>,
    /// Combined status
    #[arg(short = 's', long)]
    pub status: bool,
    /// Launch keyboard TUI
    #[arg(long)]
    pub tui: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Brightness: status | set 50 | up | down
    Brightness {
        action: String,
        value: Option<String>,
    },
    /// Volume: status | set 40 | up | down | mute | unmute
    Volume {
        action: String,
        value: Option<String>,
    },
    /// Microphone: status | mute | unmute | toggle
    Microphone {
        action: String,
        value: Option<String>,
    },
    /// Wi-Fi: status | on | off | networks | connect <SSID>
    Wifi {
        action: String,
        value: Option<String>,
    },
    /// Bluetooth: status | on | off | scan | devices | connect <ADDRESS>
    Bluetooth {
        action: String,
        value: Option<String>,
    },
    /// Combined status
    Status,
    /// Launch TUI
    Tui,
}
