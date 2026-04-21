use iced::{Element, Length, widget::{button, column, text}};

use crate::{AppState, EventMessage, SongsViewType, menu_view::MenuView};

pub struct Navbar {}

// todo: non-placeholder navbar

impl Navbar {
    pub fn view(state: &AppState) -> Element<'_, EventMessage> {
        let top_nav = column![
            button("all")
                .style({
                    if let MenuView::SongsView(SongsViewType::All) = state.menu_view {
                        button::success
                    } else {
                        button::primary
                    }
                })
                .on_press(EventMessage::ChangeViewType(SongsViewType::All)),
            button("artists")
                .style({
                    if let MenuView::SongsView(SongsViewType::ArtistSelect) = state.menu_view {
                        button::success
                    } else {
                        button::primary
                    }
                })
                .on_press(EventMessage::ChangeViewType(SongsViewType::ArtistSelect)),
            button("albums")
                .style({
                    if let MenuView::SongsView(SongsViewType::AlbumSelect) = state.menu_view {
                        button::success
                    } else {
                        button::primary
                    }
                })
                .on_press(EventMessage::ChangeViewType(SongsViewType::AlbumSelect)),
        ]
            .spacing(8)
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
