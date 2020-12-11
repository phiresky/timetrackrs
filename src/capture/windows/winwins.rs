use super::winwins_types::*;
use crate::prelude::*;
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
impl Capturer for WindowsCapturer {
    fn capture(&mut self) -> anyhow::Result<EventData> {
        let focused_window = get_foreground_window().map(|f| get_window_id(f));
        Ok(EventData::windows_v1(WindowsEventData {
            os_info: self.os_info.clone(),
            focused_window,
            windows: get_all_windows(true),
        }))
    }
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
        GetTitleBarInfo(hwnd, &mut tit);
        if (tit.rgstate[0] & STATE_SYSTEM_INVISIBLE) != 0 {
            return false;
        }
        // Tool windows should not be displayed either, these do not appear in the
        // task bar.
        if (GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW) != 0 {
            return false;
        }
        return true;
    }
}

// this was way harder than it should have been
// return None when no access rights
#[allow(dead_code)]
pub fn get_window_exe_path(hwnd: HWND) -> Option<String> {
    unsafe {
        let re: Option<String>;
        use winapi::um::psapi::GetModuleFileNameExW;
        use winapi::um::winuser::GetWindowThreadProcessId;

        let mut proc_id: winapi::shared::minwindef::DWORD = 0;
        /*let thread_id =*/
        GetWindowThreadProcessId(hwnd, &mut proc_id);
        use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
        {
            let phandle = winapi::um::processthreadsapi::OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                0,
                proc_id,
            );
            if phandle != std::ptr::null_mut() {
                // paths can be longer than MAX_PATH (smh). Most people query repeatedly with size*=2 until it fits.
                let size = winapi::shared::minwindef::MAX_PATH as u32;
                let mut title: Vec<u16> = Vec::with_capacity(size as usize);
                let read_len =
                    GetModuleFileNameExW(phandle, std::ptr::null_mut(), title.as_mut_ptr(), size);
                if read_len > 0 {
                    title.set_len(read_len as usize);
                }
                re = Some(String::from_utf16_lossy(&title));
            } else {
                re = None;
            }
            winapi::um::handleapi::CloseHandle(phandle);
        };
        re
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
    let mut vec = Vec::new();
    let a = |hwnd| -> EnumResult {
        if is_alt_tab_window(hwnd) && filter_alt_tab {
            vec.push(map_hwnd(hwnd))
        }
        EnumResult::ContinueEnum
    };
    enum_windows(&a);
    vec
}

#[allow(dead_code)]
pub fn print_all_windows(filter_alt_tab: bool) {
    let a = |hwnd| -> EnumResult {
        if is_alt_tab_window(hwnd) && filter_alt_tab {
            print_window_info(hwnd);
        }
        EnumResult::ContinueEnum
    };
    enum_windows(&a);
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

#[allow(dead_code)]
pub fn print_window_info(hwnd: HWND) {
    let title = get_window_title(hwnd);
    let exe = get_window_exe_path(hwnd);
    let wclass = get_window_class_name(hwnd);
    println!(
        "Title:\t{:?}\nClass:\t{:?}\nPath:\t{:?}\n",
        title,
        wclass,
        exe.unwrap_or(String::from("---"))
    );
}

fn map_hwnd(hwnd: HWND) -> WindowsWindow {
    WindowsWindow {
        window_id: get_window_id(hwnd),
        title: get_window_title(hwnd),
        exe: get_window_exe_path(hwnd),
        wclass: get_window_class_name(hwnd),
    }
}
