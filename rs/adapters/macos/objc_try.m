// objc_try.m — Exception catcher for Rust FFI.
// Catches both ObjC exceptions AND C++ exceptions (Rust panics).
#import <Foundation/Foundation.h>

// Calls `fn_ptr(context)` inside @try/@catch.
// Returns 0 on success, 1 on ObjC exception, 2 on other exception.
int objc_try_catch(void (*fn_ptr)(void*), void* context) {
    @try {
        fn_ptr(context);
        return 0;
    } @catch (NSException *exception) {
        NSLog(@"[CLX] ObjC exception caught: %@ — %@", exception.name, exception.reason);
        return 1;
    } @catch (id other) {
        // Catches C++ exceptions (including Rust panics that unwind as C++).
        NSLog(@"[CLX] Foreign exception caught (C++/Rust panic)");
        return 2;
    }
}
