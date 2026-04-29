use std::{rc::Rc, sync::Arc};
use color_from_hex::color_from_hex;
use iced::{Background, Border, Element, Length, alignment::{Horizontal, Vertical}, widget::{Image, Row, button, column, container, image::Handle, mouse_area, pick_list, row, scrollable, space, text, text_input}};
use iced::widget::image as img;

use crate::{AppAssets, AppState, EventMessage, SongsViewType, clip, discord_rpc::DiscordRpcMode};

#[derive(Default, Clone, Debug, PartialEq)]
pub enum MenuView {
    #[default]
    Welcome,
    SongsView(SongsViewType),
    CoverArtView,
    Settings,
}

impl MenuView {
    pub fn view<'a>(&self, state: &'a AppState) -> Element<'a, EventMessage> {
        let assets = Rc::clone(&state.assets);

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
                let items_per_row = state.items_per_row;
                let mut rows = vec![
                    space()
                        .height(8)
                        .into()
                ];

                match view_type {
                    SongsViewType::All | SongsViewType::Artist(_) | SongsViewType::Album(_) => {
                        let songs = state.saved_state.media_cache.songs();

                        rowgen(songs, &mut rows, items_per_row, assets);
                    },
                    SongsViewType::AlbumSelect => {
                        let albums = state.saved_state.media_cache.albums();

                        rowgen(albums, &mut rows, items_per_row, assets);
                    },
                    SongsViewType::ArtistSelect => {
                        let artists = state.saved_state.media_cache.artists();

                        rowgen(artists, &mut rows, items_per_row, assets);
                    },
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

            Self::CoverArtView => {
                if let Some(cur_song) = &state.current_song {
                    let cover_path = cur_song.file_info.cover.as_ref().unwrap().to_string_lossy().to_string();

                    let mut col = column![
                        img(cover_path).height(384).width(384),
                        space().height(16),
                        text(cur_song.file_info.title())
                            .size(36),
                    ];

                    col = col.push(space().height(4));

                    if let Some(line) = cur_song.file_info.album_line() {
                        col = col.push(text(line)
                            .size(18));
                    }

                    col = col.push(text(cur_song.file_info.artist())
                        .size(18));
                    
                    row![
                        col
                            .align_x(Horizontal::Center)
                            .width(Length::Fill),
                    ]
                        .align_y(Vertical::Center)
                        .height(Length::Fill)
                } else {
                    row![
                        text("how did we get here?"),
                    ]
                }
            },

            Self::Settings => {
                let mut media_dirs_col = column![].spacing(4);
                let mut rpc_items_col = column![].spacing(4);

                for i in state.saved_state.settings.get_media_directories() {
                    media_dirs_col = media_dirs_col.push(container(row![
                        space().width(4),
                        text(i.to_string_lossy())
                            .width(Length::Fill),
                        button("Remove")
                            .on_press(EventMessage::RemoveMediaDir(i.to_string_lossy().to_string()))
                    ].align_y(Vertical::Center))
                        .padding(4)
                        .style(|_| container::Style {
                        background: Some(Background::Color(color_from_hex!("#170f37"))),
                        border: Border::default().rounded(5),
                        ..container::Style::default()
                    }));
                }

                for i in state.saved_state.settings.get_rpc_list() {
                    rpc_items_col = rpc_items_col.push(container(row![
                        space().width(4),
                        text(i)
                            .width(Length::Fill),
                        button("Remove")
                            .on_press(EventMessage::RemoveRpcListEntry(i.clone()))
                    ].align_y(Vertical::Center))
                        .padding(4)
                        .style(|_| container::Style {
                        background: Some(Background::Color(color_from_hex!("#170f37"))),
                        border: Border::default().rounded(5),
                        ..container::Style::default()
                    }));
                }

                return scrollable(row![
                    space().width(Length::FillPortion(2)),
                    column![
                        space().height(32),
                        text("Backstop Settings")
                            .size(36),
                        space(),

                        column![
                            text("Media Directories")
                                .size(24),
                            media_dirs_col,
                            row![
                                button("Add a Media Directory")
                                    .on_press(EventMessage::TriggerAddMediaDir),
                                button("Scan Library")
                                    .on_press(EventMessage::TriggerRescanLibrary),
                            ].spacing(8)
                        ].spacing(4),

                        column![
                            text("Discord RPC")
                                .size(24),
                            rpc_items_col,
                            text(match state.saved_state.settings.get_rpc_mode() {
                                DiscordRpcMode::Blacklist => "If any song's title or artist CONTAINS one of these, it will not be shown.",
                                DiscordRpcMode::Whitelist => "A song will only be shown if its title or artist CONTAINS one of these.",
                                DiscordRpcMode::Disabled => "That list has no effect as RPC is disabled.",
                            }).size(12),
                            row![
                                text_input("Title/artist of songs...", &state.rpc_text_input)
                                    .on_submit(EventMessage::AddRpcListEntry)
                                    .on_input(EventMessage::UpdateRPCTextInput),
                                button("Add")
                                    .on_press(EventMessage::AddRpcListEntry),
                                pick_list(DiscordRpcMode::list_all(), Some(state.saved_state.settings.get_rpc_mode()), EventMessage::SetDiscordRpcMode)
                            ].spacing(8)
                        ].spacing(4),
                        space().height(32),
                    ]
                        .spacing(32)
                        .width(Length::FillPortion(4))
                        .align_x(Horizontal::Center),
                    space().width(Length::FillPortion(2)),
                ].align_y(Vertical::Center)
                    .height(Length::Fill)).into();
            }
        }.into()
    }
}

fn rowgen<'a>(items: &'a [Arc<impl SongListItem>], rows: &mut Vec<Element<'a, EventMessage>>, items_per_row: i32, assets: Rc<AppAssets>) {
    for i in 0..(items.len() as i32 / items_per_row) {
        let mut row_songs = vec![];

        for j in 0..items_per_row {
            row_songs.push(items[((i * items_per_row) + j) as usize].view(Rc::clone(&assets)));
        }

        rows.push(Row::from_vec(row_songs)
            .spacing(10)
            .into());
    }

    let remaining_songs = items.len() as i32 % items_per_row;
    let mut row_songs = vec![];

    for i in (items.len() as i32 - remaining_songs)..items.len() as i32 {
        row_songs.push(items[i as usize].view(Rc::clone(&assets)));
    }

    for _ in remaining_songs..items_per_row {
        row_songs.push(text("").width(192).into());
    }

    rows.push(Row::from_vec(row_songs)
        .spacing(10)
        .into());
}

pub trait SongListItem {
    fn image(&self) -> Option<Image<Handle>> {
        None
    }

    fn textrow_one<'a>(&'a self) -> Option<impl text::IntoFragment<'a>> {
        None::<String>
    }

    fn textrow_two<'a>(&'a self) -> Option<impl text::IntoFragment<'a>> {
        None::<String>
    }

    fn textrow_three<'a>(&'a self) -> Option<impl text::IntoFragment<'a>> {
        None::<String>
    }

    fn event(&self) -> EventMessage {
        EventMessage::DoNothing
    }

    /// leave this implementation as default unless otherwise needed
    fn view(&self, assets: Rc<AppAssets>) -> Element<'_, EventMessage> {
        let mut col = column![];

        col = col.push(if let Some(image) = self.image() {
            image
        } else {
            img(&assets.cover)
        }.width(192));

        col = col.push(space().height(2));

        if let Some(txt) = self.textrow_one() {
            col = col.push(clip!(text(txt)
                .width(192)
                .height(23.4) // default line height (1.3) * 18
                .wrapping(text::Wrapping::None)
                .size(18)));

            col = col.push(space().height(1));
        }

        if let Some(txt) = self.textrow_two() {
            col = col.push(clip!(text(txt)
                .width(192)
                .height(15.6) // default line height (1.3) * 12
                .wrapping(text::Wrapping::WordOrGlyph)
                .size(12)));
        }

        if let Some(txt) = self.textrow_three() {
            col = col.push(clip!(text(txt)
                .width(192)
                .height(15.6) // default line height (1.3) * 12
                .wrapping(text::Wrapping::WordOrGlyph)
                .size(12)));
        }

        mouse_area(col)
            .on_press(self.event())
            .into()
    }
}
