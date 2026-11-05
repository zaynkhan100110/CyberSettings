use super::Setting;
use anyhow::{Result, bail};
pub fn status(s: Setting) -> String {
    format!("{s}: not implemented on macOS yet")
}
pub fn perform(s: Setting, _a: &str, _v: Option<&str>) -> Result<String> {
    bail!("{s} control is not implemented on macOS yet. No change was made.")
}
