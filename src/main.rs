mod cli;
mod platform;
mod tui;

use anyhow::{Result, bail};
use clap::Parser;
use cli::{Cli, Command};
use platform::{Setting, overall_status, perform};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let cli = Cli::parse();
    if let Some(command) = cli.command {
        let (setting, action, value) = match command {
            Command::Brightness { action, value } => (Setting::Brightness, action, value),
            Command::Volume { action, value } => (Setting::Volume, action, value),
            Command::Microphone { action, value } => (Setting::Microphone, action, value),
            Command::Wifi { action, value } => (Setting::Wifi, action, value),
            Command::Bluetooth { action, value } => (Setting::Bluetooth, action, value),
            Command::Status => {
                println!("{}", overall_status());
                return Ok(());
            }
            Command::Tui => return tui::run(),
        };
        println!("{}", perform(setting, &action, value.as_deref())?);
        return Ok(());
    }

    let selected = [
        (Setting::Brightness, cli.brightness),
        (Setting::Volume, cli.volume),
        (Setting::Microphone, cli.microphone),
        (Setting::Wifi, cli.wifi),
        (Setting::Bluetooth, cli.bluetooth),
    ];
    for (setting, action) in selected {
        if let Some(action) = action {
            println!("{}", perform(setting, &action, None)?);
            return Ok(());
        }
    }
    if cli.status {
        println!("{}", overall_status());
        return Ok(());
    }
    if cli.tui || std::env::args_os().len() == 1 {
        return tui::run();
    }
    bail!("No action selected. Run 'cybersettings --help' for usage.")
}
