//! Installer-only native Windows notification icon.
//! No additional Rust crate, no shell commands, no production registry writes.
//! Main Slint thread polls actions; the hidden Win32 window lives on its own thread.
#![cfg(all(windows, feature = "windows-installer"))]
use std::{ffi::c_void, mem, ptr, sync::{atomic::{AtomicIsize, AtomicU8, Ordering}, mpsc}, thread::{self, JoinHandle}, time::Duration};

type H = *mut c_void;
const WM_TRAY: u32 = 0x8001;
const WM_CLOSE: u32 = 0x0010;
const WM_DESTROY: u32 = 0x0002;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_LBUTTONDBLCLK: u32 = 0x0203;
const WM_RBUTTONUP: u32 = 0x0205;
const NIM_ADD: u32 = 0;
const NIM_DELETE: u32 = 2;
const NIF_MESSAGE: u32 = 1;
const NIF_ICON: u32 = 2;
const NIF_TIP: u32 = 4;
const MF_STRING: u32 = 0;
const TPM_RIGHTBUTTON: u32 = 2;
const TPM_RETURNCMD: u32 = 0x0100;
static ACTION: AtomicU8 = AtomicU8::new(0);
static TRAY_HWND: AtomicIsize = AtomicIsize::new(0);

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Point { x: i32, y: i32 }
#[repr(C)]
struct Msg { hwnd: H, message: u32, wparam: usize, lparam: isize, time: u32, pt: Point, lprivate: u32 }
impl Default for Msg { fn default()->Self{Self{hwnd:ptr::null_mut(),message:0,wparam:0,lparam:0,time:0,pt:Point::default(),lprivate:0}} }
#[repr(C)]
struct WndClass {
    style: u32,
    wnd_proc: Option<unsafe extern "system" fn(H,u32,usize,isize)->isize>,
    cls_extra: i32, wnd_extra: i32, instance: H,
    icon: H, cursor: H, background: H,
    menu_name: *const u16, class_name: *const u16,
}
#[repr(C)]
struct NotifyIconData {
    cb_size: u32, hwnd: H, uid: u32, flags: u32,
    callback_message: u32, icon: H, tip: [u16; 128],
    state: u32, state_mask: u32, info: [u16; 256],
    timeout_or_version: u32, info_title: [u16; 64], info_flags: u32,
    guid_item: [u8; 16], balloon_icon: H,
}
#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(wnd: *const WndClass) -> u16;
    fn CreateWindowExW(ex_style: u32, class: *const u16, name: *const u16, style: u32,
        x:i32,y:i32,w:i32,h:i32,parent:H,menu:H,instance:H,param:H)->H;
    fn DefWindowProcW(hwnd:H,msg:u32,w:usize,l:isize)->isize;
    fn DestroyWindow(hwnd:H)->i32;
    fn PostMessageW(hwnd:H,msg:u32,w:usize,l:isize)->i32;
    fn PostQuitMessage(code:i32);
    fn GetMessageW(msg:*mut Msg,hwnd:H,min:u32,max:u32)->i32;
    fn TranslateMessage(msg:*const Msg)->i32;
    fn DispatchMessageW(msg:*const Msg)->isize;
    fn LoadIconW(instance:H,name:*const u16)->H;
    fn CreatePopupMenu()->H;
    fn AppendMenuW(menu:H,flags:u32,id:usize,label:*const u16)->i32;
    fn GetCursorPos(point:*mut Point)->i32;
    fn SetForegroundWindow(hwnd:H)->i32;
    fn TrackPopupMenu(menu:H,flags:u32,x:i32,y:i32,reserved:i32,hwnd:H,rect:*const c_void)->i32;
    fn DestroyMenu(menu:H)->i32;
}
#[link(name = "shell32")]
unsafe extern "system" { fn Shell_NotifyIconW(message:u32,data:*mut NotifyIconData)->i32; }
#[link(name = "kernel32")]
unsafe extern "system" { fn GetModuleHandleW(name:*const u16)->H; }
fn wide(text:&str)->Vec<u16>{text.encode_utf16().chain(std::iter::once(0)).collect()}

fn menu(hwnd:H){
    unsafe {
        let menu=CreatePopupMenu();
        if menu.is_null(){return;}
        let show=wide("Open Digital Clock");
        let exit=wide("Exit Digital Clock");
        let _=AppendMenuW(menu, MF_STRING, 1001, show.as_ptr());
        let _=AppendMenuW(menu, MF_STRING, 1002, exit.as_ptr());
        let mut at=Point::default();
        if GetCursorPos(&mut at)!=0 {
            let _=SetForegroundWindow(hwnd);
            let selected=TrackPopupMenu(menu,TPM_RIGHTBUTTON|TPM_RETURNCMD,
                at.x,at.y,0,hwnd,ptr::null());
            if selected==1001 { ACTION.store(1,Ordering::SeqCst); }
            if selected==1002 { ACTION.store(2,Ordering::SeqCst); }
        }
        let _=DestroyMenu(menu);
    }
}
unsafe extern "system" fn window_proc(hwnd:H,msg:u32,w:usize,l:isize)->isize {
    match msg {
        WM_TRAY => {
            match l as u32 {
                WM_LBUTTONUP|WM_LBUTTONDBLCLK=>ACTION.store(1,Ordering::SeqCst),
                WM_RBUTTONUP=>menu(hwnd),
                _=>{},
            }
            0
        },
        WM_CLOSE => { unsafe { DestroyWindow(hwnd) }; 0 },
        WM_DESTROY => { unsafe { PostQuitMessage(0) }; 0 },
        _=>unsafe { DefWindowProcW(hwnd,msg,w,l) },
    }
}
fn icon_data(hwnd:H)->NotifyIconData {
    let icon=unsafe { LoadIconW(ptr::null_mut(), 32512usize as *const u16) };
    let mut data=NotifyIconData {
        cb_size:mem::size_of::<NotifyIconData>() as u32,hwnd,uid:1,
        flags:NIF_MESSAGE|NIF_ICON|NIF_TIP,callback_message:WM_TRAY,
        icon,tip:[0;128],state:0,state_mask:0,info:[0;256],
        timeout_or_version:0,info_title:[0;64],info_flags:0,
        guid_item:[0;16],balloon_icon:ptr::null_mut(),
    };
    for (dst,src) in data.tip.iter_mut().zip(wide("Digital Clock Rust Preview").into_iter()){
        *dst=src;
    }
    data
}
fn run_thread(init:mpsc::SyncSender<bool>) {
    unsafe {
        let module=GetModuleHandleW(ptr::null());
        let class=wide("AppsGames_DigitalClockRust_Tray_P2");
        let wclass=WndClass {style:0,wnd_proc:Some(window_proc),cls_extra:0,wnd_extra:0,
            instance:module,icon:ptr::null_mut(),cursor:ptr::null_mut(),
            background:ptr::null_mut(),menu_name:ptr::null(),class_name:class.as_ptr()};
        if RegisterClassW(&wclass)==0 { let _=init.send(false); return; }
        let name=wide("Digital Clock Tray Host");
        let hwnd=CreateWindowExW(0,class.as_ptr(),name.as_ptr(),0,0,0,0,0,
            ptr::null_mut(),ptr::null_mut(),module,ptr::null_mut());
        if hwnd.is_null(){ let _=init.send(false); return; }
        let mut data=icon_data(hwnd);
        if Shell_NotifyIconW(NIM_ADD,&mut data)==0 {
            let _=init.send(false);
            let _=DestroyWindow(hwnd);
            return;
        }
        TRAY_HWND.store(hwnd as isize,Ordering::SeqCst);
        let _=init.send(true);
        let mut msg=Msg::default();
        loop {
            let n=GetMessageW(&mut msg,ptr::null_mut(),0,0);
            if n<=0 {break;}
            let _=TranslateMessage(&msg);
            let _=DispatchMessageW(&msg);
        }
        let _=Shell_NotifyIconW(NIM_DELETE,&mut data);
        TRAY_HWND.store(0,Ordering::SeqCst);
    }
}
pub struct Tray { handle:Option<JoinHandle<()>> }
impl Tray {
    pub fn start()->Result<Self,String>{
        let (tx,rx)=mpsc::sync_channel(1);
        let handle=thread::spawn(move||run_thread(tx));
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(true)=>Ok(Self{handle:Some(handle)}),
            _=>{drop(handle);Err("Windows tray icon unavailable".into())},
        }
    }
}
impl Drop for Tray {
    fn drop(&mut self){
        let hwnd=TRAY_HWND.load(Ordering::SeqCst) as H;
        if !hwnd.is_null(){ unsafe {PostMessageW(hwnd,WM_CLOSE,0,0);} }
        if let Some(handle)=self.handle.take(){let _=handle.join();}
    }
}
/// 0 = no input; 1 = show; 2 = exit. Poll on the Slint main thread.
pub fn take_action()->u8{ACTION.swap(0,Ordering::SeqCst)}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn tray_messages_are_distinct(){assert_ne!(WM_TRAY,WM_CLOSE);assert_ne!(NIM_ADD,NIM_DELETE);}
    #[test] fn tooltip_capacity(){assert!(wide("Digital Clock Rust Preview").len()<=128);}
}
