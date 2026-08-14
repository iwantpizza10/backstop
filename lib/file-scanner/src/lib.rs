use image::ImageFormat;
use lofty::{
    file::TaggedFileExt,
    picture::PictureType,
    probe::Probe,
    tag::{Accessor, ItemKey},
};
use std::{
    collections::{HashMap, HashSet},
    io,
    path::PathBuf,
};
use types::{Album, Artist, SongItem};
use uuid::Uuid;

fn discover_files(dir: &PathBuf) -> Result<Vec<PathBuf>, io::Error> {
    let scan = std::fs::read_dir(dir)?;
    let mut results: Vec<PathBuf> = vec![];

    for item in scan {
        let item = item?;

        if let Ok(file_type) = item.file_type()
            && file_type.is_dir()
        {
            let mut recursive_scan = discover_files(&item.path())?;

            results.append(&mut recursive_scan);
        } else {
            results.push(item.path());
        }
    }

    Ok(results)
}

fn save_image_to_file(data: &[u8], id: Uuid) -> Result<(), image::ImageError> {
    let mut image = image::load_from_memory(data)?;

    let mut location = constants::conf_dir();
    location.push("covers");
    location.push(id.to_string());
    location.set_extension("png");

    let smaller_size = if image.width() > image.height() {
        image.width()
    } else {
        image.height()
    };

    let x_offset = (image.width() - smaller_size) / 2;

    image = image.crop(x_offset, 0, smaller_size, smaller_size);
    image.save_with_format(location, ImageFormat::Png)?;

    Ok(())
}

fn scan_file_metadata(file: &PathBuf, create_cover: bool) -> Option<SongItem> {
    let file_probe = match Probe::open(file) {
        Ok(probe) => probe,
        Err(x) => {
            eprintln!("lofty error: {x} (during read of {:?})", file);
            return None;
        }
    };

    let tagged_file = match file_probe.read() {
        Ok(file) => file,
        Err(x) => {
            eprintln!("lofty error: {x} (during read of {:?})", file);
            return None;
        }
    };

    let tags = match tagged_file.first_tag() {
        Some(tag) => tag,
        None => {
            eprintln!(
                "lofty error: first_tag not found (during read of {:?})",
                file
            );
            return None;
        }
    };

    let title = tags.title().map(|x| x.to_string());
    let album_name = tags.album().map(|x| x.to_string());
    let track = tags.track();
    let total_tracks = tags.track_total();
    let genre = tags.genre().map(|x| x.to_string());
    let date = tags.date();
    let id = Uuid::new_v4();

    macro_rules! process_items {
        ($type_singular:expr, $type_plural:expr, $split_seq:expr) => {{
            let items = tags.get_strings($type_singular).collect::<Vec<_>>();
            let items_more = tags.get_strings($type_plural).collect::<Vec<_>>();
            let mut actual_items: Vec<String> = vec![];
            let ignore_results: bool;

            if items.len() == 0 && items_more.len() == 0 {
                ignore_results = true;
            } else {
                ignore_results = false;
            }

            for item in items {
                let items_split = item.split($split_seq);

                for split_item in items_split {
                    actual_items.push(split_item.to_string());
                }
            }

            for item in items_more {
                let items_split = item.split($split_seq);

                for split_item in items_split {
                    actual_items.push(split_item.to_string());
                }
            }

            if ignore_results {
                None
            } else {
                Some(actual_items)
            }
        }};
    }

    let track_artists = process_items!(ItemKey::TrackArtist, ItemKey::TrackArtists, ", ");
    let album_artists = process_items!(ItemKey::AlbumArtist, ItemKey::AlbumArtists, ", ");

    if create_cover {
        for i in tags.pictures() {
            if i.pic_type() == PictureType::CoverFront {
                if let Err(x) = save_image_to_file(i.data(), id) {
                    eprintln!(
                        "cover error: {x:?} (while saving \"{} - {}\", id {})",
                        track_artists
                            .as_ref()
                            .map(|x| x.join(", "))
                            .unwrap_or("?".to_string()),
                        title.as_ref().unwrap_or(&"?".to_string()),
                        id
                    );
                }

                // note how we do NOT return `None` here cause not having a cover art still
                // results in 100% listenable audio i hope for most people lmao

                break;
            }
        }
    }

    Some(SongItem {
        title,
        track_artists,
        date,
        track,
        genre,
        total_tracks,
        album_name,
        album_artists,
        id,
    })
}

fn scan_dir_metadata(dir: &PathBuf, create_covers: bool) -> Vec<SongItem> {
    let files = match discover_files(dir) {
        Ok(x) => x,
        Err(err) => {
            eprintln!("io error: {err:?} (during read of {:?})", dir);
            return vec![];
        }
    };

    let mut songitems = vec![];

    for file in files {
        let metadata = scan_file_metadata(&file, create_covers);

        match metadata {
            Some(metadata) => songitems.push(metadata),
            None => continue,
        }
    }

    songitems
}

fn scan_artists(items: &Vec<SongItem>) -> Vec<Artist> {
    let mut artists: HashMap<Option<String>, Artist> = HashMap::new();

    for item in items {
        if let Some(track_artists) = &item.track_artists {
            for track_artist in track_artists {
                if let Some(artist) = artists.get_mut(&Some(track_artist.clone())) {
                    artist.tracks.push(item.id);

                    if let Some(album_artists) = item.album_artists.clone() {
                        if album_artists.contains(&track_artist.clone())
                            && !artist.album_names.contains(&item.album_name)
                        {
                            artist.album_names.push(item.album_name.clone());
                        }
                    }
                } else {
                    let mut artist = Artist {
                        name: Some(track_artist.clone()),
                        album_names: vec![],
                        tracks: vec![item.id],
                    };

                    if let Some(album_artists) = item.album_artists.clone() {
                        if album_artists.contains(&track_artist.clone())
                            && !artist.album_names.contains(&item.album_name)
                        {
                            artist.album_names.push(item.album_name.clone());
                        }
                    }

                    artists.insert(Some(track_artist.clone()), artist);
                }
            }
        } else {
            if let Some(artist) = artists.get_mut(&None) {
                artist.tracks.push(item.id);

                if let None = item.album_artists.clone() {
                    if !artist.album_names.contains(&item.album_name) {
                        artist.album_names.push(item.album_name.clone());
                    }
                }
            } else {
                let mut artist = Artist {
                    name: None,
                    album_names: vec![],
                    tracks: vec![item.id],
                };

                if let None = item.album_artists.clone() {
                    if !artist.album_names.contains(&item.album_name) {
                        artist.album_names.push(item.album_name.clone());
                    }
                }

                artists.insert(None, artist);
            }
        }
    }

    artists.into_values().collect()
}

fn scan_albums(items: &Vec<SongItem>) -> Vec<Album> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lofty::tag::items::Timestamp;
    use std::{ffi::OsStr, str::FromStr};

    #[test]
    fn files_discover() {
        let path = PathBuf::from_str("../../tests_res/file_scan_test").expect("path should parse");
        let files = discover_files(&path).expect("should not error while discovering");

        assert_eq!(files.len(), 3);
        assert_eq!(files[0].file_name(), Some(OsStr::new("beep_1.mp3")));
        assert_eq!(files[1].file_name(), Some(OsStr::new("hello.txt")));
        assert_eq!(files[2].file_name(), Some(OsStr::new("beep_2.flac")));
    }

    #[test]
    fn metadata_scans_mp3() {
        let path = PathBuf::from_str("../../tests_res/file_scan_test").expect("path should parse");
        let files = discover_files(&path).expect("should not error while discovering");
        let beep_1 = scan_file_metadata(&files[0], false).unwrap();

        assert_eq!(beep_1.title, Some("beep_1".to_string()));
        assert_eq!(
            beep_1.track_artists,
            Some(vec!["me".to_string(), "my friend".to_string()])
        );
        assert_eq!(beep_1.date, Some(Timestamp::from_str("2026").unwrap()));
        assert_eq!(beep_1.track, Some(1));
        assert_eq!(beep_1.total_tracks, Some(2));
        assert_eq!(beep_1.album_name, Some("the beeps".to_string()));
        assert_eq!(beep_1.album_artists, Some(vec!["me".to_string()]));
    }

    #[test]
    #[should_panic]
    fn metadata_does_not_scan_txt() {
        let path = PathBuf::from_str("../../tests_res/file_scan_test").expect("path should parse");
        let files = discover_files(&path).expect("should not error while discovering");
        let beep_2 = scan_file_metadata(&files[1], false).unwrap();
    }

    #[test]
    fn metadata_scans_flac() {
        let path = PathBuf::from_str("../../tests_res/file_scan_test").expect("path should parse");
        let files = discover_files(&path).expect("should not error while discovering");
        let beep_2 = scan_file_metadata(&files[2], false).unwrap();

        assert_eq!(beep_2.title, Some("beep_2".to_string()));
        assert_eq!(beep_2.track_artists, Some(vec!["me".to_string()]));
        assert_eq!(
            beep_2.date,
            Some(Timestamp::from_str("2026-08-07").unwrap())
        );
        assert_eq!(beep_2.track, Some(2));
        assert_eq!(beep_2.total_tracks, Some(2));
        assert_eq!(beep_2.album_name, Some("the beeps".to_string()));
        assert_eq!(beep_2.album_artists, Some(vec!["me".to_string()]));
    }

    #[test]
    fn metadata_scans_dir() {
        let path = PathBuf::from_str("../../tests_res/file_scan_test").expect("path should parse");
        let scan = scan_dir_metadata(&path, false);

        assert_eq!(scan.len(), 2);
        // no more checks are needed because metadata_scans_(mp3|flac) covers it all
    }

    #[test]
    fn scan_artists_from_dir_metadata() {
        let path = PathBuf::from_str("../../tests_res/file_scan_test").expect("path should parse");
        let scan = scan_dir_metadata(&path, false);
        let artists = scan_artists(&scan);
        let the_one_track_uuid = artists[0].tracks[0];

        assert_eq!(artists.len(), 2);
        // not going to test more because i frankly don't feel like writing
        // code to handle when the artists flip spots randomly due to hashsets
        // being arbitrarily ordered when collected into Vec<_>s or whatever
    }
}
