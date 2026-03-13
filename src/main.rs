#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use backstop::{cache::{self, MediaCache}, settings::BackstopSettings};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let _ = backstop::init_config_dirs(); // not gonna ? this cause its not technically needed to run

    let ui = BackstopWindow::new()?;
    let mut media_cache = cache::load_else_dead().unwrap_or_else(|_| MediaCache::dead());
    let settings = BackstopSettings::load_else_new();

    if media_cache.state() == &cache::CacheState::Dead {
        media_cache.rescan_library(settings.media_directories()).unwrap();
        media_cache.save_to_disk().unwrap();
    }

    if settings.is_first_launch() {
        ui.set_menustate(MenuState::Onboarding);
    } else {
        ui.set_menustate(MenuState::Welcome);
    }

    // // from here down is placeholder i suppose

    // let device_handle = DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    // let audio_player = Player::connect_new(&device_handle.mixer());
    // let index = Rc::new(RefCell::new(0));

    // ui.on_test({
    //     let idx2 = Rc::clone(&index);

    //     move || {
    //         let file = File::open(media_cache.songs()[*idx2.borrow()].filepath.clone()).unwrap();
    //         let source = Decoder::try_from(file).unwrap();

    //         audio_player.clear();
    //         audio_player.append(source);
    //         audio_player.play();
    //         *idx2.borrow_mut() += 1;
    //     }
    // });

    ui.run()?;

    Ok(())
}
