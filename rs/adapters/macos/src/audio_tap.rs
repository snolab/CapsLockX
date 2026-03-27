//! System audio capture via Core Audio Taps (macOS 14.2+).
//!
//! Uses AudioHardwareCreateProcessTap to capture the exact digital audio
//! going to speakers — zero acoustic distortion, perfect for AEC reference.
//! Falls back to ScreenCaptureKit if taps are unavailable.

use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::{Mutex, atomic::{AtomicBool, Ordering}};
use capslockx_core::platform::SystemAudioStream;

// ── Raw FFI declarations ────────────────────────────────────────────────────

type AudioObjectID = u32;
type OSStatus = i32;

#[repr(C)]
#[derive(Clone, Copy)]
struct AudioObjectPropertyAddress {
    m_selector: u32,
    m_scope: u32,
    m_element: u32,
}

#[repr(C)]
#[allow(dead_code)]
struct AudioBufferList {
    m_number_buffers: u32,
    m_buffers: [AudioBuffer; 1],
}

#[repr(C)]
#[allow(dead_code)]
struct AudioBuffer {
    m_number_channels: u32,
    m_data_byte_size: u32,
    m_data: *mut c_void,
}

#[repr(C)]
struct AudioTimeStamp {
    _data: [u8; 64], // opaque
}

extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
    fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;

    fn AudioHardwareCreateProcessTap(
        description: *mut c_void,
        tap_id: *mut AudioObjectID,
    ) -> OSStatus;

    fn AudioHardwareDestroyProcessTap(tap_id: AudioObjectID) -> OSStatus;

    fn AudioHardwareCreateAggregateDevice(
        description: *const c_void, // CFDictionaryRef
        device_id: *mut AudioObjectID,
    ) -> OSStatus;

    fn AudioHardwareDestroyAggregateDevice(device_id: AudioObjectID) -> OSStatus;

    fn AudioDeviceCreateIOProcID(
        device: AudioObjectID,
        proc_fn: unsafe extern "C" fn(
            AudioObjectID, *const AudioTimeStamp,
            *const AudioBufferList, *const AudioTimeStamp,
            *mut AudioBufferList, *const AudioTimeStamp,
            *mut c_void,
        ) -> OSStatus,
        client_data: *mut c_void,
        proc_id: *mut *mut c_void,
    ) -> OSStatus;

    fn AudioDeviceStart(device: AudioObjectID, proc_id: *mut c_void) -> OSStatus;
    fn AudioDeviceStop(device: AudioObjectID, proc_id: *mut c_void) -> OSStatus;
    fn AudioDeviceDestroyIOProcID(device: AudioObjectID, proc_id: *mut c_void) -> OSStatus;

    fn AudioObjectGetPropertyData(
        object_id: AudioObjectID,
        address: *const AudioObjectPropertyAddress,
        qualifier_data_size: u32,
        qualifier_data: *const c_void,
        data_size: *mut u32,
        data: *mut c_void,
    ) -> OSStatus;

    fn AudioObjectGetPropertyDataSize(
        object_id: AudioObjectID,
        address: *const AudioObjectPropertyAddress,
        qualifier_data_size: u32,
        qualifier_data: *const c_void,
        data_size: *mut u32,
    ) -> OSStatus;
}

// CoreFoundation FFI
extern "C" {
    fn CFDictionaryCreateMutable(
        allocator: *const c_void,
        capacity: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> *mut c_void;
    fn CFDictionarySetValue(dict: *mut c_void, key: *const c_void, value: *const c_void);
    fn CFStringCreateWithCString(
        allocator: *const c_void,
        c_str: *const std::ffi::c_char,
        encoding: u32,
    ) -> *mut c_void;
    fn CFNumberCreate(
        allocator: *const c_void,
        the_type: isize,
        value_ptr: *const c_void,
    ) -> *mut c_void;
    fn CFArrayCreateMutable(
        allocator: *const c_void,
        capacity: isize,
        callbacks: *const c_void,
    ) -> *mut c_void;
    fn CFArrayAppendValue(array: *mut c_void, value: *const c_void);
    fn CFRelease(cf: *const c_void);

    static kCFTypeDictionaryKeyCallBacks: c_void;
    static kCFTypeDictionaryValueCallBacks: c_void;
    static kCFTypeArrayCallBacks: c_void;
}

const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;

// AudioObject constants
const K_AUDIO_OBJECT_SYSTEM_OBJECT: AudioObjectID = 1;
const K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL: u32 = 0x676C6F62; // 'glob'
const K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN: u32 = 0x6D61696E; // 'main'
const K_AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE: u32 = 0x644F7574; // 'dOut'
const K_AUDIO_DEVICE_PROPERTY_DEVICE_UID: u32 = 0x75696420; // 'uid '
const K_AUDIO_TAP_PROPERTY_UID: u32 = 0x74756964; // 'tuid'

unsafe fn sel(name: &[u8]) -> *mut c_void { sel_registerName(name.as_ptr() as *const _) }
unsafe fn cls(name: &[u8]) -> *mut c_void { objc_getClass(name.as_ptr() as *const _) }

fn cfstr(s: &str) -> *mut c_void {
    let cstr = std::ffi::CString::new(s).unwrap();
    unsafe { CFStringCreateWithCString(null_mut(), cstr.as_ptr(), K_CF_STRING_ENCODING_UTF8) }
}

fn cfnum_i32(val: i32) -> *mut c_void {
    unsafe { CFNumberCreate(null_mut(), 3 /* kCFNumberSInt32Type */, &val as *const _ as *const c_void) }
}

fn cfbool(val: bool) -> *mut c_void {
    // kCFBooleanTrue / kCFBooleanFalse
    extern "C" {
        static kCFBooleanTrue: *mut c_void;
        static kCFBooleanFalse: *mut c_void;
    }
    unsafe { if val { kCFBooleanTrue } else { kCFBooleanFalse } }
}

// ── Shared sample buffer ────────────────────────────────────────────────────

static TAP_AUDIO_BUF: Mutex<Vec<f32>> = Mutex::new(Vec::new());
static TAP_ACTIVE: AtomicBool = AtomicBool::new(false);

// ── IO Proc callback ────────────────────────────────────────────────────────

unsafe extern "C" fn io_callback(
    _device: AudioObjectID,
    _now: *const AudioTimeStamp,
    input_data: *const AudioBufferList,
    _input_time: *const AudioTimeStamp,
    _output_data: *mut AudioBufferList,
    _output_time: *const AudioTimeStamp,
    _client_data: *mut c_void,
) -> OSStatus {
    if input_data.is_null() { return 0; }
    let buf_list = &*input_data;
    if buf_list.m_number_buffers == 0 { return 0; }
    let buf = &buf_list.m_buffers[0];
    if buf.m_data.is_null() || buf.m_data_byte_size == 0 { return 0; }

    let sample_count = buf.m_data_byte_size as usize / 4; // f32 = 4 bytes
    let samples = std::slice::from_raw_parts(buf.m_data as *const f32, sample_count);

    // If stereo, mix down to mono
    let channels = buf.m_number_channels as usize;
    let mono: Vec<f32> = if channels > 1 {
        samples.chunks(channels)
            .map(|ch| ch.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        samples.to_vec()
    };

    if let Ok(mut buf) = TAP_AUDIO_BUF.lock() {
        // Cap buffer at 5 seconds of 48kHz audio to prevent unbounded growth
        const MAX_SAMPLES: usize = 48000 * 5;
        if buf.len() + mono.len() > MAX_SAMPLES {
            let drain = (buf.len() + mono.len()) - MAX_SAMPLES;
            let drain = drain.min(buf.len());
            buf.drain(..drain);
        }
        buf.extend_from_slice(&mono);
    }

    0
}

// ── Helper: get default output device UID ───────────────────────────────────

fn get_default_output_device_uid() -> Result<(AudioObjectID, String), String> {
    unsafe {
        // Get default output device ID
        let addr = AudioObjectPropertyAddress {
            m_selector: K_AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE,
            m_scope: K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            m_element: K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };
        let mut device_id: AudioObjectID = 0;
        let mut size = std::mem::size_of::<AudioObjectID>() as u32;
        let status = AudioObjectGetPropertyData(
            K_AUDIO_OBJECT_SYSTEM_OBJECT, &addr,
            0, null_mut(), &mut size, &mut device_id as *mut _ as *mut c_void,
        );
        if status != 0 {
            return Err(format!("Failed to get default output device: {}", status));
        }

        // Get device UID (CFStringRef)
        let uid_addr = AudioObjectPropertyAddress {
            m_selector: K_AUDIO_DEVICE_PROPERTY_DEVICE_UID,
            m_scope: K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            m_element: K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };
        let mut uid_ref: *mut c_void = null_mut();
        let mut uid_size = std::mem::size_of::<*mut c_void>() as u32;
        let status = AudioObjectGetPropertyData(
            device_id, &uid_addr,
            0, null_mut(), &mut uid_size, &mut uid_ref as *mut _ as *mut c_void,
        );
        if status != 0 || uid_ref.is_null() {
            return Err(format!("Failed to get device UID: {}", status));
        }

        // Convert CFString to Rust String
        let f: extern "C" fn(*mut c_void, *mut c_void) -> *const std::ffi::c_char =
            std::mem::transmute(objc_msgSend as *const ());
        let utf8 = f(uid_ref, sel(b"UTF8String\0"));
        let uid = if utf8.is_null() {
            CFRelease(uid_ref);
            return Err("Device UID UTF8String is null".into());
        } else {
            let s = std::ffi::CStr::from_ptr(utf8).to_string_lossy().into_owned();
            CFRelease(uid_ref);
            s
        };

        Ok((device_id, uid))
    }
}

// ── Helper: get tap UID ─────────────────────────────────────────────────────

fn get_tap_uid(tap_id: AudioObjectID) -> Result<String, String> {
    unsafe {
        let addr = AudioObjectPropertyAddress {
            m_selector: K_AUDIO_TAP_PROPERTY_UID,
            m_scope: K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            m_element: K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };
        let mut uid_ref: *mut c_void = null_mut();
        let mut size = std::mem::size_of::<*mut c_void>() as u32;
        let status = AudioObjectGetPropertyData(
            tap_id, &addr,
            0, null_mut(), &mut size, &mut uid_ref as *mut _ as *mut c_void,
        );
        if status != 0 || uid_ref.is_null() {
            return Err(format!("Failed to get tap UID: {}", status));
        }
        let f: extern "C" fn(*mut c_void, *mut c_void) -> *const std::ffi::c_char =
            std::mem::transmute(objc_msgSend as *const ());
        let utf8 = f(uid_ref, sel(b"UTF8String\0"));
        if utf8.is_null() {
            CFRelease(uid_ref);
            return Err("Tap UID UTF8String is null".into());
        }
        let s = std::ffi::CStr::from_ptr(utf8).to_string_lossy().into_owned();
        CFRelease(uid_ref);
        Ok(s)
    }
}

// ── AudioTapCapture ─────────────────────────────────────────────────────────

pub struct AudioTapCapture {
    tap_id: AudioObjectID,
    aggregate_id: AudioObjectID,
    proc_id: *mut c_void,
}

unsafe impl Send for AudioTapCapture {}

impl AudioTapCapture {
    /// Create a Core Audio Tap capturing all system audio.
    /// Returns Err if macOS < 14.2 or the API is unavailable.
    pub fn new() -> Result<Self, String> {
        // Check if CATapDescription class exists (macOS 14.2+)
        let tap_desc_cls = unsafe { cls(b"CATapDescription\0") };
        if tap_desc_cls.is_null() {
            return Err("CATapDescription not available (requires macOS 14.2+)".into());
        }

        let (_device_id, _output_uid) = get_default_output_device_uid()?;
        eprintln!("[CLX] audio_tap: default output device: {:?}", _output_uid);

        unsafe {
            // ── Step 1: Create CATapDescription for global stereo tap ────────
            let desc = {
                let alloc = {
                    let f0: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
                        std::mem::transmute(objc_msgSend as *const ());
                    f0(tap_desc_cls, sel(b"alloc\0"))
                };

                // Create empty NSArray for exclusion list
                let nsarray_cls = cls(b"NSArray\0");
                let f0: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
                    std::mem::transmute(objc_msgSend as *const ());
                let empty_array = f0(nsarray_cls, sel(b"array\0"));

                // initStereoGlobalTapButExcludeProcesses:
                let f1: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
                    std::mem::transmute(objc_msgSend as *const ());
                let desc = f1(alloc, sel(b"initStereoGlobalTapButExcludeProcesses:\0"), empty_array);
                if desc.is_null() {
                    return Err("CATapDescription init returned nil".into());
                }

                // Set mute behavior to unmuted (0 = CATapUnmuted)
                let f_set_i64: extern "C" fn(*mut c_void, *mut c_void, i64) =
                    std::mem::transmute(objc_msgSend as *const ());
                f_set_i64(desc, sel(b"setMuteBehavior:\0"), 0);

                desc
            };

            // ── Step 2: Create the process tap ───────────────────────────────
            let mut tap_id: AudioObjectID = 0;
            let status = AudioHardwareCreateProcessTap(desc, &mut tap_id);
            if status != 0 {
                return Err(format!("AudioHardwareCreateProcessTap failed: {}", status));
            }
            eprintln!("[CLX] audio_tap: process tap created (id={})", tap_id);

            let tap_uid = get_tap_uid(tap_id)?;
            eprintln!("[CLX] audio_tap: tap UID: {}", tap_uid);

            // ── Step 3: Create aggregate device with tap ─────────────────────
            let agg_dict = {
                let dict = CFDictionaryCreateMutable(
                    null_mut(), 0,
                    &kCFTypeDictionaryKeyCallBacks, &kCFTypeDictionaryValueCallBacks,
                );

                CFDictionarySetValue(dict, cfstr("uid") as _, cfstr("com.snomiao.clx.audiotap") as _);
                CFDictionarySetValue(dict, cfstr("name") as _, cfstr("CLX-AudioTap") as _);
                CFDictionarySetValue(dict, cfstr("private") as _, cfbool(true) as _);
                CFDictionarySetValue(dict, cfstr("stacked") as _, cfbool(false) as _);
                CFDictionarySetValue(dict, cfstr("tapautostart") as _, cfbool(true) as _);

                // Tap list: [{ uid: "<tap_uid>" }]
                let tap_entry = CFDictionaryCreateMutable(
                    null_mut(), 0,
                    &kCFTypeDictionaryKeyCallBacks, &kCFTypeDictionaryValueCallBacks,
                );
                CFDictionarySetValue(tap_entry, cfstr("uid") as _, cfstr(&tap_uid) as _);

                let tap_array = CFArrayCreateMutable(null_mut(), 0, &kCFTypeArrayCallBacks);
                CFArrayAppendValue(tap_array, tap_entry as _);
                CFDictionarySetValue(dict, cfstr("taps") as _, tap_array as _);

                dict
            };

            let mut aggregate_id: AudioObjectID = 0;
            let status = AudioHardwareCreateAggregateDevice(agg_dict as _, &mut aggregate_id);
            CFRelease(agg_dict);
            if status != 0 {
                AudioHardwareDestroyProcessTap(tap_id);
                return Err(format!("AudioHardwareCreateAggregateDevice failed: {}", status));
            }
            eprintln!("[CLX] audio_tap: aggregate device created (id={})", aggregate_id);

            // ── Step 4: Create IO proc and start ─────────────────────────────
            let mut proc_id: *mut c_void = null_mut();
            let status = AudioDeviceCreateIOProcID(
                aggregate_id, io_callback, null_mut(), &mut proc_id,
            );
            if status != 0 {
                AudioHardwareDestroyAggregateDevice(aggregate_id);
                AudioHardwareDestroyProcessTap(tap_id);
                return Err(format!("AudioDeviceCreateIOProcID failed: {}", status));
            }

            let status = AudioDeviceStart(aggregate_id, proc_id);
            if status != 0 {
                AudioDeviceDestroyIOProcID(aggregate_id, proc_id);
                AudioHardwareDestroyAggregateDevice(aggregate_id);
                AudioHardwareDestroyProcessTap(tap_id);
                return Err(format!("AudioDeviceStart failed: {}", status));
            }

            TAP_ACTIVE.store(true, Ordering::Relaxed);
            eprintln!("[CLX] audio_tap: Core Audio Tap started (capturing all system audio)");

            Ok(Self { tap_id, aggregate_id, proc_id })
        }
    }
}

impl SystemAudioStream for AudioTapCapture {
    fn take_samples(&self) -> Vec<f32> {
        let mut buf = TAP_AUDIO_BUF.lock().unwrap();
        std::mem::take(&mut *buf)
    }

    fn stop(&self) {
        TAP_ACTIVE.store(false, Ordering::Relaxed);
        unsafe {
            AudioDeviceStop(self.aggregate_id, self.proc_id);
            AudioDeviceDestroyIOProcID(self.aggregate_id, self.proc_id);
            AudioHardwareDestroyAggregateDevice(self.aggregate_id);
            AudioHardwareDestroyProcessTap(self.tap_id);
        }
        eprintln!("[CLX] audio_tap: stopped");
    }

    fn sample_rate(&self) -> u32 {
        48000 // Core Audio taps use the output device sample rate (typically 48kHz)
    }
}

impl Drop for AudioTapCapture {
    fn drop(&mut self) {
        if TAP_ACTIVE.load(Ordering::Relaxed) {
            self.stop();
        }
    }
}
