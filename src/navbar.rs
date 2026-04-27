use iced::{Background, Element, Length, Theme, alignment::Horizontal, border, widget::{button::{Status, Style}, column, space, svg::Handle}};
use iced::widget::svg as svg_img;
use iced::widget::button;

use crate::{AppState, EventMessage, SongsViewType, menu_view::MenuView};
use crate::tooltip_gen;

pub struct Navbar {}

impl Navbar {
    pub fn view(state: &AppState) -> Element<'_, EventMessage> {
        let top_nav = column![
            space().width(56),
            tooltip_gen!(button(svg_img(Handle::from_memory(include_bytes!("../assets/icons/music.svg"))).width(36).height(36))
                .width(56).height(56)
                .style(|a, b| button_style_thing(a, b, MenuView::SongsView(SongsViewType::All) == state.menu_view))
                .on_press(EventMessage::ChangeViewType(SongsViewType::All)), "All"),
            tooltip_gen!(button(svg_img(Handle::from_memory(include_bytes!("../assets/icons/turntable.svg"))).width(36).height(36))
                .width(56).height(56)
                .style(|a, b| button_style_thing(a, b, MenuView::SongsView(SongsViewType::ArtistSelect) == state.menu_view))
                .on_press(EventMessage::ChangeViewType(SongsViewType::ArtistSelect)), "Artists"),
            tooltip_gen!(button(svg_img(Handle::from_memory(include_bytes!("../assets/icons/palette.svg"))).width(36).height(36))
                .width(56).height(56)
                .style(|a, b| button_style_thing(a, b, MenuView::SongsView(SongsViewType::AlbumSelect) == state.menu_view))
                .on_press(EventMessage::ChangeViewType(SongsViewType::AlbumSelect)), "Albums"),
        ]
            .spacing(8)
            .height(Length::Fill);

        let bottom_nav = column![
            tooltip_gen!(button(svg_img(Handle::from_memory(include_bytes!("../assets/icons/arrow-down-wide-narrow.svg"))).width(36).height(36))
                .width(56).height(56)
                .style(|a, b| button_style_thing(a, b, false))
                .on_press(EventMessage::ToggleSortType), state.sort_type.as_str()),
            tooltip_gen!(button(svg_img(Handle::from_memory(include_bytes!("../assets/icons/sliders-horizontal.svg"))).width(36).height(36))
                .width(56).height(56)
                .style(|a, b| button_style_thing(a, b, false))
                .on_press(EventMessage::ChangeMenuView(MenuView::Settings)), "Settings"),
            space().width(56),
        ].spacing(8);

        column![top_nav, bottom_nav]
            .width(64)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .into()
    }
}

// todo: colors

// lowkey took this (& the next couple functions) from button::primary and made changes
fn button_style_thing(theme: &Theme, status: Status, active: bool) -> Style {
    let palette = theme.extended_palette();
    let base = Style {
        background: if active {
            Some(Background::Color(palette.success.base.color))
        } else {
            Some(Background::Color(palette.primary.base.color))
        },
        text_color: palette.primary.base.text,
        border: border::rounded(15),
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