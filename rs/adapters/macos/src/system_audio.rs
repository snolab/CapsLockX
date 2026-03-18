//! System audio capture via ScreenCaptureKit (macOS 12.3+).
//!
//! Captures what's playing through the system speakers — meetings,
//! YouTube, music, etc. Mixed with mic audio for voice transcription.

use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicPtr, Ordering}};
use capslockx_core::platform::SystemAudioStream;

// ── ObjC runtime FFI ─────────────────────────────────────────────────────────

extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
    fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
    fn objc_allocateClassPair(sup: *mut c_void, name: *const std::ffi::c_char, extra: usize) -> *mut c_void;
    fn objc_registerClassPair(cls: *mut c_void);
    fn class_addMethod(cls: *mut c_void, sel: *mut c_void, imp: *const c_void, types: *const std::ffi::c_char) -> bool;
    fn class_addProtocol(cls: *mut c_void, protocol: *mut c_void) -> bool;
    fn objc_getProtocol(name: *const std::ffi::c_char) -> *mut c_void;
    fn objc_retain(obj: *mut c_void) -> *mut c_void;
}

// CoreMedia FFI
extern "C" {
    fn CMSampleBufferGetDataBuffer(sbuf: *mut c_void) -> *mut c_void;
    fn CMBlockBufferGetDataPointer(
        block: *mut c_void,
        offset: usize,
        length_at_offset: *mut usize,
        total_length: *mut usize,
        data_pointer: *mut *mut u8,
    ) -> i32;
}

// CoreFoundation / GCD FFI
extern "C" {
    fn dispatch_semaphore_create(value: isize) -> *mut c_void;
    fn dispatch_semaphore_wait(dsema: *mut c_void, timeout: u64) -> isize;
    fn dispatch_semaphore_signal(dsema: *mut c_void) -> isize;
    fn CFArrayGetCount(array: *mut c_void) -> isize;
    fn CFArrayGetValueAtIndex(array: *mut c_void, idx: isize) -> *mut c_void;
}

const DISPATCH_TIME_FOREVER: u64 = !0;

// Block runtime — _NSConcreteGlobalBlock is in libSystem
#[link(name = "System", kind = "dylib")]
extern "C" {
    static _NSConcreteGlobalBlock: *const c_void;
}

unsafe fn sel(name: &[u8]) -> *mut c_void { sel_registerName(name.as_ptr() as *const _) }
unsafe fn cls(name: &[u8]) -> *mut c_void { objc_getClass(name.as_ptr() as *const _) }
unsafe fn msg0(obj: *mut c_void, s: *mut c_void) -> *mut c_void {
    let f: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
    f(obj, s)
}

// ── ObjC Block types for completion handlers ────────────────────────────────

/// Minimal ObjC block layout (global block — no captures, lives in static memory).
#[repr(C)]
struct GlobalBlock {
    isa: *const *const c_void,
    flags: i32,
    reserved: i32,
    invoke: *const c_void,
    descriptor: *const BlockDescriptor,
}

// Safety: GlobalBlock only contains raw pointers to static data and function pointers.
unsafe impl Sync for GlobalBlock {}

#[repr(C)]
struct BlockDescriptor {
    reserved: u64,
    size: u64,
}

static BLOCK_DESCRIPTOR: BlockDescriptor = BlockDescriptor {
    reserved: 0,
    size: std::mem::size_of::<GlobalBlock>() as u64,
};

// ── SCShareableContent async result ─────────────────────────────────────────

static CONTENT_RESULT: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static CONTENT_SEM: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

/// Completion handler for `getShareableContentExcludingDesktopWindows:onScreenWindowsOnly:completionHandler:`
/// Signature: ^(SCShareableContent * _Nullable content, NSError * _Nullable error)
unsafe extern "C" fn shareable_content_completion(
    _block: *mut GlobalBlock,
    content: *mut c_void,
    error: *mut c_void,
) {
    if !error.is_null() {
        // Log the error
        let desc = msg0(error, sel(b"localizedDescription\0"));
        if !desc.is_null() {
            let utf8: *const u8 = std::mem::transmute(msg0(desc, sel(b"UTF8String\0")));
            if !utf8.is_null() {
                let s = std::ffi::CStr::from_ptr(utf8 as *const _);
                eprintln!("[CLX] system_audio: SCShareableContent error: {:?}", s);
            }
        }
    }
    if !content.is_null() {
        // Retain so it survives past the completion handler
        objc_retain(content);
        CONTENT_RESULT.store(content, Ordering::Release);
    }
    let sem = CONTENT_SEM.load(Ordering::Acquire);
    if !sem.is_null() {
        dispatch_semaphore_signal(sem);
    }
}

// SAFETY: _NSConcreteGlobalBlock is a stable ABI symbol provided by libSystem.
// These blocks are global (no captures) and live for the entire program lifetime.
static CONTENT_COMPLETION_BLOCK: GlobalBlock = unsafe {
    GlobalBlock {
        isa: &_NSConcreteGlobalBlock as *const *const c_void,
        flags: 1 << 28,
        reserved: 0,
        invoke: shareable_content_completion as *const c_void,
        descriptor: &BLOCK_DESCRIPTOR,
    }
};

// ── SCStream start capture completion ───────────────────────────────────────

static START_SEM: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static START_ERROR: AtomicBool = AtomicBool::new(false);

/// Completion handler for `startCaptureWithCompletionHandler:`
/// Signature: ^(NSError * _Nullable error)
unsafe extern "C" fn start_capture_completion(
    _block: *mut GlobalBlock,
    error: *mut c_void,
) {
    if !error.is_null() {
        START_ERROR.store(true, Ordering::Release);
        let desc = msg0(error, sel(b"localizedDescription\0"));
        if !desc.is_null() {
            let utf8: *const u8 = std::mem::transmute(msg0(desc, sel(b"UTF8String\0")));
            if !utf8.is_null() {
                let s = std::ffi::CStr::from_ptr(utf8 as *const _);
                eprintln!("[CLX] system_audio: startCapture error: {:?}", s);
            }
        }
    } else {
        START_ERROR.store(false, Ordering::Release);
    }
    let sem = START_SEM.load(Ordering::Acquire);
    if !sem.is_null() {
        dispatch_semaphore_signal(sem);
    }
}

static START_COMPLETION_BLOCK: GlobalBlock = unsafe {
    GlobalBlock {
        isa: &_NSConcreteGlobalBlock as *const *const c_void,
        flags: 1 << 28,
        reserved: 0,
        invoke: start_capture_completion as *const c_void,
        descriptor: &BLOCK_DESCRIPTOR,
    }
};

// ── Shared audio buffer ──────────────────────────────────────────────────────

static SYS_AUDIO_BUF: once_cell::sync::Lazy<Arc<Mutex<Vec<f32>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(Vec::new())));
static SYS_AUDIO_ACTIVE: AtomicBool = AtomicBool::new(false);

// ── SCStreamOutput handler ───────────────────────────────────────────────────

/// `stream:didOutputSampleBuffer:ofType:` callback
extern "C" fn did_output_sample_buffer(
    _self: *mut c_void,
    _cmd: *mut c_void,
    _stream: *mut c_void,
    sample_buffer: *mut c_void,
    output_type: i64,
) {
    if output_type != 1 { return; } // 1 = SCStreamOutputTypeAudio
    if !SYS_AUDIO_ACTIVE.load(Ordering::Relaxed) { return; }

    unsafe {
        let block_buf = CMSampleBufferGetDataBuffer(sample_buffer);
        if block_buf.is_null() { return; }

        let mut data_ptr: *mut u8 = std::ptr::null_mut();
        let mut total_len: usize = 0;
        let status = CMBlockBufferGetDataPointer(
            block_buf, 0, std::ptr::null_mut(), &mut total_len, &mut data_ptr,
        );
        if status != 0 || data_ptr.is_null() || total_len == 0 { return; }

        // Audio arrives as interleaved f32 PCM (mono or stereo depending on config).
        let f32_count = total_len / 4;
        let samples = std::slice::from_raw_parts(data_ptr as *const f32, f32_count);

        let mut buf = SYS_AUDIO_BUF.lock().unwrap();
        buf.extend_from_slice(samples);
        // Cap buffer at 5 seconds of audio (48kHz * 5 = 240000)
        if buf.len() > 240_000 {
            let excess = buf.len() - 240_000;
            buf.drain(..excess);
        }
    }
}

static mut OUTPUT_HANDLER_CLS_REGISTERED: bool = false;

fn register_output_handler_class() {
    unsafe {
        if OUTPUT_HANDLER_CLS_REGISTERED { return; }
        let nsobject = cls(b"NSObject\0");
        let new_cls = objc_allocateClassPair(nsobject, b"CLXAudioOutputHandler\0".as_ptr() as *const _, 0);
        if new_cls.is_null() { OUTPUT_HANDLER_CLS_REGISTERED = true; return; }

        let proto = objc_getProtocol(b"SCStreamOutput\0".as_ptr() as *const _);
        if !proto.is_null() {
            class_addProtocol(new_cls, proto);
        }

        // Method: stream:didOutputSampleBuffer:ofType:
        let types = b"v@:@@q\0"; // void, self, SEL, SCStream*, CMSampleBufferRef, NSInteger
        class_addMethod(
            new_cls,
            sel(b"stream:didOutputSampleBuffer:ofType:\0"),
            did_output_sample_buffer as *const c_void,
            types.as_ptr() as *const _,
        );

        objc_registerClassPair(new_cls);
        OUTPUT_HANDLER_CLS_REGISTERED = true;
    }
}

// ── SystemAudioCapture ───────────────────────────────────────────────────────

pub struct SystemAudioCapture {
    stream: *mut c_void, // SCStream
    handler: *mut c_void, // CLXAudioOutputHandler instance
}

// SCStream is accessed from main thread, the handler callback runs on a dispatch queue
unsafe impl Send for SystemAudioCapture {}

impl SystemAudioCapture {
    pub fn new() -> Result<Self, String> {
        register_output_handler_class();

        unsafe {
            // Check if ScreenCaptureKit is available
            let sc_config_cls = cls(b"SCStreamConfiguration\0");
            if sc_config_cls.is_null() {
                return Err("ScreenCaptureKit not available (requires macOS 12.3+)".into());
            }

            eprintln!("[CLX] system_audio: ScreenCaptureKit setup starting...");

            // ── Step 1: Get SCShareableContent asynchronously ────────────────

            let sem = dispatch_semaphore_create(0);
            CONTENT_SEM.store(sem, Ordering::Release);
            CONTENT_RESULT.store(null_mut(), Ordering::Release);

            let sc_content_cls = cls(b"SCShareableContent\0");
            if sc_content_cls.is_null() {
                return Err("SCShareableContent class not found".into());
            }

            // [SCShareableContent getShareableContentExcludingDesktopWindows:YES
            //                                          onScreenWindowsOnly:NO
            //                                            completionHandler:block]
            let sel_get = sel(b"getShareableContentExcludingDesktopWindows:onScreenWindowsOnly:completionHandler:\0");
            let f_get: extern "C" fn(*mut c_void, *mut c_void, bool, bool, *const GlobalBlock) =
                std::mem::transmute(objc_msgSend as *const ());
            f_get(sc_content_cls, sel_get, true, false, &CONTENT_COMPLETION_BLOCK);

            // Block until the completion handler fires (with a 10s timeout)
            let timeout_ns: u64 = 10_000_000_000; // 10 seconds
            extern "C" { fn dispatch_time(when: u64, delta: i64) -> u64; }
            let deadline = dispatch_time(0, timeout_ns as i64);
            let wait_result = dispatch_semaphore_wait(sem, deadline);

            if wait_result != 0 {
                return Err("Timed out waiting for SCShareableContent".into());
            }

            let content = CONTENT_RESULT.load(Ordering::Acquire);
            if content.is_null() {
                return Err("SCShareableContent returned nil (permission denied?)".into());
            }

            // ── Step 2: Get the first display ────────────────────────────────

            let displays = msg0(content, sel(b"displays\0"));
            if displays.is_null() {
                return Err("SCShareableContent.displays is nil".into());
            }
            let count = CFArrayGetCount(displays as *mut _);
            if count <= 0 {
                return Err("No displays found in SCShareableContent".into());
            }
            let first_display = CFArrayGetValueAtIndex(displays as *mut _, 0) as *mut c_void;
            if first_display.is_null() {
                return Err("First display is nil".into());
            }

            eprintln!("[CLX] system_audio: found {} display(s)", count);

            // ── Step 3: Create SCContentFilter ───────────────────────────────

            // Create empty NSArray for excludingApplications and exceptingWindows
            let nsarray_cls = cls(b"NSArray\0");
            let empty_array = msg0(nsarray_cls, sel(b"array\0"));

            // [[SCContentFilter alloc] initWithDisplay:excludingApplications:exceptingWindows:]
            let filter_cls = cls(b"SCContentFilter\0");
            let filter_alloc = msg0(filter_cls, sel(b"alloc\0"));
            let sel_init_filter = sel(b"initWithDisplay:excludingApplications:exceptingWindows:\0");
            let f_init_filter: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            let filter = f_init_filter(filter_alloc, sel_init_filter, first_display, empty_array, empty_array);
            if filter.is_null() {
                return Err("Failed to create SCContentFilter".into());
            }

            // ── Step 4: Create SCStreamConfiguration ─────────────────────────

            let config = msg0(msg0(sc_config_cls, sel(b"alloc\0")), sel(b"init\0"));

            // config.capturesAudio = YES
            let f_set_bool: extern "C" fn(*mut c_void, *mut c_void, bool) =
                std::mem::transmute(objc_msgSend as *const ());
            f_set_bool(config, sel(b"setCapturesAudio:\0"), true);

            // config.excludesCurrentProcessAudio = YES
            f_set_bool(config, sel(b"setExcludesCurrentProcessAudio:\0"), true);

            // config.channelCount = 1 (mono)
            let f_set_i64: extern "C" fn(*mut c_void, *mut c_void, i64) =
                std::mem::transmute(objc_msgSend as *const ());
            f_set_i64(config, sel(b"setChannelCount:\0"), 1);

            // config.sampleRate = 48000
            f_set_i64(config, sel(b"setSampleRate:\0"), 48000);

            // Don't capture video — set minimal size to avoid wasting resources
            f_set_i64(config, sel(b"setWidth:\0"), 2);
            f_set_i64(config, sel(b"setHeight:\0"), 2);

            // config.minimumFrameInterval = very long (we don't want video frames)
            // CMTime { value=1, timescale=1 } = 1 frame per second (minimum)
            // Actually just skip video output entirely — we only add audio output below.

            // ── Step 5: Create SCStream ──────────────────────────────────────

            let stream_cls = cls(b"SCStream\0");
            let stream_alloc = msg0(stream_cls, sel(b"alloc\0"));
            let sel_init_stream = sel(b"initWithFilter:configuration:delegate:\0");
            let f_init_stream: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            let stream = f_init_stream(stream_alloc, sel_init_stream, filter, config, null_mut());
            if stream.is_null() {
                return Err("Failed to create SCStream".into());
            }

            // ── Step 6: Create handler and add as stream output ──────────────

            let handler_cls = cls(b"CLXAudioOutputHandler\0");
            let handler = msg0(msg0(handler_cls, sel(b"alloc\0")), sel(b"init\0"));
            if handler.is_null() {
                return Err("Failed to create CLXAudioOutputHandler".into());
            }

            // [stream addStreamOutput:handler type:SCStreamOutputTypeAudio
            //        sampleHandlerQueue:nil error:&error]
            // SCStreamOutputTypeAudio = 1
            let sel_add_output = sel(b"addStreamOutput:type:sampleHandlerQueue:error:\0");
            let mut error: *mut c_void = null_mut();
            let f_add_output: extern "C" fn(
                *mut c_void, *mut c_void,
                *mut c_void, i64, *mut c_void, *mut *mut c_void,
            ) -> bool = std::mem::transmute(objc_msgSend as *const ());
            let ok = f_add_output(stream, sel_add_output, handler, 1, null_mut(), &mut error);
            if !ok || !error.is_null() {
                let err_msg = if !error.is_null() {
                    let desc = msg0(error, sel(b"localizedDescription\0"));
                    if !desc.is_null() {
                        let utf8: *const u8 = std::mem::transmute(msg0(desc, sel(b"UTF8String\0")));
                        if !utf8.is_null() {
                            let s = std::ffi::CStr::from_ptr(utf8 as *const _);
                            format!("{:?}", s)
                        } else {
                            "unknown".into()
                        }
                    } else {
                        "unknown".into()
                    }
                } else {
                    "addStreamOutput returned false".into()
                };
                return Err(format!("Failed to add stream output: {}", err_msg));
            }

            // ── Step 7: Start capture ────────────────────────────────────────

            let start_sem = dispatch_semaphore_create(0);
            START_SEM.store(start_sem, Ordering::Release);
            START_ERROR.store(false, Ordering::Release);

            // [stream startCaptureWithCompletionHandler:block]
            let sel_start = sel(b"startCaptureWithCompletionHandler:\0");
            let f_start: extern "C" fn(*mut c_void, *mut c_void, *const GlobalBlock) =
                std::mem::transmute(objc_msgSend as *const ());
            f_start(stream, sel_start, &START_COMPLETION_BLOCK);

            let deadline = dispatch_time(0, 10_000_000_000); // 10s
            let wait_result = dispatch_semaphore_wait(start_sem, deadline);

            if wait_result != 0 {
                return Err("Timed out waiting for startCapture".into());
            }
            if START_ERROR.load(Ordering::Acquire) {
                return Err("startCapture failed (check stderr for details)".into());
            }

            SYS_AUDIO_ACTIVE.store(true, Ordering::Relaxed);

            eprintln!("[CLX] system_audio: ScreenCaptureKit stream started (48kHz mono)");

            Ok(Self {
                stream,
                handler,
            })
        }
    }
}

impl SystemAudioStream for SystemAudioCapture {
    fn take_samples(&self) -> Vec<f32> {
        let mut buf = SYS_AUDIO_BUF.lock().unwrap();
        std::mem::take(&mut *buf)
    }

    fn stop(&self) {
        SYS_AUDIO_ACTIVE.store(false, Ordering::Relaxed);
        if !self.stream.is_null() {
            unsafe {
                // [stream stopCaptureWithCompletionHandler:nil]
                let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
                    std::mem::transmute(objc_msgSend as *const ());
                f(self.stream, sel(b"stopCaptureWithCompletionHandler:\0"), std::ptr::null_mut());
            }
        }
        eprintln!("[CLX] system_audio: stopped");
    }

    fn sample_rate(&self) -> u32 { 48000 }
}
