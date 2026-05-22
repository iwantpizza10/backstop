use async_fs::create_dir_all;

pub mod media_cache;
pub mod settings;
pub mod song_file_info;

use crate::BackstopError;
use crate::constants::conf_dir;
use crate::saved_state::media_cache::MediaCache;
use crate::saved_state::settings::BackstopSettings;

#[derive(Clone, Default, Debug)]
pub struct SavedState {
    pub settings: BackstopSettings,
    pub media_cache: MediaCache,
}

impl SavedState {
    pub async fn load() -> Result<Self, BackstopError> {
        let mut path = conf_dir();
        path.push("covers");
        
        if let Err(_) = create_dir_all(path).await {
            return Err(BackstopError::LoadingError);
        }

        let settings = BackstopSettings::load();
        let media_cache = MediaCache::load();

        if let Ok(settings) = settings && let Ok(media_cache) = media_cache {
            Ok(Self {
                settings,
                media_cache,
            })
        } else {
            Err(BackstopError::LoadingError)
        }
    }
}
