//! Windows-local single-instance guard for this Portable application's state file.
//! A named mutex is derived from the normalized state path, so separate
//! Portable folders are independent while two copies sharing one Data folder
//! cannot silently overwrite each other's settings.
use std::{ffi::c_void, io, path::Path, ptr};

type Handle = *mut c_void;
const ERROR_ALREADY_EXISTS: u32 = 183;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateMutexW(attributes: *mut c_void, initial_owner: i32, name: *const u16) -> Handle;
    fn GetLastError() -> u32;
    fn CloseHandle(handle: Handle) -> i32;
}
#[link(name = "user32")]
unsafe extern "system" {
    fn MessageBoxW(window: Handle, message: *const u16, title: *const u16, flags: u32) -> i32;
}

pub struct InstanceGuard(Handle);

impl InstanceGuard {
    pub fn acquire(state_file: &Path) -> io::Result<Option<Self>> {
        // Windows filenames are case-insensitive in the typical Portable setup.
        // Resolve the already-existing EXE folder, *not* the state file:
        // Data/app-state-native-p28.json might not exist on first launch.
        let exe_folder = state_file.parent()
            .and_then(|data_dir| data_dir.parent())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing Portable application directory"))?
            .canonicalize()?;
        let canonical = exe_folder.to_string_lossy().to_lowercase();
        // Stable FNV-1a (no randomized std::collections::hash_map::DefaultHasher).
        let mut hash = 0xcbf29ce484222325_u64;
        for byte in canonical.as_bytes() {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
        }
        let name: Vec<u16> = format!("Local\\AppsGamesDigitalClock_{hash:016X}")
            .encode_utf16().chain(std::iter::once(0)).collect();
        let handle = unsafe { CreateMutexW(ptr::null_mut(), 0, name.as_ptr()) };
        if handle.is_null() { return Err(io::Error::last_os_error()); }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe { CloseHandle(handle); }
            return Ok(None);
        }
        Ok(Some(Self(handle)))
    }
}
impl Drop for InstanceGuard {
    fn drop(&mut self) { unsafe { CloseHandle(self.0); } }
}

pub fn show_already_open_message() {
    let message: Vec<u16> = "Digital Clock is already open from this Portable folder.\0"
        .encode_utf16().collect();
    let title: Vec<u16> = "Digital Clock\0".encode_utf16().collect();
    unsafe { MessageBoxW(ptr::null_mut(), message.as_ptr(), title.as_ptr(), 0x40); }
}
