use std::time::Duration;
use color_from_hex::color_from_hex;
use iced::{Background, Color, Element, Length, Theme, alignment::{Horizontal, Vertical}, border, widget::{Column, Image, Row, button, button::{Status, Style}, column, container, image::Handle, mouse_area, progress_bar, row, slider, space, svg, text}};

use crate::{AppState, EventMessage, PlayingState};
use crate::constants::{SPEED_STEPS, VOLUME_DYNAMIC_RANGE_DB};
use crate::menu_view::MenuView;

pub struct Footer {}

impl Footer {
    pub fn view(state: &AppState) -> Element<'_, EventMessage> {
        container(row![ left_nav(state), center_nav(state), right_nav(state) ]
            .align_y(Vertical::Center)
            .height(128)).style(|_| {
                container::Style {
                    background: Some(Background::Color(color_from_hex!("#170f37"))),
                    ..container::Style::default()
                }
            })
            .into()
    }
}

fn left_nav<'a>(state: &'a AppState) -> Element<'a, EventMessage> {
    let image_handle;

    if let Some(song) = &state.current_song {
        let mut col = Column::new();

        if let Some(cover) = &song.file_info.cover {
            image_handle = Handle::from_path(cover);
        } else {
            image_handle = state.assets.cover.clone();
        }

        if let Some(album) = &song.file_info.album {
            if let Some(track) = &song.file_info.track_number {
                col = col.push(text!("{album} #{track}")
                    .size(12));
            } else {
                col = col.push(text(album.clone())
                    .size(12));
            }
        }

        col = col.push(text(song.file_info.artist())
            .size(12));

        return row![
            space(),
            mouse_area(Image::new(image_handle).height(120).width(120))
                .on_press(EventMessage::ChangeMenuView(MenuView::CoverArtView)),
            space().width(4),
            row![
                column![
                    text(song.file_info.title())
                        .size(18),
                    col,
                ].spacing(4),
            ]
                .height(120)
                .align_y(Vertical::Center),
        ]
            .spacing(4)
            .width(Length::Fill)
            .height(128)
            .align_y(Vertical::Center)
            .into();
    }

    Row::new()
        .width(Length::Fill)
        .into()
}

fn center_nav<'a>(state: &'a AppState) -> Element<'a, EventMessage> {
    let timestamp_one = duration_to_string(state.player.get_pos());
    let timestamp_two = duration_to_string(state.player.get_duration());

    macro_rules! make_button {
        ($path:expr, $evt:expr) => {
            button(svg(svg::Handle::from_memory(include_bytes!($path)))
                    .width(48)
                    .height(48))
                .width(48)
                .height(48)
                .on_press($evt)
        };
    }

    column![
        row![
            text!("{}", timestamp_one),
            progress_bar(0.0..=state.player.get_duration().as_millis() as f32, state.player.get_pos().as_millis() as f32)
                .girth(8),
            text!("{}", timestamp_two),
        ]
            .spacing(16)
            .align_y(Vertical::Center),
        row![
            make_button!("../assets/icons/eraser.svg", EventMessage::ClearDiscordRpc)
                .style(|a, b| button_style_thing(a, b, false)),
            make_button!("../assets/icons/shuffle.svg", EventMessage::ToggleShuffle)
                .style(|a, b| button_style_thing(a, b, state.saved_state.settings.get_shuffle())),
            make_button!("../assets/icons/arrow-big-left-dash.svg", EventMessage::PrevTrack)
                .style(|a, b| button_style_thing(a, b, false)),
            if state.playing == PlayingState::Playing {
                make_button!("../assets/icons/pause.svg", EventMessage::PlayPause)
            } else {
                make_button!("../assets/icons/play.svg", EventMessage::PlayPause)
            }
                .style(|a, b| button_style_thing(a, b, false)),
            make_button!("../assets/icons/arrow-big-right-dash.svg", EventMessage::NextTrack)
                .style(|a, b| button_style_thing(a, b, false)),
            make_button!("../assets/icons/repeat.svg", EventMessage::ToggleRepeat)
                .style(|a, b| button_style_thing(a, b, state.saved_state.settings.get_repeat())),
            make_button!("../assets/icons/list-plus.svg", EventMessage::ToggleQueuePeek)
                .style(|a, b| button_style_thing(a, b, state.peeking_queue)),
        ]
            .spacing(8),
    ]
        .spacing(8)
        .width(384)
        .into()
}

fn right_nav<'a>(state: &'a AppState) -> Element<'a, EventMessage> {
    row![
        column![
            text!("Vol: {}dB", state.saved_state.settings.get_volume_db()),
            slider(0..=VOLUME_DYNAMIC_RANGE_DB, state.saved_state.settings.get_volume_step(), EventMessage::SetVolume)
                .width(128),
            space().height(8),
            text!("Spd: {}x", state.saved_state.settings.get_speed()),
            slider(1..=SPEED_STEPS, state.saved_state.settings.get_speed_step(), EventMessage::SetSpeed)
                .default(SPEED_STEPS / 2)
                .width(128),
        ].align_x(Horizontal::Right)
        .width(Length::Fill),
        space().width(8),
    ]
        .into()
}

fn duration_to_string(dur: Duration) -> String {
    let minutes = dur.as_secs() / 60;
    let seconds = dur.as_secs() - (minutes * 60);

    format!("{:02}:{:02}", minutes, seconds)
}

fn button_style_thing(theme: &Theme, status: Status, active: bool) -> Style {
    let palette = theme.extended_palette();
    let base = Style {
        background: if active {
            Some(Background::Color(palette.success.base.color))
        } else {
            Some(Background::Color(palette.primary.base.color))
        },
        text_color: palette.primary.base.text,
        border: border::rounded(10),
        ..Style::default()
    };

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: if active {
                Some(Background::Color(palette.success.strong.color))
            } else {
                Some(Background::Color(palette.primary.strong.color))
            },
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

fn disabled(style: Style) -> Style {
    Style {
        background: style
            .background
            .map(|background| background.scale_alpha(0.5)),
        text_color: style.text_color.scale_alpha(0.5),
        ..style
    }
}
