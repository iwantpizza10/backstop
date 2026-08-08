use image::ImageFormat;
use lofty::{
    file::TaggedFileExt,
    picture::PictureType,
    probe::Probe,
    tag::{Accessor, ItemKey},
};
use std::{fs::DirEntry, io, path::PathBuf};
use types::SongItem;
use uuid::Uuid;

fn discover_files(dir: PathBuf) -> Result<Vec<DirEntry>, io::Error> {
    let scan = std::fs::read_dir(dir)?;
    let mut results: Vec<DirEntry> = vec![];

    for item in scan {
        let item = item?;

        if let Ok(file_type) = item.file_type()
            && file_type.is_dir()
        {
            let mut recursive_scan = discover_files(item.path())?;

            results.append(&mut recursive_scan);
        } else {
            results.push(item);
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

// "one metadata please 🤓👆"
fn scan_metadata_singular(file: &DirEntry, create_cover: bool) -> SongItem {
    let tagged_file = Probe::open(file.path()).unwrap().read().unwrap();
    let tagged_file = tagged_file.first_tag().unwrap();

    let title = tagged_file.title().map(|x| x.to_string());
    let album_name = tagged_file.album().map(|x| x.to_string());
    let track = tagged_file.track();
    let total_tracks = tagged_file.track_total();
    let genre = tagged_file.genre().map(|x| x.to_string());
    let date = tagged_file.date();
    let id = Uuid::new_v4();

    macro_rules! process_items {
        ($type_singular:expr, $type_plural:expr, $split_seq:expr) => {{
            let items = tagged_file.get_strings($type_singular).collect::<Vec<_>>();
            let items_more = tagged_file.get_strings($type_plural).collect::<Vec<_>>();
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

    let artist_names = process_items!(ItemKey::TrackArtist, ItemKey::TrackArtists, ", ");

    let album_artists = process_items!(ItemKey::AlbumArtist, ItemKey::AlbumArtists, ", ");

    if create_cover {
        for i in tagged_file.pictures() {
            if i.pic_type() == PictureType::CoverFront {
                if save_image_to_file(i.data(), id).is_err() {
                    eprintln!(
                        "cover art for \"{} - {}\" (id {}) did not save!",
                        artist_names
                            .as_ref()
                            .map(|x| x.join(", "))
                            .unwrap_or("?".to_string()),
                        title.as_ref().unwrap_or(&"?".to_string()),
                        id
                    );
                }

                break;
            }
        }
    }

    SongItem {
        title,
        artist_names,
        date,
        track,
        genre,
        total_tracks,
        album_name,
        album_artists,
        id,
    }
}

#[cfg(test)]
mod tests {
    use lofty::tag::items::Timestamp;

    use super::*;
    use std::str::FromStr;

    #[test]
    fn files_discover() {
        let path = PathBuf::from_str("../../tests_res/file_scan_test").expect("path should parse");
        let files = discover_files(path).expect("should not error while discovering");

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].file_name(), "beep_1.mp3");
        assert_eq!(files[1].file_name(), "beep_2.flac");
    }

    #[test]
    fn metadata_scans_mp3() {
        let path = PathBuf::from_str("../../tests_res/file_scan_test").expect("path should parse");
        let files = discover_files(path).expect("should not error while discovering");
        let beep_1 = scan_metadata_singular(&files[0], false);

        assert_eq!(beep_1.title, Some("beep_1".to_string()));
        assert_eq!(
            beep_1.artist_names,
            Some(vec!["me".to_string(), "my friend".to_string()])
        );
        assert_eq!(beep_1.date, Some(Timestamp::from_str("2026").unwrap()));
        assert_eq!(beep_1.track, Some(1));
        assert_eq!(beep_1.total_tracks, Some(2));
        assert_eq!(beep_1.album_name, Some("the beeps".to_string()));
        assert_eq!(beep_1.album_artists, Some(vec!["me".to_string()]));
    }

    #[test]
    fn metadata_scans_flac() {
        let path = PathBuf::from_str("../../tests_res/file_scan_test").expect("path should parse");
        let files = discover_files(path).expect("should not error while discovering");
        let beep_2 = scan_metadata_singular(&files[1], false);

        assert_eq!(beep_2.title, Some("beep_2".to_string()));
        assert_eq!(beep_2.artist_names, Some(vec!["me".to_string()]));
        assert_eq!(
            beep_2.date,
            Some(Timestamp::from_str("2026-08-07").unwrap())
        );
        assert_eq!(beep_2.track, Some(2));
        assert_eq!(beep_2.total_tracks, Some(2));
        assert_eq!(beep_2.album_name, Some("the beeps".to_string()));
        assert_eq!(beep_2.album_artists, Some(vec!["me".to_string()]));
    }
}
