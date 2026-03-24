// objc_try.m — ObjC exception catcher for Rust FFI.
// Compile: clang -c objc_try.m -o objc_try.o -fobjc-arc
#import <Foundation/Foundation.h>

// Calls `fn_ptr(context)` inside @try/@catch.
// Returns 0 on success, 1 on ObjC exception.
int objc_try_catch(void (*fn_ptr)(void*), void* context) {
    @try {
        fn_ptr(context);
        return 0;
    } @catch (NSException *exception) {
        NSLog(@"[CLX] ObjC exception caught: %@ — %@", exception.name, exception.reason);
        return 1;
    }
}
