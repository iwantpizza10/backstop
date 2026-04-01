use rand::{rng, seq::SliceRandom};
use crate::cache::{MediaCache, SongFileInfo};

pub struct SongsQueue {
    position: usize,
    songs: Vec<SongFileInfo>,
    songs_unshuffled: Vec<SongFileInfo>
}

impl SongsQueue {
    pub fn new() -> Self {
        Self {
            position: 0,
            songs: vec![],
            songs_unshuffled: vec![]
        }
    }

    pub fn append_song(&mut self, song: SongFileInfo) {
        self.songs.push(song.clone());
        self.songs_unshuffled.push(song);
    }

    pub fn insert_next(&mut self, song: SongFileInfo) {
        if self.songs.len() == 0 {
            self.append_song(song);
        } else {
            self.songs.insert(self.position as usize, song.clone());
            self.songs_unshuffled.push(song); // we can make do w/ this
        }
    }

    pub fn songs(&self) -> &[SongFileInfo] {
        &self.songs
    }

    pub fn current_song(&self) -> Option<&SongFileInfo> {
        if self.position >= self.songs.len() {
            return None
        }

        Some(&self.songs[self.position])
    }

    pub fn next_song(&mut self) -> Option<&SongFileInfo> {
        if self.position + 1 >= self.songs.len() {
            return None;
        }
        
        self.position += 1;

        Some(&self.songs[self.position])
    }

    pub fn prev_song(&mut self) -> Option<&SongFileInfo> {
        if self.position == 0 {
            return None;
        }

        self.position -= 1;

        Some(&self.songs[self.position])
    }

    pub fn clear(&mut self) {
        self.songs.clear();
        self.songs_unshuffled.clear();
        self.position = 0;
    }

    pub fn shuffle(&mut self) {
        if self.songs.len() == 0 {
            return;
        }

        let first = self.songs[self.position].clone();
        let mut songs_vec: Vec<SongFileInfo> = self.songs.iter().filter(|x| **x != first).map(|x| x.clone()).collect();

        songs_vec.shuffle(&mut rng());
        songs_vec.insert(0, first);

        self.songs = songs_vec;
        self.position = 0;
    }

    pub fn unshuffle(&mut self) {
        if self.position >= self.songs.len() {
            return
        }

        let cur_song = self.songs[self.position].title.clone();
        self.songs = self.songs_unshuffled.clone();
        self.position = self.songs.iter()
            .position(|x| x.title == cur_song)
            .expect("schrodingers song has been located");
    }

    pub fn create_from_cache(cache: &MediaCache, current_song: i32) -> Self {
        Self {
            position: current_song as usize,
            songs: cache.songs().clone(),
            songs_unshuffled: cache.songs().clone()
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn next_3(&self) -> Vec<String> {
        let gen_str = |sog: Option<&SongFileInfo>| if let Some(sog) = sog {
            format!("{} - {}", sog.artist.clone(), sog.title.clone())
        } else {
            String::new()
        };

        vec![
            gen_str(self.songs.get(self.position + 1)),
            gen_str(self.songs.get(self.position + 2)),
            gen_str(self.songs.get(self.position + 3))
        ]
    }
}
