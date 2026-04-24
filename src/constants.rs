use std::path::PathBuf;

pub const BACKSTOP_LOGO: &[u8] = include_bytes!("../assets/backstopfull.png");
pub const PLACEHOLDER_COVER: &[u8] = include_bytes!("../assets/cover_placeholder.png");

pub const VOLUME_DYNAMIC_RANGE_DB: i32 = 20;
pub const SPEED_STEPS: i32 = 40;

pub const BACKSTOP_PAUSE_ICON_URL: &str = "https://github.com/iwantpizza10/backstop/blob/main/assets/pause.png?raw=true";
pub const BACKSTOP_LOGO_URL: &str = "https://github.com/iwantpizza10/backstop/blob/main/assets/backstopshort_square.png?raw=true";
pub const BACKSTOP_REPO_URL: &str = "https://github.com/iwantpizza10/backstop/releases";
pub const DISCORD_APP_ID: &str = "1483067786589765702";
pub const MUSIC_EXTS: [&str; 7] = [
    "mp3", "ogg", "flac", "m4a",
    "aac", "wav", "opus",
];

#[cfg(target_os="linux")]
static CONFIG_DIR: &str = "~/.config/backstop/";
#[cfg(target_os="windows")]
static CONFIG_DIR: &str = "\\\\?/%localappdata%/backstop/";

pub fn conf_dir() -> PathBuf {
    let mut dir = dirs::config_local_dir().unwrap_or_else(|| PathBuf::from(CONFIG_DIR));

    dir.push("backstop");

    dir
}

#[macro_export(local_inner_macros)]
macro_rules! softunwrap_str {
    ($x:expr) => {
        $x.map_or(None, |a| Some(a.to_string()))
    };
}

#[macro_export(local_inner_macros)]
macro_rules! tooltip_gen {
    ($a:expr, $b:expr) => {
        iced::widget::tooltip($a, iced::widget::button($b).style(|_, _| {
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(iced::Color::BLACK)),
                    text_color: iced::Color::WHITE,
                    border: iced::Border::default().rounded(5),
                    shadow: iced::Shadow::default(),
                    snap: false
                }
            }), iced::widget::tooltip::Position::Right)
    };

    ($a:expr, $b:expr, $c:expr) => {
        iced::widget::tooltip($a, iced::widget::button($b).style(|_, _| {
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(iced::Color::BLACK)),
                    text_color: iced::Color::WHITE,
                    border: iced::Border::default().rounded(5),
                    shadow: iced::Shadow::default(),
                    snap: false
                }
            }), $c)
    };
}
