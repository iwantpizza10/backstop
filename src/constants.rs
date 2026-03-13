use std::path::PathBuf;

pub const MUSIC_EXTS: [&str; 8] = [
    "mp3", "ogg", "ogg", "flac",
    "m4a", "aac", "wav", "opus"
];

#[cfg(target_os = "linux")]
pub static CONFIG_DIR: &str = "~/.config/backstop/";
#[cfg(target_os = "windows")]
pub static CONFIG_DIR: &str = "\\\\?/%localappdata%/backstop/";

pub fn conf_dir() -> PathBuf {
    let mut dir = dirs::config_local_dir().unwrap_or_else(|| PathBuf::from(CONFIG_DIR));

    dir.push("backstop");

    dir
}
