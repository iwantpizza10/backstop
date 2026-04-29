// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core::f32;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use color_from_hex::color_from_hex;
use iced::alignment::Horizontal;
use iced::keyboard::Modifiers;
use iced::theme::Palette;
use iced::widget::image::Handle;
use iced::widget::{center, column, container, mouse_area, opaque, row, space, stack, text};
use iced::{Alignment, Border, Color, Element, Event, Length, Shadow, Size, Subscription, Task, Theme, event, keyboard, time, window};

mod discord_rpc;
mod saved_state;
mod constants;
mod player;
mod queue;

mod menu_view;
mod navbar;
mod footer;
mod svg_button;

use crate::constants::{BACKSTOP_LOGO, PLACEHOLDER_COVER, SPEED_STEPS, VOLUME_DYNAMIC_RANGE_DB};
use crate::discord_rpc::{DiscordRpc, DiscordRpcMode};
use crate::footer::Footer;
use crate::menu_view::MenuView;
use crate::navbar::Navbar;
use crate::player::{CurrentSong, Player};
use crate::saved_state::SavedState;
use crate::saved_state::media_cache::{Album, Artist, CacheFilterType, CacheSortType, MediaCache};
use crate::saved_state::song_file_info::SongFileInfo;
use crate::queue::Queue;

fn main() -> iced::Result {
    iced::application(BackstopApp::new, BackstopApp::update, BackstopApp::view)
        .subscription(BackstopApp::subscriptions)
        .title(BackstopApp::title)
        .theme(BackstopApp::theme)
        .window_size((1290, 768))
        .run()
}

#[derive(Default, PartialEq, Debug, Clone, Copy)]
enum PlayingState {
    Playing,
    Paused,
    #[default]
    NotPlaying,
}

#[derive(Clone, Debug)]
enum EventMessage {
    DoNothing,
    WindowResize(Size),
    KeyboardModifiersChanged(Modifiers),
    UpdatePlaybackPosition,
    UpdateRPCTextInput(String),

    // app init stuff
    Loaded(Result<SavedState, BackstopError>),

    // library/index stuff
    TriggerAddMediaDir,
    AddMediaDir(PathBuf),
    TriggerRemoveMediaDir,
    RemoveMediaDir(String),
    TriggerRescanLibrary,
    UpdateLibrary(MediaCache),

    // menu stuff
    ChangeMenuView(MenuView),
    ChangeViewType(SongsViewType),
    ToggleSortType,
    ToggleQueuePeek,

    // song controls
    PlaySong(Arc<SongFileInfo>),
    PrevTrack,
    NextTrack,
    PlayPause,
    ToggleShuffle,
    ToggleRepeat,
    ClearQueue,

    // state settings
    SetVolume(i32),
    SetSpeed(i32),

    // discord rpc
    SetDiscordRpcMode(DiscordRpcMode),
    RemoveRpcListEntry(String),
    AddRpcListEntry,
}

#[derive(Clone, Debug)]
enum BackstopError {
    PlaybackError,
    IndexError,
    LoadingError,
}

impl BackstopError {
    /// describes when an error might've occurred
    fn when(&self) -> &str {
        match self {
            Self::PlaybackError => "during playback",
            Self::IndexError => "during indexing",
            Self::LoadingError => "during loading",
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq)]
enum SongsViewType {
    #[default]
    All,
    ArtistSelect,
    Artist(Arc<Artist>),
    AlbumSelect,
    Album(Arc<Album>),
}

#[derive(Debug)]
struct AppAssets {
    logo: iced::widget::image::Handle,
    cover: iced::widget::image::Handle,
}

impl Default for AppAssets {
    fn default() -> Self {
        Self {
            logo: Handle::from_bytes(BACKSTOP_LOGO),
            cover: Handle::from_bytes(PLACEHOLDER_COVER)
        }
    }
}

#[derive(Debug)]
struct AppState {
    menu_view: MenuView,
    saved_state: SavedState,
    playing: PlayingState,
    discord_rpc: DiscordRpc,
    queue: Option<Queue>,
    current_song: Option<CurrentSong>,
    assets: Rc<AppAssets>,
    player: Player,
    items_per_row: i32,
    sort_type: CacheSortType,
    peeking_queue: bool,
    keyboard_modifiers: Modifiers,
    rpc_text_input: String,
}

impl TryFrom<SavedState> for AppState {
    type Error = BackstopError;

    fn try_from(value: SavedState) -> Result<Self, BackstopError> {
        let player = Player::new(value.settings.get_speed(), value.settings.get_volume_linear());
        let rpc = DiscordRpc::new(&value.settings, PlayingState::default());

        if let Ok(player) = player && let Ok(rpc) = rpc {
            Ok(Self {
                menu_view: if value.settings.get_first_launch() { MenuView::Welcome } else { MenuView::SongsView(SongsViewType::All) },
                discord_rpc: rpc,
                saved_state: value,
                playing: PlayingState::default(),
                queue: None,
                current_song: None,
                assets: Rc::new(AppAssets::default()),
                player,
                items_per_row: 1,
                sort_type: CacheSortType::default(),
                peeking_queue: false,
                keyboard_modifiers: Modifiers::NONE,
                rpc_text_input: String::new(),
            })
        } else {
            Err(BackstopError::LoadingError)
        }
    }
}

#[derive(Default)]
enum BackstopApp {
    #[default]
    Loading,
    Loaded(AppState),
    Error(BackstopError)
}

impl BackstopApp {
    fn new() -> (Self, Task<EventMessage>) {
        (
            Self::Loading,
            Task::perform(SavedState::load(), EventMessage::Loaded),
        )
    }

    fn update(&mut self, message: EventMessage) -> Task<EventMessage> {
        match self {
            Self::Loading => {
                match message {
                    EventMessage::DoNothing => {},
                    EventMessage::WindowResize(_) => {},
                    EventMessage::KeyboardModifiersChanged(_) => {},
                    EventMessage::UpdatePlaybackPosition => {},

                    // app init stuff
                    EventMessage::Loaded(state) => {
                        if let Ok(state) = state && let Ok(mut loaded) = AppState::try_from(state) {
                            loaded.saved_state.media_cache.sort(CacheSortType::default());

                            *self = Self::Loaded(loaded);
                        } else {
                            *self = Self::Error(BackstopError::LoadingError);
                        }

                        return Task::done(EventMessage::WindowResize(Size { width: 1366.0, height: 768.0 }))
                    },

                    x => {
                        unimplemented!("event {:?} in context {}", x, "BackstopApp::Loading")
                    },
                }
            },
            Self::Loaded(state) => {
                match message {
                    EventMessage::DoNothing => {},

                    EventMessage::WindowResize(size) => {
                        state.items_per_row = (((size.width - 64.0) / 202.0) as i32).clamp(1, i32::MAX); // yes i would like 2147483647 songs per row thanks
                        //                       navbar width ^^      ^^^ item width + 10px (spacing)
                    },

                    EventMessage::KeyboardModifiersChanged(mods) => {
                        state.keyboard_modifiers = mods;
                    },

                    EventMessage::UpdatePlaybackPosition => {
                        if state.player.song_done_or_empty() {
                            let song;

                            if state.saved_state.settings.get_repeat() && let Some(cur) = &state.current_song {
                                song = Some(Arc::clone(&cur.file_info));
                            } else if let Some(q) = &mut state.queue {
                                if let Some(sog) = q.next_song() {
                                    song = Some(sog);
                                } else {
                                    song = None;
                                }
                            } else {
                                song = None
                            }

                            if let Some(song) = song {
                                if let Err(err) = state.player.play_song(Arc::clone(&song)) {
                                    *self = BackstopApp::Error(err);
                                } else {
                                    state.playing = PlayingState::Playing;

                                    let cur_song = CurrentSong {
                                        start_time: Utc::now(),
                                        file_info: song,
                                    };

                                    state.current_song = Some(cur_song.clone());
                                    state.discord_rpc.update_playing_state(state.playing);
                                    state.discord_rpc.play_song(cur_song);
                                }
                            }
                        }
                    },

                    EventMessage::UpdateRPCTextInput(text) => {
                        state.rpc_text_input = text;
                    }

                    // library/index stuff

                    EventMessage::TriggerAddMediaDir => {
                        return Task::perform(async {
                            rfd::AsyncFileDialog::new().pick_folder().await
                        }, |x| x.map_or(EventMessage::DoNothing, |x| EventMessage::AddMediaDir(x.path().to_path_buf())));
                    },

                    EventMessage::AddMediaDir(dir) => {
                        state.saved_state.settings.add_media_directory(dir);

                        let settings_two = state.saved_state.settings.clone();

                        return Task::future(async move {
                            let _ = settings_two.clone().save().await;

                            EventMessage::DoNothing
                        });
                    },

                    // triggerremovemediadir
                    //  removemediadir

                    EventMessage::TriggerRescanLibrary => {
                        let dirs = state.saved_state.settings.get_media_directories().clone();

                        return Task::perform(async move {
                            MediaCache::from_scan(dirs).await
                        }, |x| x.map_or(EventMessage::DoNothing, |x| EventMessage::UpdateLibrary(x)));
                    },

                    EventMessage::UpdateLibrary(cache) => {
                        state.saved_state.media_cache = cache;
                    },

                    // menu stuff

                    EventMessage::ChangeMenuView(view) => {
                        state.menu_view = view;
                    },

                    EventMessage::ChangeViewType(view) => {
                        if let SongsViewType::Artist(artist) = &view {
                            let artist = Arc::clone(&artist);

                            state.saved_state.media_cache.filter(CacheFilterType::Artist(artist));
                        }

                        if let SongsViewType::Album(album) = &view {
                            let album = Arc::clone(&album);

                            state.saved_state.media_cache.filter(CacheFilterType::Album(album));
                        }

                        if let SongsViewType::All = &view {
                            state.saved_state.media_cache.filter(CacheFilterType::None);
                        }

                        state.menu_view = MenuView::SongsView(view);
                    },

                    EventMessage::ToggleSortType => {
                        if let MenuView::SongsView(SongsViewType::Album(_)) = state.menu_view {
                            // not sorting anything here because it's in order by track number
                            // i feel like that makes most sense to have as an immutable thing
                            // + i don't feel like making other sorting work there :)
                        } else {
                            state.sort_type = match state.sort_type {
                                CacheSortType::ArtistAlphabetical => CacheSortType::TitleAlphabetical,
                                CacheSortType::TitleAlphabetical => CacheSortType::ArtistAlphabetical,
                            };

                            state.saved_state.media_cache.sort(state.sort_type.clone());
                        }
                    },

                    EventMessage::ToggleQueuePeek => {
                        state.peeking_queue = !state.peeking_queue;
                    },

                    // song controls

                    EventMessage::PlaySong(song) => {
                        let play_song_now;
                        
                        if state.keyboard_modifiers.command() {
                            if state.keyboard_modifiers.shift() {
                                // ctrl+shift+click - inserts song as next song in queue, makes new queue if needed

                                if let Some(q) = &mut state.queue {
                                    q.insert_next(Arc::clone(&song));
                                    play_song_now = false;
                                } else {
                                    state.queue = Queue::from_vec(&vec![
                                        Arc::clone(&song),
                                    ], Arc::clone(&song));

                                    play_song_now = true;
                                }
                            } else {
                                // ctrl+click - inserts song as last song in queue, makes new queue if needed

                                if let Some(q) = &mut state.queue {
                                    q.append_song(Arc::clone(&song));
                                    play_song_now = false;
                                } else {
                                    state.queue = Queue::from_vec(&vec![
                                        Arc::clone(&song),
                                    ], Arc::clone(&song));

                                    play_song_now = true;
                                }
                            }
                        } else {
                            // click - plays song immediately, creating new queue from all visible items & shuffling if needed

                            state.queue = Queue::from_vec(state.saved_state.media_cache.songs(), Arc::clone(&song));

                            if let Some(queue) = &mut state.queue && state.saved_state.settings.get_shuffle() {
                                queue.shuffle();
                            }

                            play_song_now = true;
                        }

                        if play_song_now {
                            if let Err(err) = state.player.play_song(Arc::clone(&song)) {
                                *self = BackstopApp::Error(err);
                            } else {
                                state.playing = PlayingState::Playing;

                                let cur_song = CurrentSong {
                                    start_time: Utc::now(),
                                    file_info: song,
                                };

                                state.current_song = Some(cur_song.clone());
                                state.discord_rpc.update_playing_state(state.playing);
                                state.discord_rpc.play_song(cur_song);
                            }
                        }
                    },

                    EventMessage::NextTrack | EventMessage::PrevTrack => {
                        if let Some(q) = &mut state.queue && let Some(song) = match message {
                            EventMessage::NextTrack => {q.next_song()},
                            EventMessage::PrevTrack => {q.previous_song()},
                            _ => { panic!("literally how") } 
                        } {
                            if let Err(err) = state.player.play_song(Arc::clone(&song)) {
                                *self = BackstopApp::Error(err);
                            } else {
                                state.playing = PlayingState::Playing;

                                let cur_song = CurrentSong {
                                    start_time: Utc::now(),
                                    file_info: song,
                                };

                                state.current_song = Some(cur_song.clone());
                                state.discord_rpc.update_playing_state(state.playing);
                                state.discord_rpc.play_song(cur_song);
                            }
                        }
                    },

                    EventMessage::PlayPause => {
                        match state.playing {
                            PlayingState::NotPlaying => {},
                            PlayingState::Paused => {
                                state.playing = PlayingState::Playing;
                                state.player.resume();
                            },
                            PlayingState::Playing => {
                                state.playing = PlayingState::Paused;
                                state.player.pause();
                            }
                        }

                        state.discord_rpc.update_playing_state(state.playing);
                    },

                    EventMessage::ToggleShuffle => {
                        state.saved_state.settings.toggle_shuffle();

                        if let Some(q) = &mut state.queue {
                            if state.saved_state.settings.get_shuffle() {
                                q.shuffle();
                            } else {
                                q.unshuffle();
                            }
                        }
                    },

                    EventMessage::ToggleRepeat => {
                        state.saved_state.settings.toggle_repeat();
                    },

                    // state settings

                    EventMessage::SetVolume(vol_step) => {
                        state.saved_state.settings.set_volume_db((vol_step - VOLUME_DYNAMIC_RANGE_DB) as f32);
                        state.player.set_volume(state.saved_state.settings.get_volume_linear());

                        let settings_two = state.saved_state.settings.clone();

                        return Task::future(async move {
                            let _ = settings_two.clone().save().await;

                            EventMessage::DoNothing
                        });
                    },

                    EventMessage::SetSpeed(speed_step) => {
                        state.saved_state.settings.set_speed(speed_step as f32 / (SPEED_STEPS / 2) as f32);
                        state.player.set_speed(state.saved_state.settings.get_speed());

                        let settings_two = state.saved_state.settings.clone();

                        return Task::future(async move {
                            let _ = settings_two.clone().save().await;

                            EventMessage::DoNothing
                        });
                    },

                    EventMessage::ClearQueue => {
                        let _ = state.discord_rpc.clear_rpc();
                        state.queue = None;
                        state.current_song = None;
                        state.player.clear();
                    },

                    // discord rpc

                    EventMessage::SetDiscordRpcMode(mode) => {
                        state.saved_state.settings.set_rpc_mode(mode);
                        state.discord_rpc.update_rpc_mode(mode);

                        let settings_two = state.saved_state.settings.clone();

                        return Task::future(async move {
                            let _ = settings_two.clone().save().await;

                            EventMessage::DoNothing
                        });
                    },

                    EventMessage::RemoveRpcListEntry(item) => {                        
                        state.saved_state.settings.remove_rpc_list(item);
                        state.discord_rpc.update_rpc_list(state.saved_state.settings.get_rpc_list());

                        let settings_two = state.saved_state.settings.clone();

                        return Task::future(async move {
                            let _ = settings_two.clone().save().await;

                            EventMessage::DoNothing
                        });
                    }

                    EventMessage::AddRpcListEntry => {
                        state.saved_state.settings.add_rpc_list(state.rpc_text_input.clone());
                        state.discord_rpc.update_rpc_list(state.saved_state.settings.get_rpc_list());
                        state.rpc_text_input = String::new();

                        let settings_two = state.saved_state.settings.clone();

                        return Task::future(async move {
                            let _ = settings_two.clone().save().await;

                            EventMessage::DoNothing
                        });
                    },

                    x => {
                        todo!("event {:?} in context {}", x, "BackstopApp::Loaded")
                    },
                }
            },
            Self::Error(err) => {
                match message {
                    EventMessage::WindowResize(_) => {},

                    x => unimplemented!("event {:?} in context {}", x, format!("BackstopApp::Error({:?})", err)),
                }
            },
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, EventMessage> {
        match self {
            BackstopApp::Loading => {
                text("Loading...")
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .size(36)
            },
            BackstopApp::Loaded(state) => {
                let content = column![
                    row![
                        Navbar::view(state),
                        state.menu_view.view(state)
                    ],
                    Footer::view(state),
                ];

                if state.peeking_queue {
                    let mut content_col = column![
                        text("Queue (Next 3)")
                            .size(36),
                        space().height(4)
                    ].spacing(4)
                    .align_x(Horizontal::Center);

                    if let Some(q) = &state.queue {
                        let peek = q.peek()
                            .iter()
                            .filter(|x| if let Some(_) = x { true } else { false })
                            .map(|x| x.clone().expect("these had better be some"))
                            .collect::<Vec<_>>();

                        for i in peek {
                            content_col = content_col.push(text!("{} - {}", i.artist(), i.title()));
                        }
                    }

                    return stack![
                        content,
                        opaque(
                            mouse_area(center(container(content_col).style(|_| {
                                container::Style {
                                    text_color: Some(Color::WHITE),
                                    background: Some(color_from_hex!("#170f37").into()),
                                    border: Border::default().color(Color::WHITE).width(1).rounded(15),
                                    shadow: Shadow::default(),
                                    snap: false,
                                }
                            }).padding(16)).style(|_| {
                                container::Style {
                                    background: Some(Color {
                                        a: 0.75,
                                        ..Color::BLACK
                                    }.into()),
                                    ..container::Style::default()
                                }
                            }))
                            .on_press(EventMessage::ToggleQueuePeek)
                        )
                    ].into();
                } else {
                    return content.into();
                }
            },
            BackstopApp::Error(error) => {
                text!("An error occurred {}!", error.when())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .size(36)
            },
        }.into()
    }

    fn subscriptions(&self) -> Subscription<EventMessage> {
        Subscription::batch(vec![
            event::listen_with(|event, _, _| {
                match event {
                    Event::Window(window::Event::Resized(size)) => {
                        Some(EventMessage::WindowResize(size))
                    },
                    Event::Keyboard(keyboard::Event::ModifiersChanged(mods)) => {
                        Some(EventMessage::KeyboardModifiersChanged(mods))
                    },
                    _ => None,
                }
            }),
            time::every(Duration::from_millis(125)).map(|_| EventMessage::UpdatePlaybackPosition),
        ])
    }

    fn title(&self) -> String {
        "Backstop".to_string()
    }

    fn theme(&self) -> Option<Theme> {
        Some(Theme::custom("Backstop Theme", Palette {
            background: color_from_hex!("#0b071b"),
            text: color_from_hex!("#ffffff"),
            primary: color_from_hex!("#4b0fa3"),
            success: color_from_hex!("#7221ea"),
            warning: Color::from_rgb8(255,255,255),
            danger: Color::from_rgb8(255,255,255),
        }))
    }
}
