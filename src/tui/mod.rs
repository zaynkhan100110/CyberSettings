use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use std::{
    io,
    time::{Duration, Instant},
};

use crate::platform::{self, Setting, perform, status};

const ITEMS: [Setting; 5] = [
    Setting::Brightness,
    Setting::Volume,
    Setting::Microphone,
    Setting::Wifi,
    Setting::Bluetooth,
];

const AUTO_REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const KEY_REPEAT_DELAY: Duration = Duration::from_millis(140);

struct Restore;

impl Drop for Restore {
    fn drop(&mut self) {
        let _ = disable_raw_mode();

        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Main,
    Wifi,
    Bluetooth,
}

fn update_setting(setting: Setting, action: &str, values: &mut [String], message: &mut String) {
    match perform(setting, action, None) {
        Ok(result) => {
            *message = result.clone();

            if let Some(index) = ITEMS.iter().position(|item| *item == setting) {
                values[index] = result;
            }
        }

        Err(error) => {
            *message = format!("Operation failed: {error}");
        }
    }
}

fn refresh_values(values: &mut Vec<String>) {
    *values = ITEMS.iter().map(|setting| status(*setting)).collect();
}

/// Temporarily leaves the TUI so nmcli --ask can read a password
/// from a normal terminal, then restores the TUI.
fn connect_wifi_from_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ssid: &str,
) -> Result<String> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    println!();
    println!("Connecting to Wi-Fi network: {ssid}");
    println!("Enter the network password if requested.");
    println!();

    let connection_result = platform::connect_wifi(ssid);

    println!();

    let mut input = String::new();

    println!("Press Enter to return to CyberSettings...");
    let pause_result = io::stdin().read_line(&mut input);

    // Restore the TUI even if the connection or pause had an error.
    let screen_result = execute!(terminal.backend_mut(), EnterAlternateScreen);

    let raw_mode_result = enable_raw_mode();

    screen_result?;
    raw_mode_result?;

    terminal.clear()?;
    terminal.hide_cursor()?;

    pause_result?;

    connection_result
}

pub fn run() -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // Restore the terminal when the TUI exits.
    let _restore = Restore;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut screen = Screen::Main;
    let mut selected = 0usize;

    let mut rows: Vec<String> = Vec::new();
    let mut device_selected = 0usize;

    let mut values = ITEMS
        .iter()
        .map(|setting| status(*setting))
        .collect::<Vec<_>>();

    let mut message = String::from(
        "Up/Down navigate | Left/Right adjust | Enter open | S disable | E enable | Q quit",
    );

    let mut refresh_at = Instant::now();
    let mut last_adjust: Option<Instant> = None;

    loop {
        // Refresh main-screen status periodically.
        if screen == Screen::Main && refresh_at.elapsed() >= AUTO_REFRESH_INTERVAL {
            refresh_values(&mut values);
            refresh_at = Instant::now();
        }

        terminal.draw(|frame| {
            let areas = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(7),
                    Constraint::Length(3),
                ])
                .split(frame.area());

            let heading = match screen {
                Screen::Main => "CyberSettings 0.1.0 - live system controls",
                Screen::Wifi => {
                    "Wi-Fi networks | Enter connect | R rescan | S disable | E enable | Esc back"
                }
                Screen::Bluetooth => {
                    "Bluetooth devices | Enter connect | R scan | S disable | E enable | Esc back"
                }
            };

            frame.render_widget(
                Paragraph::new(heading).block(Block::default().borders(Borders::ALL)),
                areas[0],
            );

            let entries = match screen {
                Screen::Main => ITEMS
                    .iter()
                    .enumerate()
                    .map(|(index, setting)| {
                        let marker = if index == selected { "▶ " } else { "  " };

                        let style = if index == selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };

                        ListItem::new(format!("{marker}{setting:<13} {}", values[index]))
                            .style(style)
                    })
                    .collect::<Vec<_>>(),

                Screen::Wifi | Screen::Bluetooth => rows
                    .iter()
                    .enumerate()
                    .map(|(index, row)| {
                        let marker = if index == device_selected {
                            "▶ "
                        } else {
                            "  "
                        };

                        let style = if index == device_selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };

                        ListItem::new(format!("{marker}{row}")).style(style)
                    })
                    .collect::<Vec<_>>(),
            };

            frame.render_widget(
                List::new(entries).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Up/Down to select"),
                ),
                areas[1],
            );

            frame.render_widget(
                Paragraph::new(message.as_str())
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Message / keyboard shortcuts"),
                    ),
                areas[2],
            );
        })?;

        if !event::poll(Duration::from_millis(80))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        // Quit from any screen.
        if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') {
            break;
        }

        // Escape returns to the main screen.
        if key.code == KeyCode::Esc {
            if screen != Screen::Main {
                screen = Screen::Main;
                message = String::from("Returned to the main settings screen.");
                refresh_values(&mut values);
                refresh_at = Instant::now();
            } else {
                break;
            }

            continue;
        }

        match screen {
            Screen::Main => {
                let setting = ITEMS[selected];

                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        selected = selected.saturating_sub(1);
                    }

                    KeyCode::Down | KeyCode::Char('j') => {
                        selected = (selected + 1).min(ITEMS.len() - 1);
                    }

                    // Open the Wi-Fi network list.
                    KeyCode::Enter if setting == Setting::Wifi => match platform::wifi_networks() {
                        Ok(networks) => {
                            rows = networks;
                            device_selected = 0;
                            screen = Screen::Wifi;

                            message = String::from("Select a network and press Enter to connect.");
                        }

                        Err(error) => {
                            message = format!("Could not list Wi-Fi networks: {error}");
                        }
                    },

                    // Open the Bluetooth device list.
                    KeyCode::Enter if setting == Setting::Bluetooth => {
                        message = String::from("Scanning for Bluetooth devices...");

                        terminal.draw(|frame| {
                            frame.render_widget(Paragraph::new(message.as_str()), frame.area());
                        })?;

                        match platform::bluetooth_devices(true) {
                            Ok(devices) => {
                                rows = devices;
                                device_selected = 0;
                                screen = Screen::Bluetooth;

                                message =
                                    String::from("Select a device and press Enter to connect.");
                            }

                            Err(error) => {
                                message = format!("Bluetooth scan failed: {error}");
                            }
                        }
                    }

                    // Disable selected Wi-Fi or Bluetooth radio.
                    KeyCode::Char('s') | KeyCode::Char('S')
                        if setting == Setting::Wifi || setting == Setting::Bluetooth =>
                    {
                        update_setting(setting, "off", &mut values, &mut message);

                        refresh_at = Instant::now();
                    }

                    // Enable selected Wi-Fi or Bluetooth radio.
                    KeyCode::Char('e') | KeyCode::Char('E')
                        if setting == Setting::Wifi || setting == Setting::Bluetooth =>
                    {
                        update_setting(setting, "on", &mut values, &mut message);

                        refresh_at = Instant::now();
                    }

                    // Microphone mute toggle.
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        update_setting(Setting::Microphone, "toggle", &mut values, &mut message);
                    }

                    // Volume mute toggle.
                    KeyCode::Char('v') | KeyCode::Char('V') => {
                        update_setting(Setting::Volume, "toggle", &mut values, &mut message);
                    }

                    // Refresh all settings.
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        refresh_values(&mut values);
                        refresh_at = Instant::now();
                        message = String::from("All settings refreshed.");
                    }

                    // Keyboard help.
                    KeyCode::Char('?') => {
                        message = String::from(
                            "Up/Down navigate | Left/Right adjust | Home/End min/max | Enter open | S disable radio | E enable radio | M mic mute | V volume mute | R refresh | Esc/Q quit",
                        );
                    }

                    // Brightness and volume adjustments.
                    KeyCode::Left | KeyCode::Right | KeyCode::Home | KeyCode::End => {
                        if !matches!(setting, Setting::Brightness | Setting::Volume) {
                            message = String::from("Select Brightness or Volume to adjust.");
                            continue;
                        }

                        if last_adjust.is_some_and(|last| last.elapsed() < KEY_REPEAT_DELAY) {
                            continue;
                        }

                        last_adjust = Some(Instant::now());

                        let action = match key.code {
                            KeyCode::Left => "down",
                            KeyCode::Right => "up",
                            KeyCode::Home => "0",
                            KeyCode::End => "100",
                            _ => continue,
                        };

                        update_setting(setting, action, &mut values, &mut message);

                        refresh_at = Instant::now();
                    }

                    // Space toggles microphone mute.
                    KeyCode::Char(' ') if setting == Setting::Microphone => {
                        update_setting(setting, "toggle", &mut values, &mut message);
                    }

                    _ => {}
                }
            }

            Screen::Wifi | Screen::Bluetooth => {
                let setting = if screen == Screen::Wifi {
                    Setting::Wifi
                } else {
                    Setting::Bluetooth
                };

                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        device_selected = device_selected.saturating_sub(1);
                    }

                    KeyCode::Down | KeyCode::Char('j') => {
                        if !rows.is_empty() {
                            device_selected = (device_selected + 1).min(rows.len() - 1);
                        }
                    }

                    // Disable the radio.
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        update_setting(setting, "off", &mut values, &mut message);

                        refresh_at = Instant::now();
                    }

                    // Enable the radio.
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        update_setting(setting, "on", &mut values, &mut message);

                        refresh_at = Instant::now();
                    }

                    // Refresh the current device list.
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        let result = if screen == Screen::Wifi {
                            platform::wifi_networks()
                        } else {
                            platform::bluetooth_devices(true)
                        };

                        match result {
                            Ok(new_rows) => {
                                rows = new_rows;
                                device_selected = 0;
                                message = String::from("Device list refreshed.");
                            }

                            Err(error) => {
                                message = format!("Could not refresh device list: {error}");
                            }
                        }
                    }

                    // Connect to the selected network or device.
                    KeyCode::Enter if !rows.is_empty() => {
                        let chosen = rows.get(device_selected).cloned().unwrap_or_default();

                        if screen == Screen::Wifi {
                            let ssid = chosen
                                .split("  |  ")
                                .next()
                                .unwrap_or("")
                                .trim()
                                .to_string();

                            if ssid.is_empty() || ssid.starts_with("No visible networks") {
                                message = String::from("Select a valid Wi-Fi network.");
                                continue;
                            }

                            // Leave the TUI for the interactive password prompt.
                            match connect_wifi_from_tui(&mut terminal, &ssid) {
                                Ok(result) => {
                                    message = result;
                                }

                                Err(error) => {
                                    message = format!("Wi-Fi connection failed: {error}");
                                }
                            }

                            refresh_values(&mut values);
                            refresh_at = Instant::now();
                        } else {
                            let address =
                                chosen.split_whitespace().nth(1).unwrap_or("").to_string();

                            message = match platform::connect_bluetooth(&address) {
                                Ok(result) => result,
                                Err(error) => {
                                    format!("Bluetooth connection failed: {error}")
                                }
                            };

                            refresh_values(&mut values);
                            refresh_at = Instant::now();
                        }
                    }

                    KeyCode::Char('?') => {
                        message = String::from(
                            "Up/Down select | Enter connect | R refresh/scan | S disable radio | E enable radio | Esc back | Q quit",
                        );
                    }

                    _ => {}
                }
            }
        }
    }

    Ok(())
}
