mod audio;
mod game;
mod pitch;
mod song;
mod tab;

use audio::get_scarlet;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use game::{Game, StepResult};
use pitch_detection::detector::yin::YINDetector;
use pitch_detection::detector::PitchDetector;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const WINDOW_SIZE: usize = 4096;
const PROCESS_INTERVAL_MS: u64 = 50;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    // let device = get_scarlet(&host)
    //     .or_else(|| host.default_input_device())
    //     .expect("No input device available");
    let device = host.default_input_device().unwrap();

    let config = device.default_input_config()?;
    let sample_rate = config.sample_rate();
    let stream_config = config.into();
    let sample_queue = Arc::new(Mutex::new(VecDeque::<f32>::with_capacity(WINDOW_SIZE * 2)));

    let stream = device.build_input_stream(
        stream_config,
        {
            let queue = sample_queue.clone();
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut q = queue.lock().unwrap();
                for &sample in data {
                    q.push_back(sample);
                    if q.len() > WINDOW_SIZE * 4 {
                        q.drain(..WINDOW_SIZE);
                    }
                }
            }
        },
        move |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    stream.play()?;

    let detector_queue = sample_queue.clone();
    let handle = thread::spawn(move || {
        let mut detector = YINDetector::new(WINDOW_SIZE, WINDOW_SIZE);
        let mut buffer = vec![0.0_f64; WINDOW_SIZE];
        let mut game = Game::new(song::wonderful_tonight());

        loop {
            if game.is_complete() {
                break;
            }

            thread::sleep(Duration::from_millis(PROCESS_INTERVAL_MS));

            let maybe_samples = {
                let mut q = detector_queue.lock().unwrap();
                if q.len() >= WINDOW_SIZE {
                    let samples: Vec<f64> = q.drain(0..WINDOW_SIZE).map(|s| s as f64).collect();
                    Some(samples)
                } else {
                    None
                }
            };

            if let Some(samples) = maybe_samples {
                buffer.copy_from_slice(&samples);
                let power_threshold = 0.01;
                let clarity_threshold = 0.1;
                if let Some(pitch) = detector.get_pitch(
                    &buffer,
                    sample_rate as usize,
                    power_threshold,
                    clarity_threshold,
                ) {
                    match game.on_pitch(pitch.frequency) {
                        StepResult::SongComplete => break,
                        StepResult::NoMatch
                        | StepResult::Advanced
                        | StepResult::WindowBumped => {}
                    }
                }
            }
        }
    });

    handle.join().unwrap();
    Ok(())
}
