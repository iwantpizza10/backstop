use std::{fs::File, path::PathBuf};
use symphonia::core::{audio::{Audio, AudioBuffer, GenericAudioBuffer, GenericAudioBufferRef}, codecs::audio::AudioDecoderOptions, errors::Error, formats::{FormatOptions, TrackType, probe::Hint}, io::MediaSourceStream, meta::MetadataOptions};

pub struct Player {
    cur_song: GenericAudioBuffer,
    next_song: GenericAudioBuffer,
}

impl Player {
    fn decode_file(file: PathBuf) -> GenericAudioBuffer {
        let src = File::open(&file).unwrap();
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        let mut hint = Hint::new();
        hint.with_extension(file.extension().unwrap().to_str().unwrap());

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let mut format = symphonia::default::get_probe()
            .probe(&hint, mss, fmt_opts, meta_opts)
            .unwrap();

        let track = format.default_track(TrackType::Audio).unwrap();
        let decode_opts: AudioDecoderOptions = Default::default();

        let mut decoder = symphonia::default::get_codecs()
            .make_audio_decoder(
                track.codec_params.as_ref().unwrap().audio().unwrap(),
                &decode_opts
            ).unwrap();

        let track_id = track.id;

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
                    match decoded {
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
                    };
                },
                Err(Error::IoError(_)) | Err(Error::DecodeError(_)) => continue,
                Err(err) => panic!("{err}"),
            }
        }

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a() {
        Player::decode_file(PathBuf::from("/home/iwantpizza/Desktop/backstop/tests_res/file_scan_test/subfolder/beep_2.flac"));
        // Player::decode_file(PathBuf::from("/home/iwantpizza/Music/glorious flac collection/Eliminate - Get Off The Internet (2024)/03. Eliminate - EGOTRIP.flac"));

        panic!();
    }
}
