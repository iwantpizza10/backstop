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
    pub fn from_vec(vec: &Vec<Arc<SongFileInfo>>, current: Arc<SongFileInfo>) -> Option<Self> {
        if vec.len() == 0 {
            return None;
        }

        let pos = vec.iter().position(|x| *x == current)?;

        Some(Self {
            position: pos,
            rng: rand::rng(),
            songs: vec.clone(),
            unshuffled_songs: vec.clone(),
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

    /// returns an ordered list of songs in the queue
    pub fn get_songs(&self) -> &Vec<Arc<SongFileInfo>> {
        &self.songs
    }

    /// returns the previous song in the queue
    pub fn get_previous_song(&self) -> Option<Arc<SongFileInfo>> {
        self.songs.get(self.position - 1).map_or(None, |x| Some(Arc::clone(x)))
    }

    /// returns the current song in the queue
    pub fn get_current_song(&self) -> Arc<SongFileInfo> {
        Arc::clone(&self.songs[self.position])
    }

    /// returns the next song in the queue
    pub fn get_next_song(&self) -> Option<Arc<SongFileInfo>> {
        self.songs.get(self.position + 1).map_or(None, |x| Some(Arc::clone(x)))
    }

    /// de-advances the queue position & returns the new current song. returns None if de-advancing is not possible
    pub fn previous_song(&mut self) -> Option<Arc<SongFileInfo>> {
        if self.position >= self.songs.len() {
            return None;
        }

        self.position -= 1;
        Some(Arc::clone(&self.songs[self.position]))
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
            self.songs.get(self.position + 1).map_or(None, |x| Some(Arc::clone(x))),
            self.songs.get(self.position + 2).map_or(None, |x| Some(Arc::clone(x))),
            self.songs.get(self.position + 3).map_or(None, |x| Some(Arc::clone(x))),
        ]
    }

    /// shuffles the queue
    pub fn shuffle(&mut self) {
        let current_song = Arc::clone(&self.songs[self.position]);

        self.songs.remove(0);
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
