use std::ops::RangeInclusive;
use iced::{Element, Event, Length, Rectangle, Size, advanced::{Clipboard, Layout, Widget, graphics::core::window, layout, mouse, renderer, widget::Tree}, widget::{ProgressBar, progress_bar}};

pub struct UpdatingProgressBar<'a, F> where F: Fn() -> f32 {
    range: RangeInclusive<f32>,
    value: F,
    length: Length,
    girth: Length,
    is_vertical: bool,
    style: progress_bar::StyleFn<'a, iced::Theme>,
}

pub fn updating_progress_bar<'a, F>(range: RangeInclusive<f32>, value: F) -> UpdatingProgressBar<'a, F> where F: Fn() -> f32 {
    UpdatingProgressBar::new(range, value)
}

impl<'a, F> UpdatingProgressBar<'a, F> where F: Fn() -> f32 {
    pub fn new(range: RangeInclusive<f32>, value: F) -> Self {
        Self {
            range, value,
            length: Length::Fill,
            girth: Length::from(ProgressBar::<iced::Theme>::DEFAULT_GIRTH),
            is_vertical: false,
            style: Box::new(progress_bar::primary),
        }
    }

    pub fn length(mut self, length: impl Into<Length>) -> Self {
        self.length = length.into();
        self
    }

    pub fn girth(mut self, girth: impl Into<Length>) -> Self {
        self.girth = girth.into();
        self
    }

    pub fn vertical(mut self) -> Self {
        self.is_vertical = true;
        self
    }

    pub fn style(mut self, style: impl Fn(&iced::Theme) -> progress_bar::Style + 'a) -> Self {
        self.style = Box::new(style);
        self
    }

    fn current_value(&self) -> f32 {
        (self.value)()
    }

    /// produces a temporary bar, do not store this
    fn as_progress_bar(&self) -> ProgressBar<'_, iced::Theme> {
        let mut bar = ProgressBar::new(self.range.clone(), self.current_value())
            .length(self.length)
            .girth(self.girth)
            .style(|theme| (self.style)(theme));

        if self.is_vertical {
            bar = bar.vertical();
        }

        bar
    }
}

impl<Message, Renderer, F> Widget<Message, iced::Theme, Renderer> for UpdatingProgressBar<'_, F> where Renderer: iced::advanced::Renderer, F: Fn() -> f32 {
    fn size(&self) -> Size<Length> {
        <ProgressBar<'_, iced::Theme> as Widget<Message, iced::Theme, Renderer>>::size(&self.as_progress_bar())
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let mut inner = self.as_progress_bar();

        <ProgressBar<'_, iced::Theme> as Widget<Message, iced::Theme, Renderer>>::layout(&mut inner, tree, renderer, limits)
    }

    fn update(&mut self, tree: &mut Tree, event: &Event, layout: Layout<'_>, cursor: mouse::Cursor, // more!
    renderer: &Renderer, clipboard: &mut dyn Clipboard, shell: &mut iced::advanced::Shell<'_, Message>, viewport: &Rectangle ) {
        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            shell.request_redraw();
        }

        let mut inner = self.as_progress_bar();

        <ProgressBar<'_, iced::Theme> as Widget<Message, iced::Theme, Renderer>>::update(&mut inner, tree, event, layout, cursor, renderer, clipboard, shell, viewport);
    }

    fn draw(&self, tree: &Tree, renderer: &mut Renderer, theme: &iced::Theme, style: &renderer::Style, // more pt 2!
    layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle) {
        <ProgressBar<'_, iced::Theme> as Widget<Message, iced::Theme, Renderer>>::draw(&self.as_progress_bar(), tree, renderer, theme, style, layout, cursor, viewport);
    }
}

impl<'a, Message, Renderer, F> From<UpdatingProgressBar<'a, F>> for Element<'a, Message, iced::Theme, Renderer>
where Renderer: 'a + iced::advanced::Renderer, F: Fn() -> f32 + 'a {
    fn from(progress_bar: UpdatingProgressBar<'a, F>) -> Self {
        Element::new(progress_bar)
    }
}
