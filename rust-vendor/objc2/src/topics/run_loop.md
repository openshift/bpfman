# Run loops and applications

At the core of all Cocoa development sits what is known as the "run loop". This is Apple's a mechanism for allowing scheduling different tasks on the same thread, a bit like a Rust `async` runtime. See [their introductory documentation][runloop-doc] for more details.

A lot of things in various different frameworks assume that the main thread's run loop is currently running, and will block indefinitely or fail in confusing ways if it is not. To avoid this, you should make sure to, well, run the run loop.

[runloop-doc]: https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Multithreading/RunLoopManagement/RunLoopManagement.html


## Non-graphical applications

In non-graphical applications, you get the thread's current `NSRunLoop`, and run it periodically to allow scheduled work to complete.

```rust, no_run
use objc2_foundation::{NSDate, NSDefaultRunLoopMode, NSRunLoop};

fn main() {
    let run_loop = unsafe { NSRunLoop::currentRunLoop() };

    // Set up timers, sources, etc.

    let mut date = unsafe { NSDate::now() };
    // Run for roughly 10 seconds
    for i in 0..10 {
        date = unsafe { date.dateByAddingTimeInterval(1.0) };
        unsafe { run_loop.runUntilDate(&date) };

        // Do something every second (if there are any sources attached)
    }
}
```


## Graphical applications

In graphical applications, the main run loop needs to be managed by the application object. To get feedback during the execution of the application, you usually use a delegate instead, as can be seen in the following example.

```rust, no_run
use objc2::rc::{Allocated, Retained};
use objc2::{define_class, msg_send, ClassType, DefinedClass, MainThreadOnly};
use objc2_foundation::{NSNotification, NSObject, NSObjectProtocol};

// Application delegate protocols happens to share a few methods,
// we can utilize that to be a bit more platform-generic.
#[cfg(target_os = "macos")]
use objc2_app_kit::NSApplicationDelegate as DelegateProtocol;
#[cfg(not(target_os = "macos"))]
use objc2_ui_kit::UIApplicationDelegate as DelegateProtocol;

#[derive(Default)]
struct AppState {
    // Whatever state you want to store in your delegate.
}

define_class!(
    // SAFETY:
    // - NSObject does not have any subclassing requirements.
    // - `AppDelegate` does not implement `Drop`.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "AppDelegate"]
    #[ivars = AppState]
    struct AppDelegate;

    impl AppDelegate {
        // Called by `NSApplicationMain`, `UIApplicationMain`
        // or our `msg_send![AppDelegate::class(), new]`.
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(AppState::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl DelegateProtocol for AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, _notification: &NSNotification) {
            println!("did finish launching!");

            // Do UI initialization in here, such as creating windows, views, etc.
        }

        #[unsafe(method(applicationWillTerminate:))]
        fn will_terminate(&self, _notification: &NSNotification) {
            println!("will terminate!");

            // Tear down your application state here. `NSApplicationMain` and
            // `UIApplicationMain` will not return, this is (roughly) the last
            // thing that will be called.
        }
    }
);

// AppKit (macOS).
#[cfg(target_os = "macos")]
fn main() {
    let mtm = objc2::MainThreadMarker::new().unwrap();
    let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
    let delegate: Retained<AppDelegate> = unsafe { msg_send![AppDelegate::class(), new] };
    app.setDelegate(Some(objc2::runtime::ProtocolObject::from_ref(&*delegate)));
    app.run();
}

// AppKit (macOS), if bundled and using a storyboard.
#[cfg(target_os = "macos")]
# #[cfg(with_storyboard)] // Hack to make example compile.
fn main() {
    // Initialize the class so that the storyboard can see it.
    //
    // The name specified in `define_class!`, i.e. "AppDelegate", must
    // match what's specified in the storyboard.
    let _cls = AppDelegate::class();
    unsafe {
        objc2_app_kit::NSApplicationMain(
            *libc::_NSGetArgc(),
            std::ptr::NonNull::new(*libc::_NSGetArgv()).unwrap().cast(),
        );
    }
}

// UIKit (iOS/tvOS/watchOS/visionOS).
#[cfg(not(target_os = "macos"))]
fn main() {
    let delegate_class = unsafe { objc2_foundation::NSStringFromClass(AppDelegate::class()) };
    unsafe {
        objc2_ui_kit::UIApplicationMain(
            *libc::_NSGetArgc(),
            std::ptr::NonNull::new(*libc::_NSGetArgv()).unwrap(),
            None,
            Some(&delegate_class),
        );
    }
}
```

See [the documentation in `objc2-app-kit`][appkit-docs] and [in `objc2-ui-kit`][uikit-docs] for more examples. Note in particular that in UIKit, you may want to use scenes as well.

[appkit-docs]: https://docs.rs/objc2-app-kit/
[uikit-docs]: https://docs.rs/objc2-ui-kit/


## Performance sensitive applications

In some performance-sensitive cases, it can make sense to drop into the lower-level details, and directly use `dispatch_main`, `CFRunLoop` or similar to run the run loop. Note that this may prevent things that depend on `NSRunLoop` features from working, so test thoroughly.
