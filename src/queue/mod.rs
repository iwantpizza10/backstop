use rand::{rngs::ThreadRng, seq::SliceRandom};
use std::rc::Rc;

use crate::saved_state::song_file_info::SongFileInfo;

#[derive(Debug)]
pub struct Queue {
    position: usize,
    rng: ThreadRng,
    songs: Vec<Rc<SongFileInfo>>,
    unshuffled_songs: Vec<Rc<SongFileInfo>>,
}

impl Queue {
    /// returns a Queue or None if the Vec<_> is empty
    pub fn from_vec(vec: Vec<Rc<SongFileInfo>>) -> Option<Self> {
        if vec.len() == 0 {
            return None;
        }

        Some(Self {
            position: 0,
            rng: rand::rng(),
            songs: vec.clone(),
            unshuffled_songs: vec,
        })
    }

    /// appends song to the end of the queue
    pub fn append_song(&mut self, song: Rc<SongFileInfo>) {
        self.unshuffled_songs.push(Rc::clone(&song));
        self.songs.push(song);
    }

    /// inserts song as the next one in the queue
    pub fn insert_next(&mut self, song: Rc<SongFileInfo>) {
        self.unshuffled_songs.push(Rc::clone(&song));
        self.songs.insert(self.position + 1, song);
    }

    /// returns an ordered list of songs in the queue
    pub fn get_songs(&self) -> &Vec<Rc<SongFileInfo>> {
        &self.songs
    }

    /// returns the previous song in the queue
    pub fn get_previous_song(&self) -> Option<Rc<SongFileInfo>> {
        self.songs.get(self.position - 1).map_or(None, |x| Some(Rc::clone(x)))
    }

    /// returns the current song in the queue
    pub fn get_current_song(&self) -> Rc<SongFileInfo> {
        Rc::clone(&self.songs[self.position])
    }

    /// returns the next song in the queue
    pub fn get_next_song(&self) -> Option<Rc<SongFileInfo>> {
        self.songs.get(self.position + 1).map_or(None, |x| Some(Rc::clone(x)))
    }

    /// de-advances the queue position & returns the new current song. returns None if de-advancing is not possible
    pub fn previous_song(&mut self) -> Option<Rc<SongFileInfo>> {
        if self.position >= self.songs.len() {
            return None;
        }

        self.position -= 1;
        Some(Rc::clone(&self.songs[self.position]))
    }

    /// advances the queue position & returns the new current song. returns None if advancing is not possible
    pub fn next_song(&mut self) -> Option<Rc<SongFileInfo>> {
        if self.position >= self.songs.len() {
            return None;
        }

        self.position += 1;
        Some(Rc::clone(&self.songs[self.position]))
    }

    /// returns the next 3 songs in the queue
    pub fn peek(&self) -> Vec<Option<Rc<SongFileInfo>>> {
        vec![
            self.songs.get(self.position + 1).map_or(None, |x| Some(Rc::clone(x))),
            self.songs.get(self.position + 2).map_or(None, |x| Some(Rc::clone(x))),
            self.songs.get(self.position + 3).map_or(None, |x| Some(Rc::clone(x))),
        ]
    }

    /// shuffles the queue
    pub fn shuffle(&mut self) {
        let current_song = Rc::clone(&self.songs[self.position]);
        
        self.songs.remove(0);
        self.songs.shuffle(&mut self.rng);
        self.songs.insert(0, current_song);
        self.position = 0;
    }

    /// unshuffles the queue
    pub fn unshuffle(&mut self) {
        let current_song = Rc::clone(&self.songs[self.position]);
        
        self.songs = self.unshuffled_songs.clone();
        self.position = self.songs.iter()
            .position(|x| **x == *current_song)
            .expect("schrodinger strikes again");
    }
}
