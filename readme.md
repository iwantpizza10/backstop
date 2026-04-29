<img src="assets/backstopfull.png" width=512>

> A desktop music player built with Rust that does what I want from one. Open source!

## Features

Backstop is rich with features. Here are some notable ones:
- Customized media directories
- Toggleable Discord RPC w/ an exclusion list
- Realtime playback speed controls 
- Artist/album filters
- Alphabetical sorting

## Getting Started
The app is intended to be *mostly* self-documenting, with a few exceptions,
but here's a quick guide for anyone trying to just gloss over.

You must first have a directory (or multiple!) that contains supported audio files with metadata (title, artist, cover art, etc). If metadata is not present, artist & title will be guessed from the filename. If you want to add metadata, I recommend [Mp3tag](https://www.mp3tag.de/).

Second, open Backstop and add the media directory. If this is your first time opening the app, you will be greeted with a welcome screen featuring an "Add a Media Directory" button. Otherwise, it can be found in the Media Directories section of the Settings menu, located at the bottom of the navbar on the left.

Third, click the Scan Library button. This can be found in the same locations as the media directory button from step 2.

Finally, after indexing (which can take a while depending on hardware and the number of files being processed) is done, enter the main songs section of the app. This can be done with the "Browse Library" button on the welcome screen or by clicking one of the top three buttons on the navbar.

## Controls
Upon entering the main songs view, clicking a song will play it and queue up all songs around it. `CTRL`+clicking (or `CMD`+click) a song will append it to the end of the queue. `CTRL`+`SHIFT`+clicking one will insert it into the queue as the next song.

On the left, there is a navbar with buttons that should all have tooltips explaining what they are/do.

On the bottom of the app, there is a footer with several interactive features. Starting from the left, there will be a song info section that only shows up when playing a song. Clicking the cover art will move you to a cover art view. In the center, there are several buttons under a position indicator. The buttons are as follows, from left to right:

- Stop playing, clear queue, clear rich presence, etc
- Toggle shuffle
- Skip to previous song
- Toggle play/pause
- Skip to next song
- Toggle repeat
- Peek queue (shows next few songs in the queue)
  - To close it, simply click off the menu

On the right, there are two sliders, labelled "Gain" and "Speed." These change the *gain*, or volume, and playback speed of the player respectively.

## Formats
Although not all of these have been fully tested, Backstop currently supports the following formats:
- mp3
- ogg
- flac
- m4a
- aac
- wav
- opus

## Contributions
Any contributions are welcome! If you're willing to do some programming, feel free to submit a PR. Otherwise, if you've found a bug, have a feature request, or just need help using Backstop, go ahead and submit an issue if you'd like.
