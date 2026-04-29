use rand::{rngs::ThreadRng, seq::SliceRandom};
use std::{sync::Arc};

use crate::saved_state::song_file_info::SongFileInfo;

#[derive(Debug)]
pub struct Queue {
    position: usize,
    rng: ThreadRng,
    songs: Vec<Arc<SongFileInfo>>,
    unshuffled_songs: Vec<Arc<SongFileInfo>>,
}

impl Queue {
    /// returns a Queue or None if the Vec<_> is empty
    pub fn from_vec(vec: &[Arc<SongFileInfo>], current: Arc<SongFileInfo>) -> Option<Self> {
        if vec.is_empty() {
            return None;
        }

        let pos = vec.iter().position(|x| *x == current)?;

        Some(Self {
            position: pos,
            rng: rand::rng(),
            songs: vec.to_owned(),
            unshuffled_songs: vec.to_owned(),
        })
    }

    /// appends song to the end of the queue
    pub fn append_song(&mut self, song: Arc<SongFileInfo>) {
        self.unshuffled_songs.push(Arc::clone(&song));
        self.songs.push(song);
    }

    /// inserts song as the next one in the queue
    pub fn insert_next(&mut self, song: Arc<SongFileInfo>) {
        self.unshuffled_songs.push(Arc::clone(&song));
        self.songs.insert(self.position + 1, song);
    }

    /// de-advances the queue position & returns the new current song. returns None if de-advancing is not possible
    pub fn previous_song(&mut self) -> Option<Arc<SongFileInfo>> {
        if self.position >= self.songs.len() {
            return None;
        }

        if self.position == 0 {
            None
        } else {
            self.position -= 1;
            Some(Arc::clone(&self.songs[self.position]))
        }
    }

    /// advances the queue position & returns the new current song. returns None if advancing is not possible
    pub fn next_song(&mut self) -> Option<Arc<SongFileInfo>> {
        if self.position >= self.songs.len() {
            return None;
        }

        if self.position + 1 >= self.songs.len() {
            None
        } else {
            self.position += 1;
            Some(Arc::clone(&self.songs[self.position]))
        }
    }

    /// returns the next 3 songs in the queue
    pub fn peek(&self) -> Vec<Option<Arc<SongFileInfo>>> {
        vec![
            self.songs.get(self.position + 1).map(Arc::clone),
            self.songs.get(self.position + 2).map(Arc::clone),
            self.songs.get(self.position + 3).map(Arc::clone),
        ]
    }

    /// shuffles the queue
    pub fn shuffle(&mut self) {
        let current_song = Arc::clone(&self.songs[self.position]);

        self.songs.remove(self.position);
        self.songs.shuffle(&mut self.rng);
        self.songs.insert(0, current_song);
        self.position = 0;
    }

    /// unshuffles the queue
    pub fn unshuffle(&mut self) {
        let current_song = Arc::clone(&self.songs[self.position]);
        
        self.songs = self.unshuffled_songs.clone();
        self.position = self.songs.iter()
            .position(|x| **x == *current_song)
            .expect("schrodinger strikes again");
    }
}
