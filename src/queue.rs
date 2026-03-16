use rand::{rng, seq::SliceRandom};
use crate::cache::{MediaCache, SongFileInfo};

pub struct SongsQueue {
    position: usize,
    songs: Vec<SongFileInfo>
}

impl SongsQueue {
    pub fn new() -> Self {
        Self {
            position: 0,
            songs: vec![]
        }
    }

    pub fn append_song(&mut self, song: SongFileInfo) {
        self.songs.push(song);
    }

    pub fn insert_next(&mut self, song: SongFileInfo) {
        if self.songs.len() == 0 {
            self.append_song(song);
        } else {
            self.songs.insert(self.position as usize, song);
        }
    }

    pub fn songs(&self) -> &[SongFileInfo] {
        &self.songs
    }

    pub fn current_song(&self) -> &SongFileInfo {
        &self.songs[self.position]
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
        self.position = 0;
    }

    pub fn shuffle(&mut self) {
        let first = self.songs[0].clone();
        let mut songs_vec: Vec<SongFileInfo> = self.songs.iter().filter(|x| **x != first).map(|x| x.clone()).collect();

        songs_vec.shuffle(&mut rng());
        songs_vec.insert(0, first);

        self.songs = songs_vec;
    }

    pub fn create_from_cache(cache: &MediaCache, current_song: i32) -> Self {
        Self {
            position: current_song as usize,
            songs: cache.songs().clone()
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }
}
