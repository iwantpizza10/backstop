use iced::{Element, Task, Theme};

mod discord_rpc;
mod saved_state;
mod songs_view;
mod constants;
mod player;
mod queue;

use crate::discord_rpc::{DiscordRpc, DiscordRpcMode};
use crate::player::CurrentSong;
use crate::saved_state::SavedState;
use crate::saved_state::song_file_info::SongFileInfo;
use crate::songs_view::SongsViewType;
use crate::queue::Queue;

fn main() -> iced::Result {
    iced::application(BackstopApp::new, BackstopApp::update, BackstopApp::view)
        .title(BackstopApp::title)
        .theme(BackstopApp::theme)
        .window_size((1366, 768))
        .run()
}

#[derive(Default, PartialEq)]
enum PlayingState {
    Playing,
    Paused,
    #[default]
    NotPlaying,
}

#[derive(Clone)]
enum EventMessage {
    // app init stuff
    Loaded(Result<SavedState, BackstopError>),

    // library/index stuff
    AddMediaDir(String),
    RemoveMediaDir(String),
    RescanLibrary,

    // menu stuff
    ChangeMenuView(MenuView),
    ToggleQueuePeek,

    // song controls
    PlaySong(SongFileInfo),
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
    SetDiscordRpc(DiscordRpcMode),
    RemoveRpcListEntry(String),
    AddRpcListEntry(String),
}

#[derive(Clone)]
enum BackstopError {
    PlaybackError,
    IndexError,
    LoadingError,
}

#[derive(Default, Clone)]
enum MenuView {
    #[default]
    Welcome,
    SongsView(SongsViewType),
    CoverArtView,
    Settings,
}

struct AppState {
    error: Option<BackstopError>,
    menu_view: MenuView,
    saved_state: SavedState,
    playing: PlayingState,
    discord_rpc: DiscordRpc,
    queue: Option<Queue>,
    current_song: Option<CurrentSong>,
}

#[derive(Default)]
enum BackstopApp {
    #[default]
    Loading,
    Loaded(AppState)
}

impl BackstopApp {
    fn new() -> (Self, Task<EventMessage>) {
        (
            Self::Loading,
            Task::perform(SavedState::load(), EventMessage::Loaded),
        )
    }

    fn update(&mut self, message: EventMessage) -> Task<EventMessage> {
        Task::none()
    }

    fn view(&self) -> Element<'_, EventMessage> {
        iced::widget::column![
            match self {
                BackstopApp::Loading => iced::widget::text("loadin"),
                BackstopApp::Loaded(_) => iced::widget::text("loaded!!"),
            }
        ].into()
    }

    fn title(&self) -> String {
        "Backstop".to_string()
    }

    fn theme(&self) -> Option<Theme> {
        None
    }
}
