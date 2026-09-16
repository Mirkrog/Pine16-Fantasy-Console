use tinyaudio::*;

#[derive(Default)]
struct Wave {
    pitch: u16,
    volume: u8,
    modifier: u8,
}

pub struct AudioModule {
    output_device: OutputDevice,
    sine: Wave,
    saw: Wave,
    noise: Wave,
    triangle: Wave,
    square: Wave,
}

impl AudioModule {
    pub fn new() -> Self {
        let params = OutputDeviceParameters {
            channels_count: 1,
            sample_rate: 44100,
            channel_sample_count: 1000,
        };

        let mut clock = 0f32;
        Self {
            output_device: run_output_device(params, move |data| {
                clock = (clock + 0.1) % params.sample_rate as f32;
                let value =
                    (clock * 440.0 * 2.0 * std::f32::consts::PI / params.sample_rate as f32).sin();
                for sample in data {
                    *sample = value;
                }
            })
            .unwrap(),
            sine: Default::default(),
            saw: Default::default(),
            noise: Default::default(),
            triangle: Default::default(),
            square: Default::default(),
        }
    }
}
