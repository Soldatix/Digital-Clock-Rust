//! Experimental Windows-native .scr argument adapter for the shared Rust+Slint clock.
//! /s = fullscreen Slint clock, /c = Settings, /p HWND = native child preview.
//! Never modifies SCRNSAVE.EXE registry settings; installer does not auto-activate.
use std::{ffi::c_void, io, ptr, thread, time::Duration};
use chrono::Local;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode { Normal, Screen, Configure, Preview(isize) }

pub fn mode() -> Mode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(first) = args.first() else { return Mode::Normal };
    let arg = first.trim().to_ascii_lowercase();
    if arg == "/s" || arg == "-s" { return Mode::Screen; }
    if arg == "/c" || arg == "-c" || arg.starts_with("/c:") || arg.starts_with("-c:") {
        return Mode::Configure;
    }
    if arg == "/p" || arg == "-p" || arg.starts_with("/p:") || arg.starts_with("-p:") {
        let handle_text = arg.split_once(':').map(|(_, handle)| handle)
            .or_else(|| args.get(1).map(|arg| arg.as_str()))
            .unwrap_or("");
        return Mode::Preview(handle_text.parse::<isize>().unwrap_or_default());
    }
    Mode::Normal
}

#[repr(C)]
struct LastInputInfo { cb_size: u32, time: u32 }
#[repr(C)]
#[derive(Default)]
struct Rect { left: i32, top: i32, right: i32, bottom: i32 }
#[repr(C)]
#[derive(Default)]
struct Point { x: i32, y: i32 }
#[repr(C)]
#[derive(Default)]
struct Message {
    hwnd: *mut c_void,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    pt: Point,
    private: u32,
}

#[link(name="user32")]
unsafe extern "system" {
    fn GetLastInputInfo(info: *mut LastInputInfo) -> i32;
    fn GetClientRect(hwnd: *mut c_void, rect: *mut Rect) -> i32;
    fn IsWindow(hwnd: *mut c_void) -> i32;
    fn CreateWindowExW(ex_style: u32, class: *const u16, title: *const u16,
        style: u32, x: i32, y: i32, width: i32, height: i32,
        parent: *mut c_void, menu: *mut c_void, instance: *mut c_void,
        param: *mut c_void) -> *mut c_void;
    fn SetWindowTextW(hwnd: *mut c_void, text: *const u16) -> i32;
    fn PeekMessageW(msg: *mut Message, hwnd: *mut c_void, min: u32, max: u32, flags: u32) -> i32;
    fn TranslateMessage(msg: *const Message) -> i32;
    fn DispatchMessageW(msg: *const Message) -> isize;
    fn DestroyWindow(hwnd: *mut c_void) -> i32;
}

fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }

pub fn last_input_tick() -> Option<u32> {
    let mut input = LastInputInfo { cb_size: std::mem::size_of::<LastInputInfo>() as u32, time: 0 };
    if unsafe { GetLastInputInfo(&mut input) } != 0 { Some(input.time) } else { None }
}

pub fn run_preview(parent_value: isize) -> Result<(), Box<dyn std::error::Error>> {
    if parent_value == 0 { return Err("Windows screensaver preview parent HWND missing".into()); }
    let parent = parent_value as *mut c_void;
    let mut rect = Rect::default();
    if unsafe { IsWindow(parent) } == 0 || unsafe { GetClientRect(parent, &mut rect) } == 0 {
        return Err(io::Error::last_os_error().into());
    }
    // This is a real child of the Windows screensaver preview parent.
    // Preview is intentionally native/simple; /s uses the full Slint clock.
    const WS_CHILD: u32 = 0x4000_0000;
    const WS_VISIBLE: u32 = 0x1000_0000;
    const SS_CENTER: u32 = 0x0000_0001;
    const SS_CENTERIMAGE: u32 = 0x0000_0200;
    let class = wide("STATIC");
    let caption = wide("Digital Clock");
    let child = unsafe { CreateWindowExW(0, class.as_ptr(), caption.as_ptr(),
        WS_CHILD | WS_VISIBLE | SS_CENTER | SS_CENTERIMAGE,
        0, 0, (rect.right - rect.left).max(1), (rect.bottom - rect.top).max(1),
        parent, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) };
    if child.is_null() { return Err(io::Error::last_os_error().into()); }
    let mut previous = String::new();
    while unsafe { IsWindow(parent) } != 0 && unsafe { IsWindow(child) } != 0 {
        let next = Local::now().format("Digital Clock  %H:%M:%S").to_string();
        if previous != next {
            let label = wide(&next);
            unsafe { SetWindowTextW(child, label.as_ptr()); }
            previous = next;
        }
        let mut msg = Message::default();
        while unsafe { PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, 1) } != 0 {
            unsafe { TranslateMessage(&msg); DispatchMessageW(&msg); }
        }
        thread::sleep(Duration::from_millis(150));
    }
    if unsafe { IsWindow(child) } != 0 { unsafe { DestroyWindow(child); } }
    Ok(())
}
