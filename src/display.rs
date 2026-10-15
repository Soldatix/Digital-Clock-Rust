//! Transient display modes. Native Windows keep-awake only while Bedside Mode is active.
#[derive(Default)]
pub struct DisplaySession { pub active: bool, pub previous_fullscreen: bool }
#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" { fn SetThreadExecutionState(flags: u32) -> u32; }
#[cfg(windows)]
pub fn keep_display_awake(enabled: bool) -> bool {
    const ES_CONTINUOUS: u32 = 0x80000000;
    const ES_DISPLAY_REQUIRED: u32 = 0x00000002;
    unsafe { SetThreadExecutionState(ES_CONTINUOUS | if enabled { ES_DISPLAY_REQUIRED } else { 0 }) != 0 }
}
#[cfg(not(windows))]
pub fn keep_display_awake(_enabled: bool) -> bool { true }

/// Deterministic drift while the built-in screensaver is visible.
/// Small displacement protects against static pixels but stays inside the clock margin.
pub fn saver_drift(second: u64) -> (i32, i32) {
    fn triangle(t: u64, amplitude: i32, period: u64) -> i32 {
        let phase = (t % period) as i32;
        let half = (period / 2) as i32;
        if phase <= half { -amplitude + (2 * amplitude * phase / half) }
        else { amplitude - (2 * amplitude * (phase - half) / half) }
    }
    (triangle(second, 18, 72), triangle(second + 15, 12, 56))
}

/// Brightness applies only to the application's bedside digits,
/// not to the physical screen brightness or global Windows settings.
pub fn bedside_color(percent: u8) -> slint::Color {
    let value = (8.0 + (percent.clamp(1, 100) as f64 / 100.0).powf(1.7) * 180.0)
        .round().clamp(1.0, 255.0) as u8;
    slint::Color::from_rgb_u8(value, value, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn screensaver_drift_moves_and_is_bounded() {
        let positions: Vec<_> = (0..144).map(saver_drift).collect();
        assert!(positions.iter().all(|(x,y)| x.abs() <= 18 && y.abs() <= 12));
        assert_ne!(positions[0], positions[10]);
        assert_ne!(positions[10], positions[60]);
    }
    #[test]
    fn bedside_dimming_is_monotonic() {
        assert!(bedside_color(1).red() < bedside_color(35).red());
        assert!(bedside_color(35).red() < bedside_color(100).red());
    }
}
