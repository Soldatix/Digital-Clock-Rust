//! Windows common file dialogs for optional external JSON backup export/import.
//! No additional crates, no powershell.exe and no console windows.
use std::path::PathBuf;

#[cfg(windows)]
mod windows {
    use super::PathBuf;
    use std::{ffi::{c_void, OsString}, os::windows::ffi::OsStringExt};

    #[repr(C)]
    struct OpenFileNameW {
        struct_size: u32,
        hwnd_owner: *mut c_void,
        instance: *mut c_void,
        filter: *const u16,
        custom_filter: *mut u16,
        max_custom_filter: u32,
        filter_index: u32,
        file: *mut u16,
        max_file: u32,
        file_title: *mut u16,
        max_file_title: u32,
        initial_dir: *const u16,
        title: *const u16,
        flags: u32,
        file_offset: u16,
        file_extension: u16,
        default_extension: *const u16,
        custom_data: isize,
        hook: *mut c_void,
        template_name: *const u16,
        reserved: *mut c_void,
        reserved_dword: u32,
        flags_ex: u32,
    }

    #[link(name = "comdlg32")]
    unsafe extern "system" {
        fn GetSaveFileNameW(info: *mut OpenFileNameW) -> i32;
        fn GetOpenFileNameW(info: *mut OpenFileNameW) -> i32;
        fn CommDlgExtendedError() -> u32;
    }

    const EXPLORER: u32 = 0x0008_0000;
    const PATH_MUST_EXIST: u32 = 0x0000_0800;
    const FILE_MUST_EXIST: u32 = 0x0000_1000;
    const OVERWRITE_PROMPT: u32 = 0x0000_0002;
    const NO_CHANGE_DIR: u32 = 0x0000_0008;

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn select_json_file(save: bool) -> Result<Option<PathBuf>, String> {
        select_file(save, false)
    }
    pub fn select_audio_file() -> Result<Option<PathBuf>, String> {
        select_file(false, true)
    }
    fn select_file(save: bool, audio: bool) -> Result<Option<PathBuf>, String> {
        // Buffer and UTF-16 strings remain alive until the modal Win32 dialog closes.
        let mut file = [0u16; 32768];
        if save {
            let suggestion = wide("DigitalClock-backup.json");
            file[..suggestion.len()].copy_from_slice(&suggestion);
        }
        let filter: Vec<u16> = (if audio {
            "Audio files\0*.wav;*.mp3;*.wma;*.m4a;*.aac\0All files\0*.*\0\0"
        } else { "JSON backup (*.json)\0*.json\0All files (*.*)\0*.*\0\0" })
            .encode_utf16().collect();
        let title = wide(if audio { "Select Digital Clock sound" } else if save { "Save Digital Clock backup" } else { "Open Digital Clock backup" });
        let extension = wide("json");
        let mut info = OpenFileNameW {
            struct_size: std::mem::size_of::<OpenFileNameW>() as u32,
            hwnd_owner: std::ptr::null_mut(),
            instance: std::ptr::null_mut(),
            filter: filter.as_ptr(),
            custom_filter: std::ptr::null_mut(),
            max_custom_filter: 0,
            filter_index: 1,
            file: file.as_mut_ptr(),
            max_file: file.len() as u32,
            file_title: std::ptr::null_mut(),
            max_file_title: 0,
            initial_dir: std::ptr::null(),
            title: title.as_ptr(),
            flags: EXPLORER | PATH_MUST_EXIST | NO_CHANGE_DIR
                | if save { OVERWRITE_PROMPT } else { FILE_MUST_EXIST },
            file_offset: 0,
            file_extension: 0,
            default_extension: extension.as_ptr(),
            custom_data: 0,
            hook: std::ptr::null_mut(),
            template_name: std::ptr::null(),
            reserved: std::ptr::null_mut(),
            reserved_dword: 0,
            flags_ex: 0,
        };
        let accepted = unsafe {
            if save { GetSaveFileNameW(&mut info) } else { GetOpenFileNameW(&mut info) }
        };
        if accepted == 0 {
            let error = unsafe { CommDlgExtendedError() };
            if error == 0 { return Ok(None); } // user clicked Cancel
            return Err(format!("Windows file dialog failed (0x{error:04X})"));
        }
        let len = file.iter().position(|&c| c == 0)
            .ok_or_else(|| "Windows file dialog returned an unterminated path".to_string())?;
        if len == 0 { return Err("Empty backup path".into()); }
        Ok(Some(PathBuf::from(OsString::from_wide(&file[..len]))))
    }
}

#[cfg(windows)]
pub use windows::{select_json_file, select_audio_file};

#[cfg(not(windows))]
pub fn select_audio_file() -> Result<Option<PathBuf>, String> {
    Err("File dialogs require Windows".into())
}

#[cfg(not(windows))]
pub fn select_json_file(_save: bool) -> Result<Option<PathBuf>, String> {
    Err("File dialogs require Windows".into())
}
