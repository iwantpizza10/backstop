mod types;

use rodio::cpal::traits::HostTrait;
use rodio::{Device, DeviceSinkBuilder, DeviceTrait, MixerDeviceSink, Player as RodioPlayer};
use rodio::cpal;
pub use types::*;

pub struct Player {
    audio_player: rodio::Player,
    device: MixerDeviceSink,
}

impl Player {
    fn get_device_list() -> Vec<Device> {
        let host = cpal::default_host();
        match host.devices() {
            Ok(devices) => {
                devices
                    .filter(|x| x.supports_output())
                    .collect()
            },
            Err(err) => {
                eprintln!("error getting device list: {err}");

                vec![]
            }
        }
    }

    fn switch_device(&mut self, device: Device) {
        self.device = DeviceSinkBuilder::from_device(device)
            .expect("error opening device")
            .open_sink_or_fallback()
            .expect("error opening device sink");
    }

    fn new() -> Self {
        let device = DeviceSinkBuilder::open_default_sink().expect("error opening default device sink");

        let instance = Self {
            audio_player: RodioPlayer::connect_new(&device.mixer()),
            device,
        };

        instance
    }

    fn with_device(device: Device) -> Self {
        let device = DeviceSinkBuilder::from_device(device)
            .expect("error opening device")
            .open_sink_or_fallback()
            .expect("error opening device sink");

        let instance = Self {
            audio_player: RodioPlayer::connect_new(&device.mixer()),
            device,
        };

        instance
    }
}

#[cfg(test)]
mod tests {
    // ...

    use crate::player::Player;
    use rodio::DeviceTrait;

    #[test]
    fn main() {
        panic!();
    }

}
