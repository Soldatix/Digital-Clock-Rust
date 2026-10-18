//! Installer-only Windows custom audio playback and separate per-user preferences.
//! No shell, no .NET dependency; playback uses the system MCI device.
//! Format decoding depends on installed Windows codecs. Failed playback falls back to the built-in tone.
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}, sync::atomic::{AtomicBool, AtomicU64, Ordering}, time::{Duration, Instant}};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CustomAudioPrefs {
    pub alarm: Option<String>,
    pub timer: Option<String>,
}

fn allowed_ext(path: &Path) -> bool {
    path.extension().and_then(|v| v.to_str())
        .is_some_and(|v| matches!(v.to_ascii_lowercase().as_str(), "wav" | "mp3" | "wma" | "m4a" | "aac"))
}

pub fn validate_audio_file(path: &Path) -> Result<PathBuf, String> {
    if !allowed_ext(path) { return Err("Select WAV, MP3, WMA, M4A or AAC".into()); }
    let canonical = path.canonicalize().map_err(|e| e.to_string())?;
    let text = canonical.to_string_lossy();
    if text.chars().count() > 1000 || text.chars().any(|c| matches!(c, '"' | '\n' | '\r' | '\0')) {
        return Err("Audio path not safe for Windows MCI".into());
    }
    let metadata = fs::metadata(&canonical).map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > 100 * 1024 * 1024 {
        return Err("Audio file missing, non-regular or larger than 100 MiB".into());
    }
    Ok(canonical)
}

pub fn load(file: &Path) -> Result<CustomAudioPrefs, String> {
    if !file.exists() { return Ok(CustomAudioPrefs::default()); }
    let meta = fs::metadata(file).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > 16384 { return Err("Audio preferences type/size invalid".into()); }
    let value: CustomAudioPrefs = serde_json::from_slice(&fs::read(file).map_err(|e| e.to_string())?)
        .map_err(|e| format!("Invalid audio preferences (file preserved): {e}"))?;
    for item in [&value.alarm, &value.timer].into_iter().flatten() {
        if item.chars().count() > 1000 || !allowed_ext(Path::new(item)) { return Err("Invalid saved audio path".into()); }
    }
    Ok(value)
}

pub fn save(file: &Path, data: &CustomAudioPrefs) -> Result<(), String> {
    let parent = file.parent().ok_or("Missing audio preferences directory")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(data).map_err(|e| e.to_string())?;
    if bytes.len() > 16384 { return Err("Audio preference data too large".into()); }
    let temp = file.with_extension(format!("json.{}.tmp", std::process::id()));
    fs::write(&temp, bytes).map_err(|e| e.to_string())?;
    if let Err(err) = fs::rename(&temp, file) {
        let _ = fs::remove_file(&temp);
        return Err(format!("Cannot commit audio preferences: {err}"));
    }
    Ok(())
}

pub fn label(value: &Option<String>) -> String {
    value.as_deref().and_then(|p| Path::new(p).file_name().and_then(|f| f.to_str()))
        .unwrap_or("Built-in tone").to_string()
}

#[link(name = "winmm")]
unsafe extern "system" {
    fn mciSendStringW(command: *const u16, result: *mut u16, size: u32, callback: usize) -> u32;
}

fn wide(text: &str) -> Vec<u16> { text.encode_utf16().chain(std::iter::once(0)).collect() }
fn command(text: &str) -> bool {
    unsafe { mciSendStringW(wide(text).as_ptr(), std::ptr::null_mut(), 0, 0) == 0 }
}
fn status(alias: &str) -> Option<String> {
    let mut buffer = [0_u16; 128];
    let code = unsafe { mciSendStringW(wide(&format!("status {alias} mode")).as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32, 0) };
    if code != 0 { return None; }
    let n = buffer.iter().position(|&v| v == 0).unwrap_or(buffer.len());
    Some(String::from_utf16_lossy(&buffer[..n]).to_ascii_lowercase())
}

/// Returns true when the media was opened and started successfully.
/// The caller owns the fallback native Beep pattern for unsupported codecs/devices.
pub fn play_media(file: &str, active: &AtomicBool, repeat: bool, max_duration: Duration) -> bool {
    let path = match validate_audio_file(Path::new(file)) { Ok(p) => p, Err(_) => return false };
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let alias = format!("dc_audio_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed));
    // MCI commands are string-based: reject embedded quotes/control characters in validate_audio_file.
    // Rust canonicalize() may return a Win32 verbatim path (\\?\). MCI expects a normal DOS/UNC path.
    let canonical_text = path.to_string_lossy();
    let mci_file = if let Some(rest) = canonical_text.strip_prefix(r#"\\?\UNC\"#) {
        format!(r"\\{}", rest)
    } else if let Some(rest) = canonical_text.strip_prefix(r#"\\?\"#) {
        rest.to_string()
    } else {
        canonical_text.to_string()
    };
    if !command(&format!("open \"{}\" alias {alias}", mci_file)) { return false; }
    let started = Instant::now();
    let mut played = false;
    while active.load(Ordering::Relaxed) && started.elapsed() < max_duration {
        if !command(&format!("play {alias} from 0")) { break; }
        played = true;
        loop {
            if !active.load(Ordering::Relaxed) || started.elapsed() >= max_duration { break; }
            std::thread::sleep(Duration::from_millis(100));
            match status(&alias).as_deref() {
                Some("stopped") => break,
                Some("playing") | Some("paused") | Some("seeking") | Some("open") => {},
                _ => break,
            }
        }
        if !repeat { break; }
        std::thread::sleep(Duration::from_millis(150));
    }
    let _ = command(&format!("stop {alias}"));
    let _ = command(&format!("close {alias}"));
    played
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn extensions_validate_without_external_codec_assumptions() {
        for ext in ["wav", "mp3", "wma", "m4a", "aac"] { assert!(allowed_ext(Path::new(&format!("sound.{ext}")))); }
        for ext in ["exe", "cmd", "js", "json"] { assert!(!allowed_ext(Path::new(&format!("bad.{ext}")))); }
    }
    #[test] fn empty_preferences_are_safe() {
        let prefs=CustomAudioPrefs::default();
        assert!(prefs.alarm.is_none() && prefs.timer.is_none());
        assert_eq!(label(&prefs.alarm), "Built-in tone");
    }
}
