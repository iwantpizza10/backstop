use std::{rc::Rc, sync::Arc};
use iced::{Element, Length, alignment::{Horizontal, Vertical}, widget::{Row, button, column, row, scrollable, space, text}};

use crate::{AppAssets, EventMessage, SongsViewType, saved_state::song_file_info::SongFileInfo};

#[derive(Default, Clone, Debug)]
pub enum MenuView {
    #[default]
    Welcome,
    SongsView(SongsViewType),
    CoverArtView,
    Settings,
}

impl MenuView {
    pub fn view<'a>(&self, assets: Rc<AppAssets>, songs: &'a Vec<Arc<SongFileInfo>>, songs_per_row: i32) -> Element<'a, EventMessage> {
        match self {
            Self::Welcome => {
                let buttons_row = row![
                    button("Add a Media Directory")
                        .on_press(EventMessage::TriggerAddMediaDir),
                    button("Scan Library")
                        .on_press(EventMessage::TriggerRescanLibrary),
                    button("Browse Library")
                        .on_press(EventMessage::ChangeMenuView(MenuView::SongsView(SongsViewType::default()))),
                ].spacing(16);

                row![
                    column![
                        iced::widget::image(&assets.logo)
                            .height(96),
                        text("Looks like you're new here!")
                            .size(32),
                        buttons_row,
                        text("These buttons are ordered; you must first add a directory that contains audio files with metadata, scan the library, and finally browse your library to play songs.")
                            .width(Length::Fixed(384.0))
                            .center(),
                    ].spacing(32)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center),
                ].height(Length::Fill)
                    .align_y(Vertical::Center)
            },
            Self::SongsView(view_type) => {
                let mut rows = vec![
                    space()
                        .height(8)
                        .into()
                ];

                match view_type {
                    SongsViewType::All => {
                        for i in 0..(songs.len() as i32 / songs_per_row) {
                            let mut row_songs = vec![];

                            for j in 0..songs_per_row as i32 {
                                row_songs.push(songs[((i * songs_per_row) + j) as usize].view(Rc::clone(&assets)));
                            }

                            rows.push(Row::from_vec(row_songs)
                                .spacing(10)
                                .into());
                        }

                        let remaining_songs = songs.len() as i32 % songs_per_row;
                        let mut row_songs = vec![];

                        for i in (songs.len() as i32 - remaining_songs)..songs.len() as i32 {
                            row_songs.push(songs[i as usize].view(Rc::clone(&assets)));
                        }

                        for _ in remaining_songs..songs_per_row {
                            row_songs.push(text("").width(192).into());
                        }

                        rows.push(Row::from_vec(row_songs)
                            .spacing(10)
                            .into());
                    },

                    x => {
                        todo!("view {x:?}");
                    }
                }

                rows.push(space()
                    .height(32)
                    .into());

                row![
                    scrollable(iced::widget::Column::from_vec(rows)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .spacing(16))
                ]
            },
            _ => {
                todo!()
            },
        }.into()
    }
}
