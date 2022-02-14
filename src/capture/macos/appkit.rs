use super::types::*;
use crate::prelude::*;

use accessibility_sys::{
    kAXErrorSuccess, kAXFocusedWindowAttribute, kAXTitleAttribute, kAXTrustedCheckOptionPrompt,
    kAXWindowsAttribute, AXError, AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue,
    AXUIElementCreateApplication, AXUIElementRef,
};
use core_foundation::{
    array::{CFArray, CFArrayRef},
    base::{CFRelease, FromVoid, ItemRef, TCFType, ToVoid},
    boolean::CFBoolean,
    dictionary::{CFDictionary, CFMutableDictionary},
    number::CFNumber,
    string::{kCFStringEncodingUTF8, CFString, CFStringGetCStringPtr, CFStringRef},
};
use core_graphics::window::{
    kCGNullWindowID, kCGWindowListOptionAll, kCGWindowListOptionOnScreenOnly, CGWindowID,
    CGWindowListCopyWindowInfo,
};
use objc::{class, msg_send, runtime::Object, sel, sel_impl};
use rustc_hash::FxHashMap;
use std::{
    ffi::{c_void, CStr},
    sync::Arc,
};
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};

pub struct MacOSCapturer {
    os_info: util::OsInfo,
    // Using Arc<String>, so that
    // we don't clone the string when passing it to the MacOSWindow
    // struct.
    window_titles: FxHashMap<i32, Arc<String>>,
    accessibility_permission: bool,
}

impl MacOSCapturer {
    pub fn init() -> MacOSCapturer {
        let accessibility_permission = unsafe { check_accessibility_permission() };
        MacOSCapturer {
            os_info: util::get_os_info(),
            window_titles: FxHashMap::default(),
            accessibility_permission,
        }
    }

    /// Gets all currently running apps that may have UIs and are visible in the dock.
    /// Reference: https://developer.apple.com/documentation/appkit/nsapplicationactivationpolicy?language=objc
    pub fn get_windows(&mut self) -> Vec<MacOSWindow> {
        let MacOSCapturer {
            accessibility_permission,
            ..
        } = *self;

        let mut windows: Vec<MacOSWindow> = vec![];

        let mut system = System::new();

        unsafe {
            let cf_array: ItemRef<CFArray<CFDictionary<CFStringRef, *const c_void>>> =
                CFArray::from_void(CGWindowListCopyWindowInfo(
                    kCGWindowListOptionOnScreenOnly,
                    kCGNullWindowID,
                ) as *const _);

            for window in cf_array.iter() {
                let (keys, values) = window.get_keys_and_values();

                let mut macos_window = MacOSWindow::default();

                let mut pid: Option<i32> = None;

                for i in 0..keys.len() {
                    let key = CFStringGetCStringPtr(keys[i] as _, kCFStringEncodingUTF8);

                    let key = CStr::from_ptr(key).to_str().unwrap();

                    match key {
                        "kCGWindowNumber" => {
                            macos_window.window_id =
                                CFNumber::from_void(values[i]).to_i32().unwrap();
                        }
                        "kCGWindowOwnerPID" => {
                            pid = CFNumber::from_void(values[i]).to_i32();
                        }
                        _ => (),
                    };
                }

                if let Some(pid) = pid {
                    let sysinfo_pid = Pid::from(pid);
                    system.refresh_process(sysinfo_pid);
                    if let Some(process) = system.process(sysinfo_pid) {
                        macos_window.process = process.into(); 
                        if accessibility_permission {
                            let app_ref = AXUIElementCreateApplication(pid);

                            let mut ax_element_ref: *const c_void = std::ptr::null();

                            if AXUIElementCopyAttributeValue(
                                app_ref,
                                CFString::from_static_string(kAXFocusedWindowAttribute)
                                    .as_concrete_TypeRef(),
                                &mut ax_element_ref,
                            ) == kAXErrorSuccess
                            {
                                let mut cf_string: *const c_void = std::ptr::null();

                                if AXUIElementCopyAttributeValue(
                                    ax_element_ref as *mut _,
                                    CFString::from_static_string(kAXTitleAttribute)
                                        .as_concrete_TypeRef(),
                                    &mut cf_string,
                                ) == kAXErrorSuccess
                                {
                                    let string = CFString::from_void(cf_string).to_string();

                                    macos_window.title = Some(string);

                                    CFRelease(cf_string);
                                }
                                CFRelease(ax_element_ref);
                            }
                            CFRelease(app_ref as *const _);
                        }
                        windows.push(macos_window);
                    }
                }
            }
        }
        windows
    }
}

impl Capturer for MacOSCapturer {
    fn capture(&mut self) -> anyhow::Result<EventData> {
        let windows = self.get_windows();
        
        Ok(EventData::macos_v1(MacOSEventData {
            os_info: self.os_info.clone(),
            windows,
            duration_since_user_input: user_idle::UserIdle::get_time()
                .map(|e| e.duration())
                .map_err(|e| anyhow::Error::msg(e))
                .context("Couldn't get duration since user input")
                .unwrap_or_else(|e| {
                    log::warn!("{}", e);
                    Duration::ZERO
                }),
        }))
    }
}

/// Checks the Accessibility permission, if not available prompts the user for it.
unsafe fn check_accessibility_permission() -> bool {
    let mut dict: CFMutableDictionary<CFString, CFNumber> = CFMutableDictionary::new();

    dict.add(
        &CFString::from_void(kAXTrustedCheckOptionPrompt as *const c_void).to_owned(),
        &1i64.into(),
    );

    let app_has_permissions =
        AXIsProcessTrustedWithOptions(dict.into_untyped().to_void() as *const _);

    app_has_permissions
}

/// Frees any Objects
unsafe fn release(object: *mut Object) {
    let _: () = msg_send![object, release];
}

/// Turns an
/// [NSString](https://developer.apple.com/documentation/foundation/nsstring?language=objc) Object into a `&str`.
unsafe fn ns_string_to_string(ns_string: *mut Object) -> Option<String> {
    // Get length of name
    let string_size: usize = msg_send![ns_string, lengthOfBytesUsingEncoding: 4];

    // Allocate length of name + 1 (for null terminator)
    let char_ptr = libc::malloc(string_size + 1);

    // Copy the string into the allocated memory
    // encoding: 4 is for specifying that the string has UTF-8 encoding
    let res: bool = msg_send![ns_string, getCString:char_ptr maxLength:string_size + 1 encoding:4];

    release(ns_string);

    if !res {
        libc::free(char_ptr);
        return None;
    }

    let string = CStr::from_ptr(char_ptr as *const i8)
        .to_str()
        .unwrap()
        .to_owned();

    libc::free(char_ptr);

    Some(string)
}
