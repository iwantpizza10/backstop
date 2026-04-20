use std::{rc::Rc, sync::Arc};

use iced::{Element, Length, alignment::{Horizontal, Vertical}, widget::{button, column, row, scrollable, text}};

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
    pub fn view(&self, assets: Rc<AppAssets>, songs: &Vec<Arc<SongFileInfo>>) -> Element<'_, EventMessage> {
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
                row![
                    scrollable(iced::widget::Column::from_iter(
                        songs.iter()
                            .map(|x| row![text(format!("{} - {}", x.artist(), x.title())), button("play").on_press(EventMessage::PlaySong(Arc::clone(&x)))].into())
                    ))
                ]
            },
            _ => {
                todo!()
            },
        }.into()
    }
}
