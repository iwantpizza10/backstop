use iced::{Background, Theme, border, widget::button::{Status, Style}};

#[macro_export(local_inner_macros)]
macro_rules! make_svg_button {
    ($bytes:expr, $evt:expr, $side:literal) => {
        iced::widget::button(iced::widget::svg(iced::widget::svg::Handle::from_memory($bytes))
                .width(36)
                .height(36))
            .width($side)
            .height($side)
            .on_press($evt)
    };
}

pub fn button_style(theme: &Theme, status: Status, active: bool) -> Style {
    let palette = theme.extended_palette();
    let base = Style {
        background: if active {
            Some(Background::Color(palette.success.base.color))
        } else {
            Some(Background::Color(palette.primary.base.color))
        },
        text_color: palette.primary.base.text,
        border: border::rounded(10),
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
        Status::Disabled => Style {
            background: base
                .background
                .map(|background| background.scale_alpha(0.5)),
            text_color: base.text_color.scale_alpha(0.5),
            ..base
        },
    }
}
