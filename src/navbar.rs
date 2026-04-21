use iced::{Element, Length, widget::{button, column, svg::Handle}, widget::svg as svg_img};

use crate::{AppState, EventMessage, SongsViewType, menu_view::MenuView};
use crate::tooltip_gen;

pub struct Navbar {}

impl Navbar {
    pub fn view(state: &AppState) -> Element<'_, EventMessage> {
        let top_nav = column![
            tooltip_gen!(button(svg_img(Handle::from_memory(include_bytes!("../assets/icons/music.svg"))).width(36).height(36))
                .style(if let MenuView::SongsView(SongsViewType::All) = state.menu_view {
                        button::success
                    } else {
                        button::primary
                    })
                .on_press(EventMessage::ChangeViewType(SongsViewType::All)), "All"),
            tooltip_gen!(button(svg_img(Handle::from_memory(include_bytes!("../assets/icons/turntable.svg"))).width(36).height(36))
                .style(if let MenuView::SongsView(SongsViewType::ArtistSelect) = state.menu_view {
                        button::success
                    } else {
                        button::primary
                    })
                .on_press(EventMessage::ChangeViewType(SongsViewType::ArtistSelect)), "Artists"),
            tooltip_gen!(button(svg_img(Handle::from_memory(include_bytes!("../assets/icons/palette.svg"))).width(36).height(36))
                .style(if let MenuView::SongsView(SongsViewType::AlbumSelect) = state.menu_view {
                        button::success
                    } else {
                        button::primary
                    })
                .on_press(EventMessage::ChangeViewType(SongsViewType::AlbumSelect)), "Albums"),
        ]
            .spacing(8)
            .height(Length::Fill);

        let bottom_nav = column![
            tooltip_gen!(button(svg_img(Handle::from_memory(include_bytes!("../assets/icons/arrow-down-wide-narrow.svg"))).width(36).height(36))
                .on_press(EventMessage::ToggleSortType), state.sort_type.as_str()),
            tooltip_gen!(button(svg_img(Handle::from_memory(include_bytes!("../assets/icons/sliders-horizontal.svg"))).width(36).height(36))
                .on_press(EventMessage::ChangeMenuView(MenuView::Settings)), "Settings"),
        ].spacing(8);

        column![top_nav, bottom_nav]
            .width(64)
            .height(Length::Fill)
            .into()
    }
}
