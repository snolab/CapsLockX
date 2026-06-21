// Toggle / read / write the OS AlphaShift (CapsLock) state via IOKit.
//
// Implemented in ObjC rather than Rust FFI because a previous Rust binding
// returned kIOReturnBadArgument from IOHIDGetModifierLockState, likely due
// to a subtle ABI mismatch (the symbol `mach_task_self_` vs the macro
// `mach_task_self()`, or `bool` calling convention).  Linking directly
// against Apple's headers eliminates that class of bug.
//
// Returns 0 on success, non-zero IOReturn on failure.  `state` out-params
// are written only on success.

#import <Foundation/Foundation.h>
#import <IOKit/IOKitLib.h>
#import <IOKit/hidsystem/IOHIDLib.h>
#import <IOKit/hidsystem/IOHIDParameter.h>
#import <mach/mach.h>

static io_connect_t clx_hid_open(void) {
    static io_connect_t cached = 0;
    static dispatch_once_t once;
    dispatch_once(&once, ^{
        io_service_t service = IOServiceGetMatchingService(
            kIOMainPortDefault,
            IOServiceMatching(kIOHIDSystemClass));
        if (service == IO_OBJECT_NULL) {
            fprintf(stderr, "[CLX] clx_hid_open: no IOHIDSystem service\n");
            return;
        }
        kern_return_t kr = IOServiceOpen(service, mach_task_self(),
                                         kIOHIDParamConnectType, &cached);
        IOObjectRelease(service);
        if (kr != KERN_SUCCESS) {
            fprintf(stderr, "[CLX] clx_hid_open: IOServiceOpen kr=0x%08x\n", kr);
            cached = 0;
        }
    });
    return cached;
}

int32_t clx_caps_get(bool *out_state) {
    io_connect_t conn = clx_hid_open();
    if (conn == 0) return -1;
    bool state = false;
    IOReturn r = IOHIDGetModifierLockState(conn, kIOHIDCapsLockState, &state);
    if (r == kIOReturnSuccess && out_state) *out_state = state;
    return (int32_t)r;
}

int32_t clx_caps_set(bool state) {
    io_connect_t conn = clx_hid_open();
    if (conn == 0) return -1;
    IOReturn r = IOHIDSetModifierLockState(conn, kIOHIDCapsLockState, state);
    return (int32_t)r;
}

int32_t clx_caps_toggle(void) {
    io_connect_t conn = clx_hid_open();
    if (conn == 0) return -1;
    bool state = false;
    IOReturn r = IOHIDGetModifierLockState(conn, kIOHIDCapsLockState, &state);
    if (r != kIOReturnSuccess) {
        fprintf(stderr, "[CLX] clx_caps_toggle: get failed r=0x%08x\n", r);
        return (int32_t)r;
    }
    r = IOHIDSetModifierLockState(conn, kIOHIDCapsLockState, !state);
    if (r != kIOReturnSuccess) {
        fprintf(stderr, "[CLX] clx_caps_toggle: set failed r=0x%08x\n", r);
    }
    return (int32_t)r;
}
