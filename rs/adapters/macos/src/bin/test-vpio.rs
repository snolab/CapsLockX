/// Standalone test for VoiceProcessingIO echo cancellation.
/// Captures mic audio via VPIO and prints RMS levels continuously.
///
/// Usage: cargo run -p capslockx-macos --release --bin test-vpio

use std::ffi::c_void;
use std::sync::{Arc, Mutex};

// ── AudioToolbox FFI ────────────────────────────────────────────────────────

#[link(name = "AudioToolbox", kind = "framework")]
extern "C" {
    fn AudioComponentFindNext(comp: *mut c_void, desc: *const AudioComponentDescription) -> *mut c_void;
    fn AudioComponentInstanceNew(comp: *mut c_void, instance: *mut *mut c_void) -> i32;
    fn AudioUnitSetProperty(unit: *mut c_void, prop: u32, scope: u32, elem: u32, data: *const c_void, size: u32) -> i32;
    fn AudioUnitGetProperty(unit: *mut c_void, prop: u32, scope: u32, elem: u32, data: *mut c_void, size: *mut u32) -> i32;
    fn AudioUnitInitialize(unit: *mut c_void) -> i32;
    fn AudioOutputUnitStart(unit: *mut c_void) -> i32;
    #[allow(dead_code)]
    fn AudioOutputUnitStop(unit: *mut c_void) -> i32;
    fn AudioUnitRender(unit: *mut c_void, flags: *mut u32, ts: *const c_void, bus: u32, frames: u32, bufs: *mut c_void) -> i32;
    #[allow(dead_code)]
    fn AudioComponentInstanceDispose(unit: *mut c_void) -> i32;
}

#[repr(C)]
struct AudioComponentDescription {
    component_type: u32,
    component_sub_type: u32,
    component_manufacturer: u32,
    component_flags: u32,
    component_flags_mask: u32,
}

#[repr(C)]
#[derive(Clone)]
struct AudioStreamBasicDescription {
    sample_rate: f64, format_id: u32, format_flags: u32,
    bytes_per_packet: u32, frames_per_packet: u32, bytes_per_frame: u32,
    channels_per_frame: u32, bits_per_channel: u32, reserved: u32,
}

#[repr(C)]
struct AudioBuffer { number_channels: u32, data_byte_size: u32, data: *mut c_void }

#[repr(C)]
struct AudioBufferList { number_buffers: u32, buffers: [AudioBuffer; 1] }

#[repr(C)]
struct AURenderCallbackStruct { input_proc: *const c_void, input_proc_ref_con: *mut c_void }

struct CallbackCtx {
    unit: *mut c_void,
    buffer: Arc<Mutex<Vec<f32>>>,
    render_buf: Vec<f32>,
}
unsafe impl Send for CallbackCtx {}

static mut CTX: Option<Box<CallbackCtx>> = None;

extern "C" fn input_callback(
    _ref_con: *mut c_void, io_flags: *mut u32, ts: *const c_void,
    bus: u32, frames: u32, _: *mut c_void,
) -> i32 {
    unsafe {
        let ctx = match (*std::ptr::addr_of_mut!(CTX)).as_mut() { Some(c) => c, None => return -1 };
        let n = frames as usize;
        if ctx.render_buf.len() < n { ctx.render_buf.resize(n, 0.0); }

        // Try with 1 buffer first (mono)
        let mut abl = AudioBufferList {
            number_buffers: 1,
            buffers: [AudioBuffer {
                number_channels: 1,
                data_byte_size: (n * 4) as u32,
                data: ctx.render_buf.as_mut_ptr() as *mut c_void,
            }],
        };

        let status = AudioUnitRender(ctx.unit, io_flags, ts, bus, frames, &mut abl as *mut _ as *mut c_void);

        static COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let c = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if status != 0 {
            if c < 3 { eprintln!("\n[cb] render failed status={}, trying 9ch", status); }

            // Try with 9 buffers (non-interleaved)
            #[repr(C)]
            struct ABL9 { number_buffers: u32, buffers: [AudioBuffer; 9] }
            let mut bufs: [[f32; 2048]; 9] = [[0.0; 2048]; 9];
            let capped = n.min(2048);
            let mut abl9 = ABL9 { number_buffers: 9, buffers: std::mem::zeroed() };
            for i in 0..9 {
                abl9.buffers[i] = AudioBuffer {
                    number_channels: 1,
                    data_byte_size: (capped * 4) as u32,
                    data: bufs[i].as_mut_ptr() as *mut c_void,
                };
            }
            let status2 = AudioUnitRender(ctx.unit, io_flags, ts, bus, capped as u32, &mut abl9 as *mut _ as *mut c_void);
            if c < 3 { eprintln!("[cb] 9ch render status={}", status2); }

            if status2 == 0 {
                let samples = &bufs[0][..capped];
                if let Ok(mut buf) = ctx.buffer.try_lock() {
                    buf.extend_from_slice(samples);
                    if buf.len() > 160000 { let e = buf.len() - 160000; buf.drain(..e); }
                }

                if c < 5 {
                    let rms: f32 = (samples.iter().map(|s| s*s).sum::<f32>() / samples.len().max(1) as f32).sqrt();
                    eprintln!("[cb] 9ch ch0 rms={:.4}", rms);
                }
            }
            return 0;
        }

        // 1ch success
        let samples = &ctx.render_buf[..n];
        if let Ok(mut buf) = ctx.buffer.try_lock() {
            buf.extend_from_slice(samples);
            if buf.len() > 160000 { let e = buf.len() - 160000; buf.drain(..e); }
        }
        0
    }
}

fn write_wav(path: &str, samples: &[f32], sample_rate: u32) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(path)?;
    let n = samples.len() as u32;
    let byte_rate = sample_rate * 2;
    let data_size = n * 2;
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_size).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;        // PCM
    f.write_all(&1u16.to_le_bytes())?;        // mono
    f.write_all(&sample_rate.to_le_bytes())?;
    f.write_all(&byte_rate.to_le_bytes())?;
    f.write_all(&2u16.to_le_bytes())?;        // block align
    f.write_all(&16u16.to_le_bytes())?;       // bits
    f.write_all(b"data")?;
    f.write_all(&data_size.to_le_bytes())?;
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        f.write_all(&v.to_le_bytes())?;
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut record_secs: Option<u32> = None;
    let mut out_path: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--record" => { record_secs = args.get(i+1).and_then(|s| s.parse().ok()); i += 2; }
            "--out"    => { out_path = args.get(i+1).cloned(); i += 2; }
            _ => i += 1,
        }
    }
    let record_mode = record_secs.is_some() && out_path.is_some();

    eprintln!("[test-vpio] VoiceProcessingIO Echo Cancellation Test");
    if record_mode {
        eprintln!("[test-vpio] Record mode: {}s → {}\n", record_secs.unwrap(), out_path.as_ref().unwrap());
    } else {
        eprintln!("[test-vpio] Speak → non-zero RMS. Speakers only → low RMS = AEC working.\n");
    }

    unsafe {
        let desc = AudioComponentDescription {
            component_type: 0x61756F75,     // 'auou'
            component_sub_type: 0x7670696F, // 'vpio'
            component_manufacturer: 0x6170706C, // 'appl'
            component_flags: 0, component_flags_mask: 0,
        };
        let comp = AudioComponentFindNext(std::ptr::null_mut(), &desc);
        if comp.is_null() { eprintln!("VPIO not found!"); return; }

        let mut unit: *mut c_void = std::ptr::null_mut();
        let s = AudioComponentInstanceNew(comp, &mut unit);
        if s != 0 { eprintln!("Instance failed: {s}"); return; }
        eprintln!("[test-vpio] Component created");

        // Enable input on Bus 1
        let enable: u32 = 1;
        AudioUnitSetProperty(unit, 2003, 1, 1, &enable as *const _ as *const c_void, 4);

        // Initialize
        let s = AudioUnitInitialize(unit);
        if s != 0 { eprintln!("Init failed: {s}"); return; }
        eprintln!("[test-vpio] Initialized");

        // Minimize ducking
        #[repr(C)] struct Duck { adv: u8, level: u32 }
        let duck = Duck { adv: 0, level: 10 };
        let s = AudioUnitSetProperty(unit, 2108, 0, 0, &duck as *const _ as *const c_void, std::mem::size_of::<Duck>() as u32);
        eprintln!("[test-vpio] Ducking config: status={s}");

        // Query all scopes/buses for format
        for &(scope, bus, name) in &[(0u32, 0u32, "Global/0"), (1, 0, "Input/0"), (0, 1, "Output/1"), (1, 1, "Input/1")] {
            let mut fmt: AudioStreamBasicDescription = std::mem::zeroed();
            let mut sz = std::mem::size_of::<AudioStreamBasicDescription>() as u32;
            let s = AudioUnitGetProperty(unit, 8, scope, bus, &mut fmt as *mut _ as *mut c_void, &mut sz);
            if s == 0 {
                eprintln!("[test-vpio] {name}: {}Hz {}ch {}bit flags={:#x} bpf={}",
                    fmt.sample_rate as u32, fmt.channels_per_frame, fmt.bits_per_channel, fmt.format_flags, fmt.bytes_per_frame);
            } else {
                eprintln!("[test-vpio] {name}: query failed ({s})");
            }
        }

        // Set up callback
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        CTX = Some(Box::new(CallbackCtx {
            unit, buffer: buffer.clone(), render_buf: Vec::new(),
        }));

        let cb = AURenderCallbackStruct {
            input_proc: input_callback as *const c_void,
            input_proc_ref_con: std::ptr::null_mut(),
        };
        let s = AudioUnitSetProperty(unit, 2005, 0, 1, &cb as *const _ as *const c_void, std::mem::size_of::<AURenderCallbackStruct>() as u32);
        eprintln!("[test-vpio] Callback set: status={s}");

        let s = AudioOutputUnitStart(unit);
        eprintln!("[test-vpio] Started: status={s}\n");

        // Record mode: capture N seconds of (gain-applied) mono 48kHz, save WAV, exit.
        if record_mode {
            let secs = record_secs.unwrap();
            let path = out_path.unwrap();
            let target = (48000 * secs) as usize;
            let mut all: Vec<f32> = Vec::with_capacity(target);
            let start = std::time::Instant::now();
            while all.len() < target {
                std::thread::sleep(std::time::Duration::from_millis(50));
                let chunk = { let mut b = buffer.lock().unwrap(); std::mem::take(&mut *b) };
                if chunk.is_empty() { continue; }
                let amplified: Vec<f32> = chunk.iter().map(|&s| (s * 30.0).clamp(-1.0, 1.0)).collect();
                let rms: f32 = (amplified.iter().map(|s| s*s).sum::<f32>() / amplified.len() as f32).sqrt();
                let bar_len = (rms * 30.0).min(30.0) as usize;
                let bar = "█".repeat(bar_len) + &"░".repeat(30 - bar_len);
                eprint!("\r[{bar}] rms={rms:.4} {:>4}/{}s", start.elapsed().as_secs(), secs);
                all.extend_from_slice(&amplified);
            }
            eprintln!();
            all.truncate(target);
            let rms_total: f32 = (all.iter().map(|s| s*s).sum::<f32>() / all.len() as f32).sqrt();
            let peak: f32 = all.iter().fold(0f32, |a, &b| a.max(b.abs()));
            write_wav(&path, &all, 48000).expect("wav write failed");
            eprintln!("[test-vpio] saved {} ({} samples, rms={:.4}, peak={:.4})",
                path, all.len(), rms_total, peak);
            std::process::exit(0);
        }

        // Load Whisper for transcription test
        eprintln!("[test-vpio] Loading Whisper...");
        let mut whisper = capslockx_core::local_whisper::LocalWhisper::new()
            .expect("whisper failed");
        eprintln!("[test-vpio] Whisper ready. Transcribing every 3s...\n");

        let mut accum: Vec<f32> = Vec::new();
        let mut iter = 0u32;

        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let samples = {
                let mut buf = buffer.lock().unwrap();
                std::mem::take(&mut *buf)
            };
            if samples.is_empty() { continue; }

            // Apply 30x gain (VPIO output is very quiet after AEC)
            let amplified: Vec<f32> = samples.iter()
                .map(|&s| (s * 30.0).clamp(-1.0, 1.0))
                .collect();

            let rms: f32 = (amplified.iter().map(|s| s * s).sum::<f32>() / amplified.len() as f32).sqrt();
            let bar_len = (rms * 30.0).min(30.0) as usize;
            let bar = "█".repeat(bar_len) + &"░".repeat(30 - bar_len);
            eprint!("\r[{bar}] rms={rms:.4} n={}   ", amplified.len());

            // Resample 48kHz → 16kHz for Whisper
            let ratio = 48000.0 / 16000.0;
            let out_len = (amplified.len() as f64 / ratio) as usize;
            let resampled: Vec<f32> = (0..out_len).map(|i| {
                let src = i as f64 * ratio;
                let idx = src as usize;
                let frac = (src - idx as f64) as f32;
                let s0 = amplified.get(idx).copied().unwrap_or(0.0);
                let s1 = amplified.get(idx + 1).copied().unwrap_or(s0);
                s0 + (s1 - s0) * frac
            }).collect();

            accum.extend_from_slice(&resampled);

            // Transcribe every 3s (48000 samples at 16kHz)
            if accum.len() >= 48000 {
                iter += 1;
                let mut padded = accum.clone();
                if padded.len() < 16000 { padded.resize(16000, 0.0); }

                match whisper.transcribe(&padded) {
                    Ok(text) if !text.is_empty() => {
                        eprintln!("\n[iter {}] 🎤 VPIO: \"{}\"", iter, text);
                    }
                    Ok(_) => {
                        eprintln!("\n[iter {}] 🎤 VPIO: (empty/noise)", iter);
                    }
                    Err(e) => {
                        eprintln!("\n[iter {}] 🎤 VPIO error: {}", iter, e);
                    }
                }
                accum.clear();
            }
        }
    }
}
