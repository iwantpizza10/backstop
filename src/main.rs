#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, error::Error, fs::File, path::PathBuf, rc::Rc, time::Duration};
use backstop::{cache::{self, CacheState, MediaCache, SongFileInfo}, constants, queue::SongsQueue, settings::{BackstopSettings, RichPresenceType}};
use chrono::Utc;
use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity::{Activity, ActivityType, Assets, Button, StatusDisplayType, Timestamps}};
use rodio::{Decoder, DeviceSinkBuilder, Player};
use slint::{Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, Timer, TimerMode, VecModel};
use async_compat::Compat;

const PLACEHOLDER_COVER: &[u8] = include_bytes!("../ui/res/cover_placeholder.png");
const SECONGS_BACKSKIP_THRESHOLD: i32 = 5;
const DISCORD_APP_ID: &str = "1483067786589765702";

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let _ = backstop::init_config_dirs(); // not gonna ? this cause its not technically needed to run
    let ui = BackstopWindow::new()?;

    let media_cache = Rc::new(RefCell::new(cache::load_else_dead().unwrap_or_else(|_| MediaCache::dead())));
    let media_cache_model: Rc<VecModel<LibrarySong>> = Rc::new(VecModel::from(vec![]));
    let media_cache_rc = ModelRc::from(media_cache_model);

    let settings = Rc::new(RefCell::new(BackstopSettings::load_else_new()));
    let mut audio_device_handle = DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    audio_device_handle.log_on_drop(false);
    let audio_player = Rc::new(Player::connect_new(&audio_device_handle.mixer()));
    
    let songs_queue = Rc::new(RefCell::new(SongsQueue::new()));
    
    audio_player.set_volume(settings.borrow().volume_linear());
    audio_player.set_speed(settings.borrow().playback_speed());

    let media_dirs_temp = settings.borrow().media_directories().iter()
        .map(|x| x.to_string_lossy().to_string().into())
        .collect::<Vec<SharedString>>();
    let media_dirs_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(media_dirs_temp));
    let media_dirs_rc = ModelRc::from(media_dirs_model);

    let rpc_list_temp = settings.borrow().rich_presence_list().iter()
        .map(|x| x.into())
        .collect::<Vec<SharedString>>();
    let rpc_list_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(rpc_list_temp));
    let rpc_list_rc = ModelRc::from(rpc_list_model);

    let rpc_client = Rc::new(RefCell::new(DiscordIpcClient::new(DISCORD_APP_ID)));

    if *settings.borrow().rich_presence_type() != RichPresenceType::Disabled {
        let _ = rpc_client.borrow_mut().connect();
    }

    media_cache.borrow_mut().sort(settings.borrow().sort_type());

    if let Err(_) = load_cache_to_model(&media_cache.borrow(), media_cache_rc.clone()) {
        ui.set_menustate(MenuState::LoadingError);
    }

    ui.set_volume(settings.borrow().volume());
    ui.set_playback_speed(settings.borrow().playback_speed());
    ui.set_menustate(if settings.borrow().is_first_launch() { MenuState::Onboarding } else { MenuState::Welcome });
    ui.set_media_library(media_cache_rc.clone());
    ui.set_media_directories(media_dirs_rc.clone());
    ui.set_rpc_list(rpc_list_rc.clone());
    ui.set_rpc_type(match settings.borrow().rich_presence_type() {
        RichPresenceType::Blacklist => UIRichPresenceType::Blacklist,
        RichPresenceType::Whitelist => UIRichPresenceType::Whitelist,
        RichPresenceType::Disabled => UIRichPresenceType::Disabled,
    });
    settings.borrow_mut().set_is_first_launch(false);
    let _ = settings.borrow().save_to_disk();

    // keep this below the ui.set_menustate call so it'll override that if it needs to
    if media_cache.borrow().state() == &CacheState::Dead {
        let media_cache = Rc::clone(&media_cache);
        let settings = Rc::clone(&settings);
        let ui = ui.as_weak().unwrap();

        slint::spawn_local(Compat::new(async move {
            ui.set_menustate(MenuState::Reindexing);
            let mut media_cache = media_cache.borrow_mut();

            if let Ok(_) = media_cache.rescan_library(settings.borrow().media_directories()).await {
                if let Err(_) = media_cache.save_to_disk() {
                    ui.set_menustate(MenuState::IndexingError);
                } else {
                    ui.set_menustate(if settings.borrow().is_first_launch() { MenuState::Onboarding } else { MenuState::Welcome });
                }
            } else {
                ui.set_menustate(MenuState::IndexingError);
            }
        })).unwrap();
    }

    let ui_time_updater = Timer::default();
    ui_time_updater.start(TimerMode::Repeated, Duration::from_millis(250), {
        let ui = ui.as_weak().unwrap();
        let audio_player = Rc::clone(&audio_player);
        let queue = Rc::clone(&songs_queue);
        let last_check = Rc::new(RefCell::new(Duration::ZERO));
        let rpc_client = Rc::clone(&rpc_client);
        let settings = Rc::clone(&settings);

        move || {
            ui.set_song_position(audio_player.get_pos().as_secs() as i32);

            if queue.borrow().songs().len() != 0 && *last_check.borrow() == audio_player.get_pos() && !ui.get_paused() && ui.get_playing() {
                let song;
                let mut should_play = true;
                let mut songs_queue = queue.borrow_mut();

                if ui.get_repeat() {
                    if let Some(sog) = songs_queue.current_song().cloned() {
                        song = sog;
                    } else {
                        should_play = false;
                        song = SongFileInfo::dummy();
                    }
                } else {
                    if let Some(sog) = songs_queue.next_song().cloned() {
                        song = sog;
                    } else {
                        if let Some(sog) = songs_queue.current_song().cloned() {
                            song = sog;
                        } else {
                            should_play = false;
                            song = SongFileInfo::dummy();
                        }
                    }
                }

                if should_play {
                    let song = library_paranormal_convert(&song);

                    let _ = set_discord_rpc(&song, Rc::clone(&rpc_client), Rc::clone(&settings));

                    if let Err(_) = play_song(Rc::clone(&audio_player), ui.as_weak().unwrap(), song, songs_queue.next_3()) {
                        ui.set_menustate(MenuState::PlaybackError);
                        ui.set_playing(false);
                    }
                }
            }

            *last_check.borrow_mut() = audio_player.get_pos();
        }
    });

    ui.on_play_song({
        let ui = ui.as_weak().unwrap();
        let audio_player = Rc::clone(&audio_player);
        let queue = Rc::clone(&songs_queue);
        let media_cache = Rc::clone(&media_cache);
        let rpc_client = Rc::clone(&rpc_client);
        let settings = Rc::clone(&settings);

        move |song| {
            let cur_song_idx = media_cache.borrow().songs().iter()
                .position(|x| *x.filepath == *song.path)
                .expect("chose a song somehow that doesnt exit (???)") as i32;
            *queue.borrow_mut() = SongsQueue::create_from_cache(&*media_cache.borrow(), cur_song_idx);

            if ui.get_shuffle() {
                queue.borrow_mut().shuffle();

                let next_3_vec = queue.borrow().next_3().iter()
                    .map(|x| x.into())
                    .collect::<Vec<_>>();

                ui.set_next_three(ModelRc::from(Rc::new(VecModel::from(next_3_vec))));
            }

            let _ = set_discord_rpc(&song, Rc::clone(&rpc_client), Rc::clone(&settings));

            if let Err(_) = play_song(Rc::clone(&audio_player), ui.as_weak().unwrap(), song, queue.borrow().next_3()) {
                ui.set_menustate(MenuState::PlaybackError);
                ui.set_playing(false);
            }
        }
    });

    ui.on_rescan_library({
        let media_cache = Rc::clone(&media_cache);
        let settings = Rc::clone(&settings);
        let ui = ui.as_weak().unwrap();

        move || {
            let media_cache = Rc::clone(&media_cache);
            let settings = Rc::clone(&settings);
            let media_cache_rc = media_cache_rc.clone();
            let ui = ui.as_weak().unwrap();

            slint::spawn_local(Compat::new(async move {
                let original_menu_state = ui.get_menustate();
                let mut media_cache = media_cache.borrow_mut();

                ui.set_menustate(MenuState::Reindexing);

                if let Ok(_) = media_cache.rescan_library(settings.borrow().media_directories()).await {
                    if let Err(_) = media_cache.save_to_disk() {
                        ui.set_menustate(MenuState::IndexingError);

                        return;
                    }
                } else {
                    ui.set_menustate(MenuState::IndexingError);

                    return;
                }

                media_cache.sort(settings.borrow().sort_type());

                if let Err(_) = load_cache_to_model(&media_cache, media_cache_rc) {
                    ui.set_menustate(MenuState::LoadingError);
                } else {
                    ui.set_menustate(original_menu_state);
                }
            })).unwrap();
        }
    });

    ui.on_pause({
        let ui = ui.as_weak().unwrap();
        let audio_player = Rc::clone(&audio_player);

        move || {
            ui.set_paused(true);

            audio_player.pause();
        }
    });

    ui.on_resume({
        let ui = ui.as_weak().unwrap();
        let audio_player = Rc::clone(&audio_player);

        move || {
            ui.set_paused(false);
            audio_player.play();
        }
    });

    ui.on_set_volume({
        let ui = ui.as_weak().unwrap();
        let audio_player = Rc::clone(&audio_player);
        let settings = Rc::clone(&settings);

        move |vol_db| {
            let mut settings = settings.borrow_mut();

            settings.set_volume(vol_db);
            let _ = settings.save_to_disk();
            audio_player.set_volume(settings.volume_linear());
            ui.set_volume(vol_db);
        }
    });

    ui.on_add_media_directory({
        let settings = Rc::clone(&settings);
        let ui = ui.as_weak().unwrap();

        move || {
            let ui = ui.as_weak().unwrap();
            let settings = Rc::clone(&settings);

            slint::spawn_local(Compat::new(async move {
                let dirs_rc = ui.get_media_directories();
                let media_dirs: &VecModel<SharedString> = dirs_rc.as_any().downcast_ref().expect("media_dirs_rc downcast should downcast properly");
                let original_menu_state = ui.get_menustate(); // originally orregano_menu_state

                ui.set_menustate(MenuState::AddingDir);

                let dir = rfd::AsyncFileDialog::new()
                    .pick_folder().await;

                if let Some(dir) = dir {
                    let dir_path = dir.path().to_path_buf();
                    let mut settings = settings.borrow_mut();

                    media_dirs.push(dir_path.to_string_lossy().to_string().into());
                    settings.add_media_directory(dir_path);
                    let _ = settings.save_to_disk();

                    ui.set_menustate(original_menu_state);
                }

                ui.set_menustate(original_menu_state);
            })).unwrap();
        }
    });

    ui.on_remove_media_directory({
        let settings = Rc::clone(&settings);
        let ui = ui.as_weak().unwrap();

        move |dir| {
            let dirs_rc = ui.get_media_directories();
            let media_dirs: &VecModel<SharedString> = dirs_rc.as_any().downcast_ref().expect("media_dirs_rc downcast should downcast properly");
            let dirs: Vec<_> = media_dirs.iter()
                .filter(|x| *x != dir)
                .collect();
            let mut settings = settings.borrow_mut();

            media_dirs.set_vec(dirs);
            settings.remove_media_directory(PathBuf::from(dir.to_string()));
            let _ = settings.save_to_disk();
        }
    });

    ui.on_skip_forward({
        let ui = ui.as_weak().unwrap();
        let audio_player = Rc::clone(&audio_player);
        let queue = Rc::clone(&songs_queue);
        let rpc_client = Rc::clone(&rpc_client);
        let settings = Rc::clone(&settings);

        move || {
            let song;
            let mut should_play = true;

            let mut songs_queue = queue.borrow_mut();

            if let Some(sog) = songs_queue.next_song().cloned() {
                song = sog;
            } else {
                if let Some(sog) = songs_queue.current_song().cloned() {
                    song = sog;
                } else {
                    should_play = false;
                    song = SongFileInfo::dummy();
                }
            }

            if should_play {
                let song = library_paranormal_convert(&song);

                let _ = set_discord_rpc(&song, Rc::clone(&rpc_client), Rc::clone(&settings));

                if let Err(_) = play_song(Rc::clone(&audio_player), ui.as_weak().unwrap(), song, songs_queue.next_3()) {
                    ui.set_menustate(MenuState::PlaybackError);
                    ui.set_playing(false);
                }
            }
        }
    });

    ui.on_skip_backward({
        let ui = ui.as_weak().unwrap();
        let audio_player = Rc::clone(&audio_player);
        let queue = Rc::clone(&songs_queue);
        let rpc_client = Rc::clone(&rpc_client);
        let settings = Rc::clone(&settings);

        move || {
            let song;
            let mut should_play = true;

            if ui.get_song_position() < SECONGS_BACKSKIP_THRESHOLD {
                let mut songs_queue = queue.borrow_mut();

                if let Some(sog) = songs_queue.prev_song().cloned() {
                    song = sog;
                } else {
                    if let Some(sog) = songs_queue.current_song().cloned() {
                        song = sog;
                    } else {
                        should_play = false;
                        song = SongFileInfo::dummy();
                    }
                }
            } else {
                let songs_queue = queue.borrow();

                if let Some(sog) = songs_queue.current_song().cloned() {
                    song = sog;
                } else {
                    should_play = false;
                    song = SongFileInfo::dummy();
                }
            }

            if should_play {
                let song = library_paranormal_convert(&song);

                let _ = set_discord_rpc(&song, Rc::clone(&rpc_client), Rc::clone(&settings));

                if let Err(_) = play_song(Rc::clone(&audio_player), ui.as_weak().unwrap(), song, queue.borrow().next_3()) {
                    ui.set_menustate(MenuState::PlaybackError);
                    ui.set_playing(false);
                }
            }
        }
    });

    ui.on_toggle_shuffle({
        let ui = ui.as_weak().unwrap();
        let queue = Rc::clone(&songs_queue);

        move || {
            let mut queue = queue.borrow_mut();

            if ui.get_shuffle() {
                queue.unshuffle();
                ui.set_shuffle(false);
            } else {
                queue.shuffle();
                ui.set_shuffle(true);
            }

            let next_3_vec = queue.next_3().iter()
                .map(|x| x.into())
                .collect::<Vec<_>>();

            ui.set_next_three(ModelRc::from(Rc::new(VecModel::from(next_3_vec))));
        }
    });

    ui.on_toggle_repeat({
        let ui = ui.as_weak().unwrap();

        move || {
            ui.set_repeat(!ui.get_repeat());
        }
    });

    ui.on_set_playback_speed({
        let ui = ui.as_weak().unwrap();
        let audio_player = Rc::clone(&audio_player);
        let settings = Rc::clone(&settings);

        move |speed| {
            let mut settings = settings.borrow_mut();

            settings.set_playback_speed(speed);
            audio_player.set_speed(speed);
            ui.set_playback_speed(speed);
            let _ = settings.save_to_disk();
        }
    });

    ui.on_add_rpc_list_item({
        let ui = ui.as_weak().unwrap();
        let settings = Rc::clone(&settings);

        move |item| {
            let list_rc = ui.get_rpc_list();
            let rpc_list: &VecModel<SharedString> = list_rc.as_any().downcast_ref().expect("rpc_list_rc downcast should downcast properly");

            settings.borrow_mut().add_rich_presence_list(item.to_string());
            rpc_list.push(item);
            let _ = settings.borrow().save_to_disk();
        }
    });

    ui.on_remove_rpc_list_item({
        let ui = ui.as_weak().unwrap();
        let settings = Rc::clone(&settings);

        move |item| {
            let list_rc = ui.get_rpc_list();
            let rpc_list: &VecModel<SharedString> = list_rc.as_any().downcast_ref().expect("rpc_list_rc downcast should downcast properly");
            let items: Vec<_> = rpc_list.iter()
                .filter(|x| *x != item)
                .collect();
            let mut settings = settings.borrow_mut();

            rpc_list.set_vec(items);
            settings.remove_rich_presence_list(item.to_string());
            let _ = settings.save_to_disk();
        }
    });

    ui.on_set_rpc_type({
        let ui = ui.as_weak().unwrap();
        let settings = Rc::clone(&settings);
        let rpc_client = Rc::clone(&rpc_client);

        move |rpc_type| {
            ui.set_rpc_type(rpc_type);
            settings.borrow_mut().set_rich_presence_type(match rpc_type {
                UIRichPresenceType::Blacklist => RichPresenceType::Blacklist,
                UIRichPresenceType::Whitelist => RichPresenceType::Whitelist,
                UIRichPresenceType::Disabled => RichPresenceType::Disabled,
            });

            let _ = settings.borrow().save_to_disk();

            if rpc_type != UIRichPresenceType::Disabled {
                let _ = rpc_client.borrow_mut().connect();
            }
        }
    });

    ui.on_clear_rpc({
        let rpc_client = Rc::clone(&rpc_client);

        move || {
            let _ = rpc_client.borrow_mut().clear_activity();
        }
    });

    ui.run()?;

    Ok(())
}

fn load_cache_to_model(media_cache: &MediaCache, media_cache_rc: ModelRc<LibrarySong>) -> Result<(), Box<dyn Error>> {
    let test: &VecModel<LibrarySong> = media_cache_rc.as_any().downcast_ref().expect("media_cache_rc downcast should downcast properly");
    test.clear();

    for i in media_cache.songs() {
        let mut path = constants::conf_dir();
        path.push("covers");
        let cover_image;

        if let Some(cover_path) = i.cover.clone() {
            path.push(cover_path);

            if let Ok(image) = image::open(path) {
                cover_image = image.into_rgba8();
            } else {
                cover_image = image::load_from_memory(PLACEHOLDER_COVER)
                    .expect("placeholder coverart should process correctly")
                    .into_rgba8();
            }
        } else {
            cover_image = image::load_from_memory(PLACEHOLDER_COVER)
                .expect("placeholder coverart should process correctly")
                .into_rgba8();
        }

        let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(cover_image.as_raw(), cover_image.width(), cover_image.height());
        let cover = Image::from_rgba8(buffer);

        let lsong = LibrarySong {
            album: i.album.clone().unwrap_or("".to_string()).into(),
            album_artist: i.album_artist.clone().unwrap_or("".to_string()).into(),
            artist: i.artist.clone().into(),
            length: i.length.as_secs() as i32,
            path: i.filepath.to_string_lossy().to_string().into(),
            title: i.title.clone().into(),
            track_number: i.track_number.unwrap_or(-1),
            year: i.year.unwrap_or(-1),
            cover_path: i.cover.clone().unwrap_or("?".to_string()).into(),
            cover
        };

        test.push(lsong);
    }

    Ok(())
}

fn play_song(audio_player: Rc<Player>, ui: BackstopWindow, song: LibrarySong, next_3_vec: Vec<String>) -> Result<(), Box<dyn Error>> {
    let song_path = song.path.clone();
    let next_3_vec = next_3_vec.iter()
        .map(|x| x.into())
        .collect::<Vec<_>>();

    ui.set_current_song(song);
    ui.set_playing(true);
    ui.set_paused(false);
    ui.set_song_position(0);
    ui.set_next_three(ModelRc::from(Rc::new(VecModel::from(next_3_vec))));

    let file = File::open(song_path)?;
    let source = Decoder::try_from(file)?;

    audio_player.clear();
    audio_player.append(source);
    audio_player.play();

    Ok(())
}

fn set_discord_rpc(song: &LibrarySong, client: Rc<RefCell<DiscordIpcClient>>, settings: Rc<RefCell<BackstopSettings>>) -> Result<(), discord_rich_presence::error::Error> {
    let settings = settings.borrow();
    let list = settings.rich_presence_list();
    let rpc_type = settings.rich_presence_type();
    let mut can_set = false;

    match rpc_type {
        RichPresenceType::Blacklist => {
            can_set = true;

            for i in list {
                if song.artist.to_lowercase().contains(&i.to_lowercase()) || song.title.to_lowercase().contains(&i.to_lowercase()) {
                    can_set = false;
                }
            }
        },
        RichPresenceType::Whitelist => {
            for i in list {
                if song.artist.to_lowercase().contains(&i.to_lowercase()) || song.title.to_lowercase().contains(&i.to_lowercase()) {
                    can_set = true;
                }
            }
        },
        RichPresenceType::Disabled => {
            can_set = false;
        },
    }

    if can_set {
        client.borrow_mut().set_activity(Activity::new()
            .activity_type(ActivityType::Listening)
            .name("Backstop")
            .details(song.title.to_string())
            .state(song.artist.to_string())
            .status_display_type(StatusDisplayType::Details)
            .timestamps(Timestamps::new()
                .start(Utc::now().timestamp()))
            .buttons(vec![
                Button::new("Get Backstop", "https://github.com/iwantpizza10/backstop")
            ])
            .assets(Assets::new().large_image("https://github.com/iwantpizza10/backstop/blob/main/ui/res/backstopshort_square.png?raw=true"))
        )?;
    } else {
        client.borrow_mut().clear_activity()?;
    }

    Ok(())
}

// fn library_normal_convert(song: LibrarySong) -> SongFileInfo {
//     SongFileInfo {
//         filepath: PathBuf::from(song.path.to_string()),
//         title: song.title.to_string(),
//         length: Duration::from_secs(song.length as u64),
//         artist: song.artist.to_string(),
//         album_artist: if song.album_artist == "?" { None } else { Some(song.album_artist.to_string()) },
//         album: if song.album == "?" { None } else { Some(song.album.to_string()) },
//         track_number: if song.track_number == -1 { None } else { Some(song.track_number) },
//         year: if song.year == -1 { None } else { Some(song.year) },
//         cover: if song.cover_path == "?" { None } else { Some(song.cover_path.to_string()) }
//     }
// }

fn library_paranormal_convert(song: &SongFileInfo) -> LibrarySong {
    let mut path = constants::conf_dir();
    path.push("covers");
    let cover_image;

    if let Some(cover_path) = song.cover.clone() {
        path.push(cover_path);

        if let Ok(image) = image::open(path) {
            cover_image = image.into_rgba8();
        } else {
            cover_image = image::load_from_memory(PLACEHOLDER_COVER)
                .expect("placeholder coverart should process correctly")
                .into_rgba8();
        }
    } else {
        cover_image = image::load_from_memory(PLACEHOLDER_COVER)
            .expect("placeholder coverart should process correctly")
            .into_rgba8();
    }

    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(cover_image.as_raw(), cover_image.width(), cover_image.height());
    let cover = Image::from_rgba8(buffer);

    LibrarySong {
        album: song.album.clone().unwrap_or("".to_string()).into(),
        album_artist: song.album_artist.clone().unwrap_or("".to_string()).into(),
        artist: song.artist.clone().into(),
        length: song.length.as_secs() as i32,
        path: song.filepath.to_string_lossy().to_string().into(),
        title: song.title.clone().into(),
        track_number: song.track_number.unwrap_or(-1),
        year: song.year.unwrap_or(-1),
        cover_path: song.cover.clone().unwrap_or("?".to_string()).into(),
        cover
    }
}
