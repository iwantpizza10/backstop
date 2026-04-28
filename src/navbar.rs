use color_from_hex::color_from_hex;
use iced::{Background, Element, Length, alignment::Horizontal, widget::{column, container, space, svg::Handle}};
use iced::widget::svg as svg_img;
use iced::widget::button;

use crate::{AppState, EventMessage, SongsViewType, menu_view::MenuView, svg_button::button_style};
use crate::{make_svg_button, tooltip_gen};

pub struct Navbar {}

impl Navbar {
    pub fn view(state: &AppState) -> Element<'_, EventMessage> {
        let top_nav = column![
            space().width(56),
            tooltip_gen!(make_svg_button!(include_bytes!("../assets/icons/music.svg"), EventMessage::ChangeViewType(SongsViewType::All), 56)
                .style(|a, b| button_style(a, b, MenuView::SongsView(SongsViewType::All) == state.menu_view)), "All"),
            tooltip_gen!(make_svg_button!(include_bytes!("../assets/icons/turntable.svg"), EventMessage::ChangeViewType(SongsViewType::ArtistSelect), 56)
                .style(|a, b| button_style(a, b, MenuView::SongsView(SongsViewType::ArtistSelect) == state.menu_view)), "Artists"),
            tooltip_gen!(make_svg_button!(include_bytes!("../assets/icons/palette.svg"), EventMessage::ChangeViewType(SongsViewType::AlbumSelect), 56)
                .style(|a, b| button_style(a, b, MenuView::SongsView(SongsViewType::AlbumSelect) == state.menu_view)), "Albums"),
        ]
            .spacing(8)
            .height(Length::Fill);

        let bottom_nav = column![
            tooltip_gen!(button(svg_img(Handle::from_memory(include_bytes!("../assets/icons/arrow-down-wide-narrow.svg"))).width(36).height(36))
                .width(56).height(56)
                .style(|a, b| button_style(a, b, false))
                .on_press(EventMessage::ToggleSortType), state.sort_type.as_str()),
            tooltip_gen!(button(svg_img(Handle::from_memory(include_bytes!("../assets/icons/sliders-horizontal.svg"))).width(36).height(36))
                .width(56).height(56)
                .style(|a, b| button_style(a, b, false))
                .on_press(EventMessage::ChangeMenuView(MenuView::Settings)), "Settings"),
            space().width(56),
        ].spacing(8);

        container(column![top_nav, bottom_nav]
                .width(64)
                .height(Length::Fill)
                .align_x(Horizontal::Center))
            .style(|_| {
                container::Style {
                    background: Some(Background::Color(color_from_hex!("#110b29"))),
                    ..container::Style::default()
                }
            })
            .into()
    }
}
