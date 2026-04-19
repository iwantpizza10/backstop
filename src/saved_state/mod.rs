pub mod media_cache;
pub mod settings;
pub mod song_file_info;

use crate::BackstopError;
use crate::saved_state::media_cache::MediaCache;
use crate::saved_state::settings::BackstopSettings;

#[derive(Clone)]
pub struct SavedState {
    pub settings: BackstopSettings,
    pub media_cache: MediaCache,
}

impl SavedState {
    pub async fn load() -> Result<Self, BackstopError> {
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
