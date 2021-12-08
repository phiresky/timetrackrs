use std::{collections::HashMap, ffi::OsString, slice};

use super::types::*;
use crate::prelude::*;
use regex::Regex;
use winapi::shared::windef::HWND;

pub struct WindowsCapturer {
    os_info: util::OsInfo,
}
impl WindowsCapturer {
    pub fn init() -> anyhow::Result<WindowsCapturer> {
        Ok(WindowsCapturer {
            os_info: util::get_os_info(),
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_Process")]
#[serde(rename_all = "PascalCase")]
struct WmiInfo {
    process_id: u32,
    command_line: String,
    creation_date: wmi::datetime::WMIDateTime,
}
// probably slow, but how else to do it?
fn ol(process_id: i64) -> anyhow::Result<Option<WmiInfo>> {
    let wmi_con = wmi::WMIConnection::new(wmi::COMLibrary::new()?.into())?;
    let mut filters = HashMap::new();
    filters.insert("ProcessId".to_owned(), wmi::FilterValue::Number(process_id));
    let res: Vec<WmiInfo> = wmi_con.filtered_query::<WmiInfo>(&filters)?;
    for proc in res {
        return Ok(Some(proc));
    }
    Ok(None)
}
impl Capturer for WindowsCapturer {
    fn capture(&mut self) -> anyhow::Result<EventData> {
        let focused_window = get_foreground_window().map(|f| get_window_id(f));
        Ok(EventData::windows_v1(WindowsEventData {
            os_info: self.os_info.clone(),
            focused_window,
            windows: get_all_windows(true),
            wifi: get_wifi_ssid()
                .context("could not get wifi")
                .unwrap_or_else(|e| {
                    log::warn!("{}", e);
                    return None;
                }),
            duration_since_user_input: user_idle::UserIdle::get_time()
                .map(|e| e.duration())
                .map_err(|e| anyhow::Error::msg(e))
                .context("could not get duration since user input")
                .unwrap_or_else(|e| {
                    log::warn!("{}", e);
                    return Duration::ZERO;
                }),
        }))
    }
}

fn get_wifi_ssid() -> anyhow::Result<Option<String>> {
    lazy_static! {
        static ref SSID_MATCH: Regex = Regex::new(r"(?m)^\s*SSID\s*:\s*(.*?)\r?$").unwrap();
    }
    use std::os::windows::process::CommandExt;
    let output = std::process::Command::new("netsh")
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
        .args(&["wlan", "show", "interfaces"])
        .output()
        .context("could not run netsh")?;
    let output = String::from_utf8_lossy(&output.stdout);
    let matched = SSID_MATCH
        .captures(&output)
        .map(|m| m.get(1).unwrap().as_str().to_string());
    return Ok(matched);
}
#[allow(dead_code)]
pub fn get_foreground_window() -> Option<HWND> {
    unsafe {
        use winapi::um::winuser::GetForegroundWindow;
        let hwnd = GetForegroundWindow(); // GetActiveWindow
        if hwnd == std::ptr::null_mut() {
            return None;
        }
        Some(hwnd)
    }
}

#[allow(dead_code)]
pub fn get_window_title(hwnd: HWND) -> String {
    unsafe {
        use winapi::um::winuser::{GetWindowTextLengthW, GetWindowTextW};
        let size = GetWindowTextLengthW(hwnd) + 1;
        let mut title: Vec<u16> = Vec::with_capacity(size as usize);
        let read_len = GetWindowTextW(hwnd, title.as_mut_ptr(), size);
        if read_len > 0 {
            title.set_len(read_len as usize);
        }
        String::from_utf16_lossy(&title)
    }
}

pub fn get_window_id(hwnd: HWND) -> i64 {
    // is this unique across windows? nope
    //use winapi::um::winuser::{GetWindowLongPtrW, GWLP_ID};
    //unsafe { GetWindowLongPtrW(hwnd, GWLP_ID) as i64 }
    hwnd as i64
}

// only at-tab able windows:
// https://devblogs.microsoft.com/oldnewthing/?p=24863
// https://stackoverflow.com/questions/7277366/why-does-enumwindows-return-more-windows-than-i-expected
// This does not seem to filter everything...
#[allow(dead_code)]
pub fn is_alt_tab_window(hwnd: HWND) -> bool {
    unsafe {
        use winapi::um::winuser::{
            GetAncestor, GetLastActivePopup, GetTitleBarInfo, GetWindowLongW, IsWindowVisible,
            GA_ROOTOWNER, GWL_EXSTYLE, STATE_SYSTEM_INVISIBLE, TITLEBARINFO, WS_EX_TOOLWINDOW,
        };
        if IsWindowVisible(hwnd) == 0 {
            log::debug!("{hwnd:?}: window invisible, return false");
            return false;
        }
        let mut hwnd_walk: HWND = std::ptr::null_mut();
        // Start at the root owner
        let mut hwnd_try: HWND = GetAncestor(hwnd, GA_ROOTOWNER);
        // See if we are the last active visible popup
        while hwnd_try != hwnd_walk {
            hwnd_walk = hwnd_try;
            hwnd_try = GetLastActivePopup(hwnd_walk);
            if IsWindowVisible(hwnd_try) != 0 {
                break;
            }
        }
        if hwnd_walk != hwnd {
            log::debug!("{hwnd:?}: window not root, return false");
            return false;
        }
        let mut tit = TITLEBARINFO {
            cbSize: std::mem::size_of::<TITLEBARINFO>() as u32,
            rcTitleBar: winapi::shared::windef::RECT {
                bottom: 0,
                left: 0,
                right: 0,
                top: 0,
            },
            rgstate: [0 as u32; 6],
        };
        // the following removes some task tray programs and "Program Manager"
        //ti.cbSize = sizeof(ti);
        /*GetTitleBarInfo(hwnd, &mut tit);
        if (tit.rgstate[0] & STATE_SYSTEM_INVISIBLE) != 0 {
            log::debug!("{hwnd:?}: rgstate STATE_SYSTEM_INVISIBLE, return false");
            return false;
        }*/
        // Tool windows should not be displayed either, these do not appear in the
        // task bar.
        if (GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW) != 0 {
            log::debug!("{hwnd:?}: GWL_EXSTYLE=WS_EX_TOOLWINDOW, return false");
            return false;
        }
        return true;
    }
}

// this was way harder than it should have been
// return None when no access rights
#[allow(dead_code)]
pub fn get_window_process(hwnd: HWND) -> Option<(i64, String)> {
    unsafe {
        use winapi::um::psapi::GetModuleFileNameExW;
        use winapi::um::winuser::GetWindowThreadProcessId;

        let mut proc_id: winapi::shared::minwindef::DWORD = 0;
        /*let thread_id =*/
        GetWindowThreadProcessId(hwnd, &mut proc_id);
        use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

        let phandle = winapi::um::processthreadsapi::OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            0,
            proc_id,
        );
        let ret = if phandle != std::ptr::null_mut() {
            // paths can be longer than MAX_PATH (smh). Most people query repeatedly with size*=2 until it fits.
            let size = winapi::shared::minwindef::MAX_PATH as u32;
            let mut title: Vec<u16> = Vec::with_capacity(size as usize);
            let read_len =
                GetModuleFileNameExW(phandle, std::ptr::null_mut(), title.as_mut_ptr(), size);
            if read_len > 0 {
                title.set_len(read_len as usize);
            }
            Some((proc_id as i64, String::from_utf16_lossy(&title)))
        } else {
            None
        };
        winapi::um::handleapi::CloseHandle(phandle);
        ret
    }
}

#[allow(dead_code)]
pub fn get_window_class_name(hwnd: HWND) -> String {
    unsafe {
        use winapi::um::winuser::GetClassNameW;

        let size = winapi::shared::minwindef::MAX_PATH as u32;
        let mut title: Vec<u16> = Vec::with_capacity(size as usize);
        let read_len = GetClassNameW(hwnd, title.as_mut_ptr(), size as i32);
        if read_len > 0 {
            title.set_len(read_len as usize);
        }
        String::from_utf16_lossy(&title)
    }
}

#[allow(dead_code)]
pub fn get_all_windows(filter_alt_tab: bool) -> Vec<WindowsWindow> {
    log::debug!("getting windows!");
    let mut vec = Vec::new();
    let a = |hwnd| -> EnumResult {
        if !filter_alt_tab || is_alt_tab_window(hwnd) {
            vec.push(map_hwnd(hwnd))
        } else {
            log::debug!(
                "Skipping window (not-alt-tabbable): {hwnd:?}: {}",
                get_window_title(hwnd)
            );
        }
        EnumResult::ContinueEnum
    };
    enum_windows(&a);
    vec
}

#[allow(dead_code)]
pub enum EnumResult {
    ContinueEnum,
    StopEnum,
}

// this is completely implicit in c...
#[allow(dead_code)]
pub fn enum_windows(mut enum_func: &dyn FnMut(HWND) -> EnumResult) -> i32 {
    use winapi::shared::minwindef::LPARAM;
    use winapi::um::winuser::EnumWindows;
    // TODO: can enum_func be a fnmut?
    extern "system" fn callback(hwnd: HWND, l_param: LPARAM) -> i32 {
        let enum_func_ptr = l_param as *mut &mut dyn FnMut(HWND) -> EnumResult;
        match unsafe { (*enum_func_ptr)(hwnd) } {
            EnumResult::ContinueEnum => 1,
            EnumResult::StopEnum => 0,
        }
    }
    let enum_func_ptr = &mut enum_func as *mut &dyn FnMut(HWND) -> EnumResult;
    unsafe { EnumWindows(Some(callback), enum_func_ptr as LPARAM) }
}

// https://github.com/rust-lang/rust/blob/5da10c01214a3d3ebec65b8ba6effada92a0673f/library/std/src/sys/windows/args.rs#L42
fn parse_lp_cmd_line(lp_cmd_line: &str) -> Vec<String> {
    const BACKSLASH: u8 = '\\' as u8;
    const QUOTE: u8 = '"' as u8;
    const TAB: u8 = '\t' as u8;
    const SPACE: u8 = ' ' as u8;
    let mut ret_val: Vec<String> = Vec::new();
    let mut cmd_line = lp_cmd_line.as_bytes();
    // The executable name at the beginning is special.
    cmd_line = match cmd_line[0] {
        // The executable name ends at the next quote mark,
        // no matter what.
        QUOTE => {
            let args = {
                let mut cut = cmd_line[1..].splitn(2, |&c| c == QUOTE);
                if let Some(exe) = cut.next() {
                    ret_val.push(String::from_utf8_lossy(exe).to_string());
                }
                cut.next()
            };
            if let Some(args) = args {
                args
            } else {
                return ret_val;
            }
        }
        // Implement quirk: when they say whitespace here,
        // they include the entire ASCII control plane:
        // "However, if lpCmdLine starts with any amount of whitespace, CommandLineToArgvW
        // will consider the first argument to be an empty string. Excess whitespace at the
        // end of lpCmdLine is ignored."
        0..=SPACE => {
            ret_val.push(String::new());
            &cmd_line[1..]
        }
        // The executable name ends at the next whitespace,
        // no matter what.
        _ => {
            let args = {
                let mut cut = cmd_line.splitn(2, |&c| c > 0 && c <= SPACE);
                if let Some(exe) = cut.next() {
                    ret_val.push(String::from_utf8_lossy(exe).to_string());
                }
                cut.next()
            };
            if let Some(args) = args {
                args
            } else {
                return ret_val;
            }
        }
    };
    let mut cur = Vec::new();
    let mut in_quotes = false;
    let mut was_in_quotes = false;
    let mut backslash_count: usize = 0;
    for &c in cmd_line {
        match c {
            // backslash
            BACKSLASH => {
                backslash_count += 1;
                was_in_quotes = false;
            }
            QUOTE if backslash_count % 2 == 0 => {
                cur.extend(std::iter::repeat(b'\\' as u8).take(backslash_count / 2));
                backslash_count = 0;
                if was_in_quotes {
                    cur.push('"' as u8);
                    was_in_quotes = false;
                } else {
                    was_in_quotes = in_quotes;
                    in_quotes = !in_quotes;
                }
            }
            QUOTE if backslash_count % 2 != 0 => {
                cur.extend(std::iter::repeat(b'\\' as u8).take(backslash_count / 2));
                backslash_count = 0;
                was_in_quotes = false;
                cur.push(b'"' as u8);
            }
            SPACE | TAB if !in_quotes => {
                cur.extend(std::iter::repeat(b'\\' as u8).take(backslash_count));
                if !cur.is_empty() || was_in_quotes {
                    ret_val.push(String::from_utf8_lossy(&cur[..]).to_string());
                    cur.truncate(0);
                }
                backslash_count = 0;
                was_in_quotes = false;
            }
            _ => {
                cur.extend(std::iter::repeat(b'\\' as u8).take(backslash_count));
                backslash_count = 0;
                was_in_quotes = false;
                cur.push(c);
            }
        }
    }
    cur.extend(std::iter::repeat(b'\\' as u8).take(backslash_count));
    // include empty quoted strings at the end of the arguments list
    if !cur.is_empty() || was_in_quotes || in_quotes {
        ret_val.push(String::from_utf8_lossy(&cur[..]).to_string());
    }
    ret_val
}

fn map_hwnd(hwnd: HWND) -> WindowsWindow {
    let exe_info = get_window_process(hwnd);
    let monk = exe_info.as_ref().and_then(|exe_info| match ol(exe_info.0) {
        Ok(e) => e,
        Err(e) => {
            log::warn!("error getting exe info {:?}", e);
            None
        }
    });
    WindowsWindow {
        window_id: get_window_id(hwnd),
        title: get_window_title(hwnd),
        wclass: get_window_class_name(hwnd),
        process_id: exe_info.as_ref().map(|exe_info| exe_info.0),
        exe: exe_info.map(|exe_info| exe_info.1),
        command_line: monk.as_ref().map(|m| parse_lp_cmd_line(&m.command_line)),
        process_started: monk.as_ref().map(|m| m.creation_date.0.with_timezone(&Utc)),
    }
}
