use std::sync::Arc;

use ringbuf::{
    HeapRb, SharedRb,
    storage::Heap,
    traits::{Consumer, Observer, Producer, Split},
    wrap::caching::Caching,
};

use cpal::{
    Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

const SAMPLE_RATE: usize = 11000;
const BUFFER_SIZE: usize = SAMPLE_RATE / 30;

const CHANNEL_COUNT: usize = 5;

const CHANNEL_ADDRESS_OFFSET: usize = 6543;

pub struct AudioModule {
    _stream: Stream,
    channel_phases: [f32; CHANNEL_COUNT],
    producer: Caching<Arc<SharedRb<Heap<f32>>>, true, false>,
    lfsr_state: u16,
}

impl AudioModule {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No output device found");

        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: SAMPLE_RATE as u32,
            buffer_size: cpal::BufferSize::Default,
        };

        // The buffer is slightly bigger to allow the OS and the console to desync a little to prevent audio glitches
        let (mut producer, mut consumer) = HeapRb::new(BUFFER_SIZE * 4).split();

        for _ in 0..producer.capacity().into() {
            producer.try_push(0.0).unwrap();
        }

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    consumer.pop_slice(data);
                },
                |err| log::warn!("Audio stream error: {:?}", err),
                None,
            )
            .unwrap();

        stream.play().unwrap();

        Self {
            _stream: stream,
            producer,
            channel_phases: [0.0; CHANNEL_COUNT],
            lfsr_state: 0xFFFF,
        }
    }

    pub fn render(&mut self, ram_slice: &[u16]) {
        let overshoot: usize = if self.producer.occupied_len() > BUFFER_SIZE * 2 {
            0
        } else {
            30
        };

        for _ in 0..BUFFER_SIZE + overshoot {
            let sample = self.step_sample(ram_slice);
            self.producer.try_push(sample).unwrap();
        }
    }

    fn step_sample(&mut self, ram_slice: &[u16]) -> f32 {
        let mut mixed_sum: f32 = 0.0;

        for ch in 0..CHANNEL_COUNT {
            let freq = ram_slice[CHANNEL_ADDRESS_OFFSET + ch * 2];
            let cfg = ram_slice[CHANNEL_ADDRESS_OFFSET + ch * 2 + 1];

            let volume = (cfg & 0x00FF) as f32 / 255.0;
            let modifier = ((cfg >> 8) & 0x00FF) as u8;

            if freq == 0 || volume == 0.0 {
                continue;
            }

            let phase = self.channel_phases[ch];
            self.channel_phases[ch] = (phase + freq as f32 / SAMPLE_RATE as f32).fract();

            let raw_sample = match ch {
                0 => render_sine(phase, modifier),
                1 => render_triangle(phase, modifier),
                2 => render_saw(phase, modifier),
                3 => render_square(phase, modifier),
                4 => render_noise(&mut self.lfsr_state, modifier),
                _ => 0.0,
            };

            mixed_sum += raw_sample * volume;
        }

        (mixed_sum * 0.35).clamp(-1.0, 1.0)
    }
}

fn render_sine(phase: f32, modifier: u8) -> f32 {
    let raw = (phase * std::f32::consts::TAU).sin();
    if modifier == 0 {
        raw
    } else {
        let gain = 1.0 + (modifier as f32 / 64.0);
        (raw * gain).clamp(-1.0, 1.0)
    }
}

fn render_triangle(phase: f32, _modifier: u8) -> f32 {
    4.0 * (phase - 0.5).abs() - 1.0
}

fn render_saw(phase: f32, modifier: u8) -> f32 {
    let mut saw = 2.0 * phase - 1.0;
    if modifier != 0 {
        saw = -saw;
    }
    saw
}

fn render_square(phase: f32, modifier: u8) -> f32 {
    let threshold = match modifier {
        0 => 0.5,
        64 => 0.25,
        128 => 0.125,
        custom => (custom as f32 / 255.0).clamp(0.05, 0.95),
    };

    if phase < threshold { 1.0 } else { -1.0 }
}

fn render_noise(lfsr: &mut u16, modifier: u8) -> f32 {
    let tap = if modifier == 0 { 1 } else { 6 };
    let feedback = (*lfsr ^ (*lfsr >> tap)) & 1;
    *lfsr = (*lfsr >> 1) | (feedback << 14);

    if (*lfsr & 1) == 1 { 1.0 } else { -1.0 }
}
