use iced::{Element, Length, widget::{button, column, text}};

use crate::{AppState, EventMessage, SongsViewType};

pub struct Navbar {}

impl Navbar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(state: &AppState) -> Element<'_, EventMessage> {
        let top_nav = column![
            button("all")
                .on_press(EventMessage::ChangeViewType(SongsViewType::All)),
            button("artists")
                .on_press(EventMessage::ChangeViewType(SongsViewType::ArtistSelect)),
            button("albums")
                .on_press(EventMessage::ChangeViewType(SongsViewType::AlbumSelect)),
        ]
            .height(Length::Fill);

        let bottom_nav = column![
            text("filters (?)"),
            text("settings"),
        ];

        column![top_nav, bottom_nav]
            .width(64)
            .height(Length::Fill)
            .into()
    }
}
