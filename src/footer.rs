use std::time::Duration;
use color_from_hex::color_from_hex;
use iced::{Background, Element, Length, alignment::{Horizontal, Vertical}, widget::{Column, Image, Row, column, container, image::Handle, mouse_area, progress_bar, row, slider, space, text}};

use crate::{AppState, EventMessage, PlayingState};
use crate::constants::{SPEED_STEPS, VOLUME_DYNAMIC_RANGE_DB};
use crate::menu_view::MenuView;
use crate::svg_button::button_style;
use crate::make_svg_button;

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
            make_svg_button!(include_bytes!("../assets/icons/eraser.svg"), EventMessage::ClearQueue, 48)
                .style(|a, b| button_style(a, b, false)),
            make_svg_button!(include_bytes!("../assets/icons/shuffle.svg"), EventMessage::ToggleShuffle, 48)
                .style(|a, b| button_style(a, b, state.saved_state.settings.get_shuffle())),
            make_svg_button!(include_bytes!("../assets/icons/arrow-big-left-dash.svg"), EventMessage::PrevTrack, 48)
                .style(|a, b| button_style(a, b, false)),
            if state.playing == PlayingState::Playing {
                make_svg_button!(include_bytes!("../assets/icons/pause.svg"), EventMessage::PlayPause, 48)
            } else {
                make_svg_button!(include_bytes!("../assets/icons/play.svg"), EventMessage::PlayPause, 48)
            }
                .style(|a, b| button_style(a, b, false)),
            make_svg_button!(include_bytes!("../assets/icons/arrow-big-right-dash.svg"), EventMessage::NextTrack, 48)
                .style(|a, b| button_style(a, b, false)),
            make_svg_button!(include_bytes!("../assets/icons/repeat.svg"), EventMessage::ToggleRepeat, 48)
                .style(|a, b| button_style(a, b, state.saved_state.settings.get_repeat())),
            make_svg_button!(include_bytes!("../assets/icons/list-plus.svg"), EventMessage::ToggleQueuePeek, 48)
                .style(|a, b| button_style(a, b, state.peeking_queue)),
        ]
            .spacing(8),
    ]
        .spacing(12)
        .width(384)
        .into()
}

fn right_nav<'a>(state: &'a AppState) -> Element<'a, EventMessage> {
    row![
        column![
            text!("Gain: {}dB", state.saved_state.settings.get_volume_db())
                .size(14),
            slider(0..=VOLUME_DYNAMIC_RANGE_DB, state.saved_state.settings.get_volume_step(), EventMessage::SetVolume)
                .width(128),
            space().height(8),
            text!("Speed: {}x", state.saved_state.settings.get_speed())
                .size(14),
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
