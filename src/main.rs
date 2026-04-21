// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core::f32;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use chrono::Utc;
use iced::widget::image::Handle;
use iced::widget::{column, row, text};
use iced::{Alignment, Element, Event, Length, Size, Subscription, Task, Theme, event, window};

mod discord_rpc;
mod saved_state;
mod constants;
mod player;
mod queue;

mod menu_view;
mod navbar;

use crate::constants::{BACKSTOP_LOGO, PLACEHOLDER_COVER};
use crate::discord_rpc::{DiscordRpc, DiscordRpcMode};
use crate::menu_view::MenuView;
use crate::navbar::Navbar;
use crate::player::{CurrentSong, Player};
use crate::saved_state::SavedState;
use crate::saved_state::media_cache::{Album, Artist, CacheFilterType, MediaCache};
use crate::saved_state::song_file_info::SongFileInfo;
use crate::queue::Queue;

fn main() -> iced::Result {
    iced::application(BackstopApp::new, BackstopApp::update, BackstopApp::view)
        .subscription(BackstopApp::subscriptions)
        .title(BackstopApp::title)
        .theme(BackstopApp::theme)
        .window_size((1366, 768))
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
    ToggleQueuePeek,

    // song controls
    PlaySong(Arc<SongFileInfo>),
    AppendToQueue(Arc<SongFileInfo>),
    NextInQueue(Arc<SongFileInfo>),
    PrevTrack,
    NextTrack,
    PlayPause,
    ToggleShuffle,
    ToggleRepeat,

    // state settings
    SetVolume(f32),
    SetSpeed(f32),

    // discord rpc
    ClearDiscordRpc,
    SetDiscordRpcMode(DiscordRpcMode),
    RemoveRpcListEntry(String),
    AddRpcListEntry(String),
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

#[derive(Default, Clone, Debug)]
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
    error: Option<BackstopError>,
    menu_view: MenuView,
    saved_state: SavedState,
    playing: PlayingState,
    discord_rpc: DiscordRpc,
    queue: Option<Queue>,
    current_song: Option<CurrentSong>,
    assets: Rc<AppAssets>,
    player: Player,
    items_per_row: i32,
}

impl TryFrom<SavedState> for AppState {
    type Error = BackstopError;

    fn try_from(value: SavedState) -> Result<Self, BackstopError> {
        let player = Player::new(value.settings.get_speed(), value.settings.get_volume_linear());
        let rpc = DiscordRpc::new(&value.settings, PlayingState::default());

        if let Ok(player) = player && let Ok(rpc) = rpc {
            Ok(Self {
                error: None,
                menu_view: if value.settings.get_first_launch() { MenuView::Welcome } else { MenuView::SongsView(SongsViewType::All) },
                discord_rpc: rpc,
                saved_state: value,
                playing: PlayingState::default(),
                queue: None,
                current_song: None,
                assets: Rc::new(AppAssets::default()),
                player,
                items_per_row: 1,
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

                    // app init stuff
                    EventMessage::Loaded(state) => {
                        if let Ok(state) = state && let Ok(loaded) = AppState::try_from(state) {
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
                    }

                    // library/index stuff

                    EventMessage::TriggerAddMediaDir => {
                        return Task::perform(async {
                            rfd::AsyncFileDialog::new().pick_folder().await
                        }, |x| x.map_or(EventMessage::DoNothing, |x| EventMessage::AddMediaDir(x.path().to_path_buf())));
                    },

                    EventMessage::AddMediaDir(dir) => {
                        state.saved_state.settings.add_media_directory(dir);
                    },

                    // todo: triggerremovemediadir
                    // todo: removemediadir

                    EventMessage::TriggerRescanLibrary => {
                        let dirs = state.saved_state.settings.get_media_directories();

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
                        if let MenuView::SongsView(_) = state.menu_view {
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
                        }
                    }

                    // todo: togglequeuepeek

                    // song controls

                    EventMessage::PlaySong(song) => {
                        if let Err(err) = state.player.play_song(Arc::clone(&song)) {
                            *self = BackstopApp::Error(err);
                        } else {
                            state.playing = PlayingState::Playing;
                            state.queue = Queue::from_vec(state.saved_state.media_cache.songs(), Arc::clone(&song));

                            if let Some(queue) = &mut state.queue && state.saved_state.settings.get_shuffle() {
                                queue.shuffle();
                            }

                            let cur_song = CurrentSong {
                                duration: state.player.get_duration(),
                                start_time: Utc::now(),
                                file_info: song,
                            };

                            state.current_song = Some(cur_song.clone());
                            state.discord_rpc.update_playing_state(state.playing);
                            state.discord_rpc.play_song(cur_song);
                        }
                    },

                    // todo: appendtoqueue
                    // todo: nextinqueue
                    // todo: prevtrack
                    // todo: nexttrack
                    // todo: playpause
                    // todo: toggleshuffle
                    // todo: togglerepeat
                    
                    // state settings
                    // todo: setvolume
                    // todo: setspeed

                    // discord rpc
                    // todo: cleardiscordrpc
                    // todo: setdiscordrpcmode
                    // todo: removerpclistentry
                    // todo: addrpclistentry

                    x => {
                        todo!("event {:?} in context {}", x, "BackstopApp::Loaded")
                    },
                }
            },
            Self::Error(x) => {
                unimplemented!("event {:?} in context {}", message, format!("BackstopApp::Error({:?})", x))
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
                return column![
                    row![
                        Navbar::view(state),
                        state.menu_view.view(state)
                    ],
                    // todo: footer
                ].into()
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
        event::listen_with(|event, _, _| {
            match event {
                Event::Window(window::Event::Resized(size)) => {
                    Some(EventMessage::WindowResize(size))
                },
                _ => None,
            }
        })
    }

    fn title(&self) -> String {
        "Backstop".to_string()
    }

    fn theme(&self) -> Option<Theme> {
        None
    }
}
