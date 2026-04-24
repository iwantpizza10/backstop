use iced::{Element, Length, alignment::{Horizontal, Vertical}, widget::{Column, Image, Row, button, column, image::Handle, mouse_area, progress_bar, row, slider, svg, text}};

use crate::{AppState, EventMessage, PlayingState, constants::{SPEED_STEPS, VOLUME_DYNAMIC_RANGE_DB}, menu_view::MenuView};

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
            mouse_area(Image::new(image_handle))
                .on_press(EventMessage::ChangeMenuView(MenuView::CoverArtView)),
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
            .width(Length::Fill)
            .align_y(Vertical::Center)
            .into();
    }

    Row::new()
        .width(Length::Fill)
        .into()
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
        row![
            text!("{}", timestamp_one),
            progress_bar(0.0..=state.current_song.clone().map_or(100.0, |x| x.duration.as_millis() as f32), 0.0)
                .girth(8),
            text!("{}", timestamp_two),
        ]
            .spacing(16)
            .align_y(Vertical::Center),
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
    ]
        .width(384)
        .into()
}

fn right_nav<'a>(state: &'a AppState) -> Element<'a, EventMessage> {
    column![
        text!("Vol: {}dB", state.saved_state.settings.get_volume_db()),
        slider(0..=VOLUME_DYNAMIC_RANGE_DB, state.saved_state.settings.get_volume_step(), EventMessage::SetVolume)
            .width(128),
        text!("Spd: {}x", state.saved_state.settings.get_speed()),
        slider(1..=SPEED_STEPS, state.saved_state.settings.get_speed_step(), EventMessage::SetSpeed)
            .default(SPEED_STEPS / 2)
            .width(128),
    ]
        .align_x(Horizontal::Right)
        .width(Length::Fill)
        .into()
}
