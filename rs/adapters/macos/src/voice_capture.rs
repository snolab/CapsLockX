//! Microphone capture via Apple's VoiceProcessingIO Audio Unit.
//!
//! This replaces cpal for mic capture on macOS, using the VoiceProcessingIO
//! Audio Unit which provides built-in Acoustic Echo Cancellation (AEC).
//! Speaker bleed from system audio is automatically cancelled from the mic signal.
//!
//! Pure C FFI against AudioToolbox — no ObjC, no cpal.

use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};

// ── AudioToolbox C FFI ──────────────────────────────────────────────────────

extern "C" {
    fn AudioComponentFindNext(
        comp: *mut c_void,
        desc: *const AudioComponentDescription,
    ) -> *mut c_void;

    fn AudioComponentInstanceNew(
        comp: *mut c_void,
        instance: *mut *mut c_void,
    ) -> i32;

    fn AudioUnitSetProperty(
        unit: *mut c_void,
        prop: u32,
        scope: u32,
        element: u32,
        data: *const c_void,
        size: u32,
    ) -> i32;

    fn AudioUnitGetProperty(
        unit: *mut c_void,
        prop: u32,
        scope: u32,
        element: u32,
        data: *mut c_void,
        size: *mut u32,
    ) -> i32;

    fn AudioUnitInitialize(unit: *mut c_void) -> i32;
    fn AudioUnitUninitialize(unit: *mut c_void) -> i32;
    fn AudioOutputUnitStart(unit: *mut c_void) -> i32;
    fn AudioOutputUnitStop(unit: *mut c_void) -> i32;

    fn AudioUnitRender(
        unit: *mut c_void,
        flags: *mut u32,
        timestamp: *const AudioTimeStamp,
        bus: u32,
        frames: u32,
        buffers: *mut AudioBufferList,
    ) -> i32;

    fn AudioComponentInstanceDispose(unit: *mut c_void) -> i32;
}

// ── CoreAudio type definitions ──────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy)]
struct AudioComponentDescription {
    component_type: u32,
    component_sub_type: u32,
    component_manufacturer: u32,
    component_flags: u32,
    component_flags_mask: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct AudioStreamBasicDescription {
    sample_rate: f64,
    format_id: u32,
    format_flags: u32,
    bytes_per_packet: u32,
    frames_per_packet: u32,
    bytes_per_frame: u32,
    channels_per_frame: u32,
    bits_per_channel: u32,
    reserved: u32,
}

/// Minimal AudioTimeStamp — full struct size needed for AudioUnitRender.
#[repr(C)]
#[derive(Clone, Copy)]
struct AudioTimeStamp {
    sample_time: f64,
    host_time: u64,
    rate_scalar: f64,
    word_clock_time: u64,
    smpte_time: [u8; 24], // SMPTETime is 24 bytes
    flags: u32,
    reserved: u32,
}

#[repr(C)]
struct AudioBuffer {
    number_channels: u32,
    data_byte_size: u32,
    data: *mut c_void,
}

#[repr(C)]
struct AudioBufferList {
    number_buffers: u32,
    buffers: [AudioBuffer; 1],
}

#[repr(C)]
struct AURenderCallbackStruct {
    input_proc: Option<
        unsafe extern "C" fn(
            in_ref_con: *mut c_void,
            io_action_flags: *mut u32,
            in_time_stamp: *const AudioTimeStamp,
            in_bus_number: u32,
            in_number_frames: u32,
            io_data: *mut AudioBufferList,
        ) -> i32,
    >,
    input_proc_ref_con: *mut c_void,
}

// ── Constants ───────────────────────────────────────────────────────────────

const K_AUDIO_UNIT_TYPE_OUTPUT: u32 = 0x61756F75; // 'auou'
const K_AUDIO_UNIT_SUBTYPE_VOICE_PROCESSING_IO: u32 = 0x7670696F; // 'vpio'
const K_AUDIO_UNIT_MANUFACTURER_APPLE: u32 = 0x6170706C; // 'appl'

const K_AUDIO_OUTPUT_UNIT_PROPERTY_ENABLE_IO: u32 = 2003;
const K_AUDIO_UNIT_SCOPE_INPUT: u32 = 1;
const K_AUDIO_UNIT_SCOPE_OUTPUT: u32 = 0;
const K_AUDIO_UNIT_SCOPE_GLOBAL: u32 = 0;
const K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT: u32 = 8;
const K_AUDIO_OUTPUT_UNIT_PROPERTY_SET_INPUT_CALLBACK: u32 = 2005;
const K_AUDIO_UNIT_PROPERTY_SHOULD_ALLOCATE_BUFFER: u32 = 2;

const K_AUDIO_FORMAT_LINEAR_PCM: u32 = 0x6C70636D; // 'lpcm'
const K_AUDIO_FORMAT_FLAG_IS_FLOAT: u32 = 1;
const K_AUDIO_FORMAT_FLAG_IS_PACKED: u32 = 8;

// ── Shared state for the input callback ─────────────────────────────────────

/// Context passed into the AudioUnit input callback via ref_con.
struct CallbackContext {
    unit: *mut c_void,
    buffer: Arc<Mutex<Vec<f32>>>,
    render_buf: Vec<f32>,
}

// Safety: The AudioUnit pointer is only used from the callback thread and
// the Arc<Mutex<Vec<f32>>> is Send+Sync by construction.
unsafe impl Send for CallbackContext {}
unsafe impl Sync for CallbackContext {}

/// AudioUnit input callback — fires on CoreAudio's real-time I/O thread
/// whenever a buffer of echo-cancelled mic audio is available.
///
/// IMPORTANT: This runs on a real-time thread. No allocations, no I/O,
/// no long-held locks. The Mutex lock is held only to memcpy samples in.
unsafe extern "C" fn input_callback(
    in_ref_con: *mut c_void,
    io_action_flags: *mut u32,
    in_time_stamp: *const AudioTimeStamp,
    in_bus_number: u32,
    in_number_frames: u32,
    _io_data: *mut AudioBufferList,
) -> i32 {
    let ctx = &mut *(in_ref_con as *mut CallbackContext);

    let frames = in_number_frames as usize;
    let byte_size = (frames * std::mem::size_of::<f32>()) as u32;

    // Ensure render buffer is large enough (stable after first call)
    if ctx.render_buf.len() < frames {
        ctx.render_buf.resize(frames, 0.0);
    }

    let mut abl = AudioBufferList {
        number_buffers: 1,
        buffers: [AudioBuffer {
            number_channels: 1,
            data_byte_size: byte_size,
            data: ctx.render_buf.as_mut_ptr() as *mut c_void,
        }],
    };

    // Pull echo-cancelled audio from the VoiceProcessingIO unit
    let status = AudioUnitRender(
        ctx.unit,
        io_action_flags,
        in_time_stamp,
        in_bus_number, // Bus 1 = input
        in_number_frames,
        &mut abl,
    );

    if status != 0 {
        return status;
    }

    // Copy samples into shared buffer — lock held very briefly
    let samples = &ctx.render_buf[..frames];
    if let Ok(mut buf) = ctx.buffer.try_lock() {
        buf.extend_from_slice(samples);
        // Cap at ~10 seconds of audio at 16kHz to prevent unbounded growth
        const MAX_SAMPLES: usize = 16000 * 10;
        if buf.len() > MAX_SAMPLES {
            let excess = buf.len() - MAX_SAMPLES;
            buf.drain(..excess);
        }
    }
    // If try_lock fails, we drop this buffer — acceptable on a real-time thread.

    0 // noErr
}

// ── VoiceCapture ────────────────────────────────────────────────────────────

pub struct VoiceCapture {
    unit: *mut c_void,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    // Prevent CallbackContext from being freed while the AudioUnit is alive.
    // Boxed so the pointer stays stable.
    _ctx: Box<CallbackContext>,
}

// The AudioUnit is used from the main thread (start/stop) and the callback
// thread (render). CoreAudio guarantees thread safety for these operations.
unsafe impl Send for VoiceCapture {}
unsafe impl Sync for VoiceCapture {}

impl VoiceCapture {
    /// Create and configure a VoiceProcessingIO Audio Unit for echo-cancelled
    /// mic capture. The unit is initialized but not started.
    pub fn new() -> Result<Self, String> {
        unsafe {
            // ── Find VoiceProcessingIO component ────────────────────────────
            let desc = AudioComponentDescription {
                component_type: K_AUDIO_UNIT_TYPE_OUTPUT,
                component_sub_type: K_AUDIO_UNIT_SUBTYPE_VOICE_PROCESSING_IO,
                component_manufacturer: K_AUDIO_UNIT_MANUFACTURER_APPLE,
                component_flags: 0,
                component_flags_mask: 0,
            };

            let component = AudioComponentFindNext(null_mut(), &desc);
            if component.is_null() {
                return Err("VoiceProcessingIO AudioComponent not found".into());
            }

            let mut unit: *mut c_void = null_mut();
            let status = AudioComponentInstanceNew(component, &mut unit);
            if status != 0 || unit.is_null() {
                return Err(format!(
                    "AudioComponentInstanceNew failed (status {})",
                    status
                ));
            }

            eprintln!("[CLX] voice_capture: VoiceProcessingIO component created");

            // ── Enable input on Bus 1 (microphone) ─────────────────────────
            let enable: u32 = 1;
            let status = AudioUnitSetProperty(
                unit,
                K_AUDIO_OUTPUT_UNIT_PROPERTY_ENABLE_IO,
                K_AUDIO_UNIT_SCOPE_INPUT,
                1, // Bus 1 = input
                &enable as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );
            if status != 0 {
                AudioComponentInstanceDispose(unit);
                return Err(format!("Failed to enable input on Bus 1 (status {})", status));
            }

            // Leave output on Bus 0 enabled — VoiceProcessingIO uses it
            // internally as the AEC reference signal (speaker output).

            // ── Initialize the Audio Unit first, then query/set format ─────
            let status = AudioUnitInitialize(unit);
            if status != 0 {
                AudioComponentInstanceDispose(unit);
                return Err(format!("AudioUnitInitialize failed (status {})", status));
            }

            // Query the default format on Bus 1 output scope.
            let mut actual_format: AudioStreamBasicDescription = std::mem::zeroed();
            let mut size = std::mem::size_of::<AudioStreamBasicDescription>() as u32;
            let status = AudioUnitGetProperty(
                unit,
                K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT,
                K_AUDIO_UNIT_SCOPE_OUTPUT,
                1,
                &mut actual_format as *mut AudioStreamBasicDescription as *mut c_void,
                &mut size,
            );

            let actual_rate;
            if status == 0 {
                actual_rate = actual_format.sample_rate as u32;
                eprintln!("[CLX] voice_capture: hardware format: {}Hz {}ch {}bit",
                    actual_rate, actual_format.channels_per_frame, actual_format.bits_per_channel);

                // Try to set our preferred format: mono float32 at hardware rate.
                let format = AudioStreamBasicDescription {
                    sample_rate: actual_format.sample_rate,
                    format_id: K_AUDIO_FORMAT_LINEAR_PCM,
                    format_flags: K_AUDIO_FORMAT_FLAG_IS_FLOAT | K_AUDIO_FORMAT_FLAG_IS_PACKED,
                    bytes_per_packet: 4,
                    frames_per_packet: 1,
                    bytes_per_frame: 4,
                    channels_per_frame: 1,
                    bits_per_channel: 32,
                    reserved: 0,
                };
                let s = AudioUnitSetProperty(
                    unit,
                    K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT,
                    K_AUDIO_UNIT_SCOPE_OUTPUT,
                    1,
                    &format as *const AudioStreamBasicDescription as *const c_void,
                    std::mem::size_of::<AudioStreamBasicDescription>() as u32,
                );
                if s != 0 {
                    eprintln!("[CLX] voice_capture: couldn't set mono f32 (status {}), using hardware format", s);
                }
            } else {
                eprintln!("[CLX] voice_capture: couldn't query format (status {}), assuming 48kHz", status);
                actual_rate = 48000;

                // Set mono float32 at the hardware rate
                let hw_format = AudioStreamBasicDescription {
                    sample_rate: actual_rate as f64,
                    format_id: K_AUDIO_FORMAT_LINEAR_PCM,
                    format_flags: K_AUDIO_FORMAT_FLAG_IS_FLOAT | K_AUDIO_FORMAT_FLAG_IS_PACKED,
                    bytes_per_packet: 4,
                    frames_per_packet: 1,
                    bytes_per_frame: 4,
                    channels_per_frame: 1,
                    bits_per_channel: 32,
                    reserved: 0,
                };
                let status = AudioUnitSetProperty(
                    unit,
                    K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT,
                    K_AUDIO_UNIT_SCOPE_OUTPUT,
                    1,
                    &hw_format as *const AudioStreamBasicDescription as *const c_void,
                    std::mem::size_of::<AudioStreamBasicDescription>() as u32,
                );
                if status != 0 {
                    AudioComponentInstanceDispose(unit);
                    return Err(format!(
                        "Failed to set fallback stream format (status {})",
                        status
                    ));
                }
            }

            // ── Tell the AU not to allocate its own buffer — we provide ours ─
            let no_alloc: u32 = 0;
            let status = AudioUnitSetProperty(
                unit,
                K_AUDIO_UNIT_PROPERTY_SHOULD_ALLOCATE_BUFFER,
                K_AUDIO_UNIT_SCOPE_OUTPUT,
                1,
                &no_alloc as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );
            if status != 0 {
                eprintln!(
                    "[CLX] voice_capture: warning: ShouldAllocateBuffer failed (status {}), continuing",
                    status
                );
            }

            // ── Create shared buffer and callback context ──────────────────
            let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::with_capacity(
                actual_rate as usize * 2, // pre-alloc 2 seconds
            )));

            let mut ctx = Box::new(CallbackContext {
                unit,
                buffer: Arc::clone(&buffer),
                render_buf: vec![0.0f32; 1024], // initial render scratch buffer
            });

            // ── Set input callback on Bus 1 ────────────────────────────────
            let callback_struct = AURenderCallbackStruct {
                input_proc: Some(input_callback),
                input_proc_ref_con: &mut *ctx as *mut CallbackContext as *mut c_void,
            };

            let status = AudioUnitSetProperty(
                unit,
                K_AUDIO_OUTPUT_UNIT_PROPERTY_SET_INPUT_CALLBACK,
                K_AUDIO_UNIT_SCOPE_GLOBAL,
                0, // element 0 for the callback registration
                &callback_struct as *const AURenderCallbackStruct as *const c_void,
                std::mem::size_of::<AURenderCallbackStruct>() as u32,
            );
            if status != 0 {
                AudioComponentInstanceDispose(unit);
                return Err(format!(
                    "Failed to set input callback (status {})",
                    status
                ));
            }

            // (Already initialized above before format query)

            eprintln!(
                "[CLX] voice_capture: initialized ({}Hz mono, AEC enabled)",
                actual_rate
            );

            Ok(Self {
                unit,
                buffer,
                sample_rate: actual_rate,
                _ctx: ctx,
            })
        }
    }

    /// Start capturing echo-cancelled microphone audio.
    pub fn start(&self) -> Result<(), String> {
        let status = unsafe { AudioOutputUnitStart(self.unit) };
        if status != 0 {
            return Err(format!("AudioOutputUnitStart failed (status {})", status));
        }
        eprintln!("[CLX] voice_capture: started");
        Ok(())
    }

    /// Stop capturing. Buffered samples are retained until `take_samples()`.
    pub fn stop(&self) {
        unsafe {
            AudioOutputUnitStop(self.unit);
        }
        eprintln!("[CLX] voice_capture: stopped");
    }

    /// Drain and return all buffered echo-cancelled samples.
    pub fn take_samples(&self) -> Vec<f32> {
        let mut buf = self.buffer.lock().unwrap();
        std::mem::take(&mut *buf)
    }

    /// The actual sample rate (16000 if hardware supports it, otherwise the
    /// hardware native rate — resampling is handled upstream).
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl Drop for VoiceCapture {
    fn drop(&mut self) {
        if !self.unit.is_null() {
            unsafe {
                AudioOutputUnitStop(self.unit);
                AudioUnitUninitialize(self.unit);
                AudioComponentInstanceDispose(self.unit);
            }
            eprintln!("[CLX] voice_capture: disposed");
        }
    }
}

// Implement SystemAudioStream so VoiceCapture can be used via Platform trait.
impl capslockx_core::platform::SystemAudioStream for VoiceCapture {
    fn take_samples(&self) -> Vec<f32> { self.take_samples() }
    fn stop(&self) { self.stop() }
    fn sample_rate(&self) -> u32 { self.sample_rate() }
}
