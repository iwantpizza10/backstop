use iced::{Element, Length, alignment::Vertical, widget::{Column, Image, Row, button, column, image::Handle, row, svg, text}};

use crate::{AppState, EventMessage, PlayingState};

pub struct Footer {}

impl Footer {
    pub fn view(state: &AppState) -> Element<'_, EventMessage> {
        row![ left_nav(state), center_nav(state), right_nav(state) ]
            .height(128)
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
            Image::new(image_handle),
            row![
                column![
                    text(song.file_info.title())
                        .size(18),
                    col,
                ].spacing(8),
            ]
                .height(Length::Shrink)
                .align_y(Vertical::Center),
        ]
            .spacing(8)
            .align_y(Vertical::Center)
            .into();
    }

    Row::new().into()
}

fn center_nav<'a>(state: &'a AppState) -> Element<'a, EventMessage> {
    let timestamp_one;
    let timestamp_two;

    if let Some(song) = &state.current_song {
        timestamp_one = String::from("?:??");
        let secs = song.duration.as_secs() % 60;
        let mins = song.duration.as_secs() / 60;

        timestamp_two = format!("{mins}:{secs:02}");
    } else {
        timestamp_one = String::from("0:00");
        timestamp_two = String::from("0:00");
    }

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
        // todo: proper song progress bar
        text!("{} / {}", timestamp_one, timestamp_two),

        // todo: custom style buttons
        row![
            make_button!("../assets/icons/eraser.svg", EventMessage::ClearDiscordRpc),
            make_button!("../assets/icons/shuffle.svg", EventMessage::ToggleShuffle),
            make_button!("../assets/icons/arrow-big-left-dash.svg", EventMessage::PrevTrack),
            if state.playing == PlayingState::Playing {
                make_button!("../assets/icons/pause.svg", EventMessage::PlayPause)
            } else {
                make_button!("../assets/icons/play.svg", EventMessage::PlayPause)
            },
            make_button!("../assets/icons/arrow-big-right-dash.svg", EventMessage::NextTrack),
            make_button!("../assets/icons/repeat.svg", EventMessage::ToggleRepeat),
            make_button!("../assets/icons/list-plus.svg", EventMessage::ToggleQueuePeek),
        ]
            .spacing(8),
    ].into()
}

fn right_nav<'a>(state: &'a AppState) -> Element<'a, EventMessage> {
    column![
        // todo: volume slider
        // todo: speed slider
        text!("sliders n stuff"),
    ].into()
}
