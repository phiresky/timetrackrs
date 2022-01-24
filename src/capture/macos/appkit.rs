use super::types::*;
use crate::prelude::*;

use objc::{
    class, msg_send,
    runtime::Object,
    sel, sel_impl,
};
use std::ffi::CStr;
use sysinfo::{
    Pid, PidExt, ProcessExt, System, SystemExt,
};

pub struct MacOSCapturer {
    os_info: util::OsInfo
}

impl MacOSCapturer {
    pub fn init() -> MacOSCapturer {
        MacOSCapturer{
            os_info: util::get_os_info()
        }
    }
}

impl Capturer for MacOSCapturer {
    fn capture(&mut self) -> anyhow::Result<EventData> {
       Ok(EventData::macos_v1(MacOSEventData {
            os_info: self.os_info.clone(),
            focused_window: get_frontmost_app_pid(),
            windows: get_running_apps(),
            duration_since_user_input: user_idle::UserIdle::get_time()
                .map(|e| e.duration())
                .map_err(|e| anyhow::Error::msg(e))
                .context("Couldn't get duration since user input")
                .unwrap_or_else(|e| {
                    log::warn!("{}", e);
                    Duration::ZERO
                })
       }))
    }
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

pub fn get_frontmost_app_pid() -> Option<i32> {
    unsafe { 
        let shared_workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
        
        let frontmost_application: *mut Object = msg_send![shared_workspace, frontmostApplication];

        if frontmost_application.is_null() {
            return None;
        }

        let pid: i32 = msg_send![frontmost_application, processIdentifier];

        if pid == -1 {
            return None;
        }

        Some(pid)
    }
}

/// Gets all currently running apps that may have UIs and are visible in the dock.
/// Reference: https://developer.apple.com/documentation/appkit/nsapplicationactivationpolicy?language=objc
pub fn get_running_apps() -> Vec<MacOSApp> {
    let mut vec = vec![];

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
                None => continue
            };
            
            // If the process wasn't launched by launchd
            // it means that it's not the parent process
            // for example it might be some chrome rendering
            // process, but not the main chrome process
            if parent != 1 {
                continue;
            }

            let macos_app = MacOSApp {
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
                parent: Some(parent)
            };

            vec.push(macos_app);
        }
        release(running_applications);
        release(shared_workspace);
    }
    vec
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
    release(ns_string);
    
    bundle_url
}
