//! P2 temporary native tones. Does not load arbitrary audio files or use WebView2.
//! Windows Beep runs only on a worker thread to keep Slint UI responsive.
//! Three built-in patterns are distinct; legacy WAV/MP3/WMA/M4A/AAC parity is a later gate.
use crate::config::AlarmTone;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn Beep(frequency: u32, duration: u32) -> i32;
}

fn note(frequency: u32, duration_ms: u32) {
    #[cfg(windows)]
    unsafe {
        let _ = Beep(frequency, duration_ms);
    }
    #[cfg(not(windows))]
    { let _ = frequency; std::thread::sleep(Duration::from_millis(duration_ms as u64)); }
}

fn pattern(tone: AlarmTone, active: &AtomicBool) {
    let notes: &[(u32,u32)] = match tone {
        AlarmTone::Chime => &[(660,120), (880,180), (1046,250)],
        AlarmTone::Bell => &[(1200,240), (940,180), (1200,280)],
        AlarmTone::Pulse => &[(470,140), (470,140), (470,140)],
    };
    for &(freq, ms) in notes {
        if !active.load(Ordering::Relaxed) { return; }
        note(freq,ms);
        if !active.load(Ordering::Relaxed) { return; }
        std::thread::sleep(Duration::from_millis(70));
    }
}

fn responsive_pause(active: &AtomicBool, ms: usize, started: Instant, max_duration: Option<Duration>) {
    for _ in 0..(ms / 50) {
        if !should_continue(active, started, max_duration) { return; }
        std::thread::sleep(Duration::from_millis(50));
    }
}

const MAX_ALERT_SOUND: Duration = Duration::from_secs(5 * 60);

fn should_continue(active: &AtomicBool, started: Instant, max_duration: Option<Duration>) -> bool {
    active.load(Ordering::Relaxed)
        && max_duration.is_none_or(|duration| started.elapsed() < duration)
}

#[derive(Default)]
pub struct SoundEngine {
    current: Mutex<Option<Arc<AtomicBool>>>,
}

impl SoundEngine {
    pub fn stop(&self) {
        if let Ok(mut current) = self.current.lock() {
            if let Some(flag) = current.take() { flag.store(false, Ordering::Relaxed); }
        }
    }

    fn start(&self, tone: AlarmTone, repeat: bool, max_duration: Option<Duration>) {
        self.stop();
        let active = Arc::new(AtomicBool::new(true));
        if let Ok(mut current) = self.current.lock() {
            *current = Some(active.clone());
        }
        std::thread::spawn(move || {
            let started = Instant::now();
            loop {
                if !should_continue(&active, started, max_duration) { break; }
                pattern(tone, &active);
                if !repeat { break; }
                responsive_pause(&active, 750, started, max_duration);
            }
            active.store(false, Ordering::Relaxed);
        });
    }

    #[cfg(all(windows, feature = "windows-installer"))]
    fn custom(&self, tone: AlarmTone, file: Option<String>, repeat: bool, max_duration: Duration) {
        let Some(file) = file else { self.start(tone, repeat, if repeat { Some(max_duration) } else { None }); return; };
        self.stop();
        let active = Arc::new(AtomicBool::new(true));
        if let Ok(mut current) = self.current.lock() { *current = Some(active.clone()); }
        std::thread::spawn(move || {
            let started = Instant::now();
            let played = crate::windows_custom_audio::play_media(&file, &active, repeat, max_duration);
            if !played && should_continue(&active, started, Some(max_duration)) {
                // Device/codec unavailable: never leave an alarm silently muted.
                loop {
                    if !should_continue(&active, started, Some(max_duration)) { break; }
                    pattern(tone, &active);
                    if !repeat { break; }
                    responsive_pause(&active, 750, started, Some(max_duration));
                }
            }
            active.store(false, Ordering::Relaxed);
        });
    }

    #[cfg(all(windows, feature = "windows-installer"))]
    pub fn preview_with_file(&self, tone: AlarmTone, file: Option<String>) {
        self.custom(tone, file, false, Duration::from_secs(15));
    }
    #[cfg(all(windows, feature = "windows-installer"))]
    pub fn play_with_file(&self, tone: AlarmTone, file: Option<String>) {
        self.custom(tone, file, true, MAX_ALERT_SOUND);
    }

    pub fn preview(&self, tone: AlarmTone) { self.start(tone, false, None); }
    /// Alarm/timer repeats until acknowledged or a five-minute deadline expires.
    pub fn play_until_stopped(&self, tone: AlarmTone) { self.start(tone, true, Some(MAX_ALERT_SOUND)); }
}

impl Drop for SoundEngine {
    fn drop(&mut self) { self.stop(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn start_stop_does_not_block_gui_thread() {
        let engine = SoundEngine::default();
        engine.preview(AlarmTone::Chime);
        engine.stop();
        engine.play_until_stopped(AlarmTone::Pulse);
        engine.stop();
    }
}

#[cfg(test)]
mod p21_sound_tests {
    use super::*;
    #[test]
    fn alert_sound_deadline_is_five_minutes() {
        assert_eq!(MAX_ALERT_SOUND.as_secs(), 300);
        let active = AtomicBool::new(true);
        let now = Instant::now();
        assert!(should_continue(&active, now, Some(MAX_ALERT_SOUND)));
        active.store(false, Ordering::Relaxed);
        assert!(!should_continue(&active, now, Some(MAX_ALERT_SOUND)));
        active.store(true, Ordering::Relaxed);
        assert!(!should_continue(&active, now, Some(Duration::ZERO)));
    }
}
