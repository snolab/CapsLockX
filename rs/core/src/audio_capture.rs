//! Cross-platform audio capture using `cpal`.
//!
//! Captures microphone audio as mono f32 samples, targeting 16 kHz for Whisper.
//! Falls back to the device's default config if 16 kHz is unavailable.
//!
//! Uses a lock-free SPSC ring buffer between the cpal callback thread and
//! consumers. The audio callback must never block on a user-space lock — if a
//! consumer holds one across slow I/O while the callback is also trying to take
//! it, the callback thread stalls and a subsequent `stream.pause()/drop()` can
//! wedge waiting to join the callback thread. A lock-free ring avoids that
//! entire failure mode.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use ringbuf::{traits::*, HeapCons, HeapRb};

pub struct AudioCapture {
    recording: Arc<AtomicBool>,
    cons: Arc<Mutex<HeapCons<f32>>>,
    sample_rate: u32,
    stream: Option<Stream>,
}

impl AudioCapture {
    /// Create a new AudioCapture that targets the default input device.
    /// The stream is built but not started until `start()` is called.
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| "No default audio input device found".to_string())?;

        let default_config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input config: {e}"))?;

        // Try 16 kHz mono f32 first (ideal for Whisper); fall back to device default.
        let (config, sample_rate) = {
            let desired = StreamConfig {
                channels: 1,
                sample_rate: cpal::SampleRate(16000),
                buffer_size: cpal::BufferSize::Default,
            };
            let supported = device.supported_input_configs();
            let has_16k = supported.map_or(false, |mut cfgs| {
                cfgs.any(|c| {
                    c.channels() == 1
                        && c.min_sample_rate().0 <= 16000
                        && c.max_sample_rate().0 >= 16000
                        && c.sample_format() == SampleFormat::F32
                })
            });
            if has_16k {
                (desired, 16000u32)
            } else {
                let sr = default_config.sample_rate().0;
                let ch = default_config.channels();
                (
                    StreamConfig {
                        channels: ch,
                        sample_rate: cpal::SampleRate(sr),
                        buffer_size: cpal::BufferSize::Default,
                    },
                    sr,
                )
            }
        };

        let recording = Arc::new(AtomicBool::new(false));

        // Ring buffer sized for ~10 s of mono audio at the target rate. Consumers
        // typically drain every 50–200 ms, so this is generous head-room.
        let rb_capacity = (sample_rate as usize).saturating_mul(10).max(16000);
        let rb: HeapRb<f32> = HeapRb::new(rb_capacity);
        let (mut prod, cons) = rb.split();
        let cons = Arc::new(Mutex::new(cons));

        let rec_flag = Arc::clone(&recording);
        let channels = config.channels as usize;
        let mut downmix: Vec<f32> = Vec::with_capacity(4096);

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if !rec_flag.load(Ordering::Relaxed) {
                        return;
                    }
                    if channels == 1 {
                        let _ = prod.push_slice(data);
                    } else {
                        // Down-mix to mono by averaging channels. Reuse the
                        // scratch buffer to avoid per-callback allocation.
                        downmix.clear();
                        downmix.reserve(data.len() / channels);
                        for chunk in data.chunks(channels) {
                            let sum: f32 = chunk.iter().sum();
                            downmix.push(sum / channels as f32);
                        }
                        let _ = prod.push_slice(&downmix);
                    }
                },
                move |err| {
                    eprintln!("[CLX] audio capture error: {err}");
                },
                None,
            )
            .map_err(|e| format!("Failed to build input stream: {e}"))?;

        Ok(Self {
            recording,
            cons,
            sample_rate,
            stream: Some(stream),
        })
    }

    /// Start capturing audio from the microphone.
    pub fn start(&self) -> Result<(), String> {
        if let Some(ref stream) = self.stream {
            stream
                .play()
                .map_err(|e| format!("Failed to start audio stream: {e}"))?;
            self.recording.store(true, Ordering::Relaxed);
            Ok(())
        } else {
            Err("No audio stream available".to_string())
        }
    }

    /// Stop capturing audio. Buffered samples are retained until `take_samples()`.
    pub fn stop(&self) {
        self.recording.store(false, Ordering::Relaxed);
        if let Some(ref stream) = self.stream {
            let _ = stream.pause();
        }
    }

    /// Drain and return all buffered samples collected so far.
    pub fn take_samples(&self) -> Vec<f32> {
        let mut cons = self.cons.lock().unwrap();
        let n = cons.occupied_len();
        if n == 0 {
            return Vec::new();
        }
        let mut out = vec![0.0f32; n];
        let got = cons.pop_slice(&mut out);
        out.truncate(got);
        out
    }

    /// Whether the capture is currently recording.
    pub fn is_recording(&self) -> bool {
        self.recording.load(Ordering::Relaxed)
    }

    /// The actual sample rate of the capture stream (may differ from 16000
    /// if the device did not support it).
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
