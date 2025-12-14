#![allow(deprecated)]

#[cfg(target_os = "macos")]
use iced::wgpu::rwh::WindowHandle;

#[cfg(target_os = "macos")]
pub fn set_activation_policy_accessory() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApp, NSApplicationActivationPolicy};

    let mtm = MainThreadMarker::new().expect("must be on main thread");
    let app = NSApp(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
}

#[cfg(target_os = "macos")]
pub fn macos_window_config(handle: &WindowHandle) {
    use iced::wgpu::rwh::RawWindowHandle;
    use objc2::rc::Retained;
    use objc2_app_kit::NSView;

    match handle.as_raw() {
        RawWindowHandle::AppKit(handle) => {
            let ns_view = handle.ns_view.as_ptr();
            let ns_view: Retained<NSView> = unsafe { Retained::retain(ns_view.cast()) }.unwrap();
            let ns_window = ns_view
                .window()
                .expect("view was not installed in a window");

            use objc2_app_kit::{NSFloatingWindowLevel, NSWindowCollectionBehavior};
            ns_window.setLevel(NSFloatingWindowLevel);

            ns_window.setCollectionBehavior(NSWindowCollectionBehavior::CanJoinAllSpaces);
        }
        _ => {
            panic!(
                "Why are you running this as a non-appkit window? this is a macos only app as of now"
            );
        }
    }
}

#[cfg(target_os = "macos")]
pub fn focus_this_app() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApp;

    let mtm = MainThreadMarker::new().expect("must be on main thread");
    let app = NSApp(mtm);

    app.activateIgnoringOtherApps(true);
}

// because objc2_application_services is mean and this type isn't public
#[repr(C)]
struct ProcessSerialNumber {
    low: u32,
    hi: u32,
}

/// see mostly https://github.com/electron/electron/blob/e181fd040f72becd135db1fa977622b81da21643/shell/browser/browser_mac.mm#L512C1-L532C2.
///
/// returns ApplicationServices OSStatus (u32)
///
/// doesn't seem to do anything if you haven't opened a window yet, so wait to call it until after that.
pub fn transform_process_to_ui_element() -> u32 {
    use std::ptr;

    use objc2_application_services::{
        TransformProcessType, kCurrentProcess, kProcessTransformToUIElementApplication,
    };
    let psn = ProcessSerialNumber {
        low: 0,
        hi: kCurrentProcess,
    };
    unsafe {
        TransformProcessType(
            ptr::from_ref(&psn).cast(),
            kProcessTransformToUIElementApplication,
        )
    }
}
