use std::{error, fmt::Display, fs::File, io, path::PathBuf};
use symphonia::core::{audio::{GenericAudioBuffer, GenericAudioBufferRef}, codecs::audio::AudioDecoderOptions, errors::Error, formats::{FormatOptions, TrackType, probe::Hint}, io::MediaSourceStream, meta::MetadataOptions};

#[derive(Debug)]
pub enum DecodeError {
    NoTrackFound,
    ParamsDontExist,
    SymphoniaError(Error),
    IoError(io::Error),
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for DecodeError {}

pub struct Player {
    cur_song: GenericAudioBuffer,
    next_song: GenericAudioBuffer,
}

impl Player {
    fn decode_file(file: PathBuf) -> Result<Vec<GenericAudioBuffer>, DecodeError> {
        let src = match File::open(&file) {
            Ok(s) => s,
            Err(err) => {
                println!("decode error: {err:?}");
                return Err(DecodeError::IoError(err));
            },
        };

        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = file.extension() && let Some(ext) = ext.to_str() {
            hint.with_extension(ext);
        } else {
            println!("decode could not find file extension");
        }

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let mut format = match symphonia::default::get_probe()
            .probe(&hint, mss, fmt_opts, meta_opts)
        {
            Ok(reader) => reader,
            Err(err) => {
                println!("decode error: {err}");
                return Err(DecodeError::SymphoniaError(err));
            },
        };
        
        let decode_opts: AudioDecoderOptions = Default::default();
        let track = if let Some(tk) = format.default_track(TrackType::Audio) {
            tk
        } else if let Some(tk) = format.first_track(TrackType::Audio) {
            tk
        } else {
            return Err(DecodeError::NoTrackFound);
        };

        let params = if let Some(p) = track.codec_params.as_ref() && let Some(p) = p.audio() {
            p
        } else {
            println!("decode error: {}", DecodeError::ParamsDontExist); // lmao
            return Err(DecodeError::ParamsDontExist);
        };

        let mut decoder = match symphonia::default::get_codecs()
            .make_audio_decoder(params, &decode_opts)
        {
            Ok(d) => d,
            Err(err) => {
                println!("decode error: {err}");
                return Err(DecodeError::SymphoniaError(err));
            },
        };

        let track_id = track.id;
        let mut audio_buffers = vec![];

        loop {
            let packet = match format.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => break,
                Err(Error::ResetRequired) => unimplemented!(),
                Err(err) => panic!("{err}"),
            };

            while !format.metadata().is_latest() {
                format.metadata().pop();
            }

            if packet.track_id != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    audio_buffers.push(match decoded {
                        GenericAudioBufferRef::S8 (buf) => GenericAudioBuffer::S8(buf.clone()),
                        GenericAudioBufferRef::S16(buf) => GenericAudioBuffer::S16(buf.clone()),
                        GenericAudioBufferRef::S24(buf) => GenericAudioBuffer::S24(buf.clone()),
                        GenericAudioBufferRef::S32(buf) => GenericAudioBuffer::S32(buf.clone()),
                        GenericAudioBufferRef::U8 (buf) => GenericAudioBuffer::U8(buf.clone()),
                        GenericAudioBufferRef::U16(buf) => GenericAudioBuffer::U16(buf.clone()),
                        GenericAudioBufferRef::U24(buf) => GenericAudioBuffer::U24(buf.clone()),
                        GenericAudioBufferRef::U32(buf) => GenericAudioBuffer::U32(buf.clone()),
                        GenericAudioBufferRef::F32(buf) => GenericAudioBuffer::F32(buf.clone()),
                        GenericAudioBufferRef::F64(buf) => GenericAudioBuffer::F64(buf.clone()),
                    });
                },

                Err(Error::IoError(_)) | Err(Error::DecodeError(_)) => continue,
                Err(err) => panic!("{err}"),
            }
        }

        Ok(audio_buffers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_beep_flac() {
        let bufs = Player::decode_file(PathBuf::from("../tests_res/file_scan_test/subfolder/beep_2.flac"));
        let sample_counts = bufs.unwrap().iter().map(|x| x.samples_planar()).collect::<Vec<_>>();

        assert_eq!(sample_counts, [4096usize, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 2069]);
    }

    #[test]
    fn decodes_beep_mp3() {
        let bufs = Player::decode_file(PathBuf::from("../tests_res/file_scan_test/beep_1.mp3"));
        let sample_counts = bufs.unwrap().iter().map(|x| x.samples_planar()).collect::<Vec<_>>();

        assert_eq!(sample_counts, [1152usize; 67]);
    }
}
