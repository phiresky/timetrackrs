use super::types::*;
use crate::prelude::*;

use core_foundation::{
    array::CFArray,
    base::{Boolean, FromVoid, ItemRef},
    dictionary::CFDictionary,
    number::CFNumber,
    string::{kCFStringEncodingUTF8, CFString, CFStringGetCStringPtr},
};
use core_graphics::window::{
    kCGNullWindowID, kCGWindowLayer, kCGWindowListOptionExcludeDesktopElements,
    kCGWindowListOptionOnScreenOnly, CGWindowListCopyWindowInfo,
};
use objc::{class, msg_send, runtime::Object, sel, sel_impl};
use rustc_hash::FxHashMap;
use std::{
    ffi::{c_void, CStr},
    sync::Arc,
};
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    pub fn CGRequestScreenCaptureAccess() -> Boolean;
}

pub struct MacOSCapturer {
    os_info: util::OsInfo,
}

impl MacOSCapturer {
    pub fn init() -> MacOSCapturer {
        unsafe {
            CGRequestScreenCaptureAccess();
        }
        MacOSCapturer {
            os_info: util::get_os_info(),
        }
    }
}

impl Capturer for MacOSCapturer {
    fn capture(&mut self) -> anyhow::Result<EventData> {
        let (focused_window, windows) = get_windows();

        Ok(EventData::macos_v1(MacOSEventData {
            os_info: self.os_info.clone(),
            focused_window,
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

/// Frees any Objects
unsafe fn release(object: *mut Object) {
    msg_send![object, release]
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

/// Gets all currently running apps that may have UIs and are visible in the dock.
/// Reference: https://developer.apple.com/documentation/appkit/nsapplicationactivationpolicy?language=objc
pub fn get_windows() -> (Option<i32>, Vec<MacOSWindow>) {
    let mut focused_window_id: Option<i32> = None;

    let mut windows: Vec<MacOSWindow> = vec![];

    let mut process_data_map: FxHashMap<i32, Arc<MacOSProcessData>> = FxHashMap::default();

    let mut system = System::new();

    unsafe {
        let shared_workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];

        let running_applications: *mut Object = msg_send![shared_workspace, runningApplications];

        let count: usize = msg_send![running_applications, count];

        for i in 0..count {
            let ns_running_application: *mut Object =
                msg_send![running_applications, objectAtIndex: i];

            if ns_running_application.is_null() {
                continue;
            }

            let pid: i32 = msg_send![ns_running_application, processIdentifier];

            if pid == -1 {
                continue;
            }

            let activation_policy: isize = msg_send![ns_running_application, activationPolicy];

            // Reference: https://developer.apple.com/documentation/appkit/nsapplicationactivationpolicy/
            if activation_policy != 0 {
                continue;
            }

            let pid = Pid::from_u32(pid as u32);

            system.refresh_process(pid);

            let procinfo = match system.process(pid) {
                Some(procinfo) => procinfo,
                None => continue,
            };

            let parent = match procinfo.parent() {
                Some(parent) => parent.as_u32() as i32,
                None => continue,
            };

            // If the process wasn't launched by launchd
            // it means that it's not the parent process
            // for example it might be some chrome rendering
            // process, but not the main chrome process
            if parent != 1 {
                continue;
            }

            let process_data = MacOSProcessData {
                pid: procinfo.pid().as_u32() as i32,
                name: procinfo.name().to_string(),
                cmd: procinfo.cmd().to_vec(),
                exe: procinfo.exe().to_string_lossy().to_string(), // tbh i don't care if your executables have filenames that are not unicode
                cwd: procinfo.cwd().to_string_lossy().to_string(),
                memory_kB: procinfo.memory() as i64,
                status: procinfo.status().to_string().to_string(),
                start_time: util::unix_epoch_millis_to_date((procinfo.start_time() as i64) * 1000),
                cpu_usage: Some(procinfo.cpu_usage()),
                bundle: get_bundle_url(ns_running_application),
                parent: Some(parent),
            };

            process_data_map.insert(process_data.pid, Arc::new(process_data));
        }

        release(running_applications);

        let frontmost_application: *mut Object = msg_send![shared_workspace, frontmostApplication];

        let frontmost_application_pid: i32 = msg_send![frontmost_application, processIdentifier];

        let cf_array: ItemRef<CFArray<CFDictionary<CFString, *const c_void>>> =
            CFArray::from_void(CGWindowListCopyWindowInfo(
                kCGWindowListOptionExcludeDesktopElements | kCGWindowListOptionOnScreenOnly,
                kCGNullWindowID,
            ) as *const _);

        for window in cf_array.iter() {
            if CFNumber::from_void(*window.get(kCGWindowLayer))
                .to_i32()
                .unwrap_unchecked()
                != 0
            {
                continue;
            }
            let (keys, values) = window.get_keys_and_values();

            let mut macos_window = MacOSWindow::default();

            let mut pid: Option<i32> = None;

            for i in 0..keys.len() {
                let key = CFStringGetCStringPtr(keys[i] as _, kCFStringEncodingUTF8);

                let key = CStr::from_ptr(key).to_str().unwrap();

                match key {
                    "kCGWindowName" => {
                        macos_window.title = Some(CFString::from_void(values[i]).to_string());
                    }
                    "kCGWindowNumber" => {
                        macos_window.window_id = CFNumber::from_void(values[i]).to_i32().unwrap();
                    }
                    "kCGWindowOwnerPID" => {
                        pid = CFNumber::from_void(values[i]).to_i32();
                    }
                    _ => (),
                };
            }

            if let Some(pid) = pid {
                if let Some(process) = process_data_map.get(&pid) {
                    macos_window.process = Some(process.clone());
                    if process.pid == frontmost_application_pid {
                        focused_window_id = Some(macos_window.window_id);
                    }
                }
            }

            if macos_window.title.is_none() && macos_window.process.is_none() {
                continue;
            }

            windows.push(macos_window);
        }
    }
    (focused_window_id, windows)
}

/// Gets **bundleURL** of [NSRunningApplication](https://developer.apple.com/documentation/appkit/nsrunningapplication?language=objc).
unsafe fn get_bundle_url(object: *mut Object) -> Option<String> {
    let ns_url: *mut Object = msg_send![object, bundleURL];

    if ns_url.is_null() {
        return None;
    }

    let ns_string: *mut Object = msg_send![ns_url, absoluteString];

    if ns_string.is_null() {
        return None;
    }

    let bundle_url = ns_string_to_string(ns_string);

    release(ns_url);

    bundle_url
}
