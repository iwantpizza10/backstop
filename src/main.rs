#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, error::Error, fs::File, rc::Rc};
use backstop::{cache::{self, CacheState, MediaCache}, settings::BackstopSettings};
use rodio::{Decoder, DeviceSinkBuilder, Player};
use slint::{Model, ModelRc, VecModel};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let _ = backstop::init_config_dirs(); // not gonna ? this cause its not technically needed to run

    let ui = BackstopWindow::new()?;

    let media_cache = Rc::new(RefCell::new(cache::load_else_dead().unwrap_or_else(|_| MediaCache::dead())));
    let media_cache_model: Rc<VecModel<LibrarySong>> = Rc::new(VecModel::from(vec![]));
    let media_cache_rc = ModelRc::from(media_cache_model);

    let settings = BackstopSettings::load_else_new();
    let mut audio_device_handle = DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    audio_device_handle.log_on_drop(false);
    let audio_player = Player::connect_new(&audio_device_handle.mixer());

    if media_cache.borrow().state() == &CacheState::Dead {
        media_cache.borrow_mut().rescan_library(settings.media_directories()).unwrap();
        media_cache.borrow().save_to_disk().unwrap();
    }

    load_cache_to_model(&media_cache.borrow(), media_cache_rc.clone());

    if settings.is_first_launch() {
        ui.set_menustate(MenuState::Onboarding);
    } else {
        ui.set_menustate(MenuState::Welcome);
    }

    // theres gonna be a few more of these types of calls so ima not group em w/ something else yet
    ui.set_media_library(media_cache_rc.clone());

    ui.on_play_song({
        let ui_handle = ui.as_weak();

        move |song| {
            let ui = ui_handle.unwrap();
            let song_path = song.path.clone();

            ui.set_current_song(song);
            ui.set_playing(true);
            ui.set_song_position(0);

            let file = File::open(song_path).unwrap();
            let source = Decoder::try_from(file).unwrap();

            audio_player.clear();
            audio_player.append(source);
            audio_player.play();
        }
    });

    ui.on_rescan_library({
        let media_cache = Rc::clone(&media_cache);

        move || {
            media_cache.borrow_mut().rescan_library(settings.media_directories()).unwrap();
            media_cache.borrow().save_to_disk().unwrap();

            load_cache_to_model(&media_cache.borrow(), media_cache_rc.clone());
        }
    });

    ui.run()?;

    Ok(())
}

fn load_cache_to_model(media_cache: &MediaCache, media_cache_rc: ModelRc<LibrarySong>) {
    let test: &VecModel<LibrarySong> = media_cache_rc.as_any().downcast_ref().unwrap();
    test.clear();

    for i in media_cache.songs() {
        let lsong = LibrarySong {
            album: i.album.clone().unwrap_or("".to_string()).into(),
            album_artist: i.album_artist.clone().unwrap_or("".to_string()).into(),
            artist: i.artist.clone().into(),
            length: i.length.as_secs() as i32,
            path: i.filepath.to_string_lossy().to_string().into(),
            title: i.title.clone().into(),
            track_number: i.track_number.unwrap_or(-1),
            year: i.year.unwrap_or(-1)
        };

        test.push(lsong);
    }
}
