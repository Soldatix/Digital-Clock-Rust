//! Native P2.8 candidate storage. Does not read or mutate the original Windows app,
//! legacy WebView2 profile, or P0 settings. Import will be an explicit later step.

use crate::domain::{Alarm, AlarmBook, validate_cities};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::{fs, io::{self, Write}, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};

const P2_FORMAT: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClockSettings {
    pub hour_24: bool,
    pub show_seconds: bool,
    pub show_date: bool,
    pub dark_mode: bool,
    pub background_color: String,
    pub clock_color: String,
    pub date_color: String,
    pub light_background_color: String,
    pub light_clock_color: String,
    pub light_date_color: String,
    pub clock_size_percent: u8,
    pub date_size_percent: u8,
    pub brightness_percent: u8,
    pub contrast_percent: u8,
    pub date_format: String,
    pub font_family: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_tool_size")]
    pub tool_size_percent: u8,
    #[serde(default = "default_bedside_brightness")]
    pub bedside_brightness_percent: u8,
}
fn default_language() -> String { "en".to_string() }
fn default_tool_size() -> u8 { 100 }
fn default_bedside_brightness() -> u8 { 35 }

impl Default for ClockSettings {
    fn default() -> Self {
        Self {
            hour_24: true, show_seconds: false, show_date: true, dark_mode: true,
            background_color: "#2400D0".into(),
            clock_color: "#DAFF00".into(), date_color: "#DAFF00".into(),
            light_background_color: "#F2F4FF".into(),
            light_clock_color: "#2E18C4".into(), light_date_color: "#2E18C4".into(),
            clock_size_percent: 90, date_size_percent: 75,
            brightness_percent: 100, contrast_percent: 100,
            date_format: "ddd dd.mm.yyyy.".into(), font_family: "Consolas".into(), language: default_language(),
            tool_size_percent: default_tool_size(), bedside_brightness_percent: default_bedside_brightness(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AlarmTone { #[default] Chime, Bell, Pulse }

impl AlarmTone {
    pub fn from_label(value: &str) -> Option<Self> {
        match value { "Chime" => Some(Self::Chime), "Bell" => Some(Self::Bell),
                      "Pulse" => Some(Self::Pulse), _ => None }
    }
    pub fn as_label(self) -> &'static str {
        match self { Self::Chime => "Chime", Self::Bell => "Bell", Self::Pulse => "Pulse" }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppearanceProfile {
    pub name: String,
    pub settings: ClockSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct P2State {
    pub format_version: u32,
    pub settings: ClockSettings,
    pub alarms: Vec<Alarm>,
    #[serde(default)]
    pub alarm_tone: AlarmTone,
    pub cities: Vec<String>,
    #[serde(default)]
    pub profiles: Vec<AppearanceProfile>,
}

impl Default for P2State {
    fn default() -> Self {
        Self {
            format_version: P2_FORMAT,
            settings: ClockSettings::default(),
            alarms: Vec::new(),
            alarm_tone: AlarmTone::Chime,
            profiles: Vec::new(),
            // Match the original src/main.js getSavedCities() defaults exactly.
            cities: vec!["America/New_York".into(), "Europe/London".into(),
                         "Asia/Tokyo".into(), "Australia/Sydney".into(),
                         "Europe/Moscow".into(), "Europe/Zagreb".into()],
        }
    }
}

pub fn data_file() -> io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let directory = exe.parent().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound,
        "Executable has no parent directory"))?;
    Ok(directory.join("Data").join("app-state-native-p28.json"))
}

pub const DATE_FORMATS: &[&str] = &[
    "dd.mm.yyyy.", "mm.dd.yyyy.", "dd.mmm.yyyy.",
    "ddd dd.mm.yyyy.", "day dd.mm.yyyy.",
];

pub const FONT_FAMILIES: &[&str] = &[
    "Consolas", "Arial", "Courier New", "Verdana", "Segoe UI",
    "Times New Roman", "Orbitron", "Roboto Mono", "Digital Numbers", "Segment7",
];

pub fn rgb_from_hex(value: &str) -> Option<(u8, u8, u8)> {
    if value.len() != 7 || !value.starts_with('#') { return None; }
    let bytes = value.as_bytes();
    if !bytes[1..].iter().all(|ch| ch.is_ascii_hexdigit()) { return None; }
    let r = u8::from_str_radix(&value[1..3], 16).ok()?;
    let g = u8::from_str_radix(&value[3..5], 16).ok()?;
    let b = u8::from_str_radix(&value[5..7], 16).ok()?;
    Some((r, g, b))
}

pub fn validate_appearance(settings: &ClockSettings) -> Result<(), String> {
    for c in [&settings.background_color, &settings.clock_color, &settings.date_color,
              &settings.light_background_color, &settings.light_clock_color, &settings.light_date_color] {
        if rgb_from_hex(c).is_none() { return Err("Neispravna boja; očekuje se #RRGGBB".into()); }
    }
    if !(50..=100).contains(&settings.clock_size_percent)
        || !(40..=100).contains(&settings.date_size_percent)
        || settings.brightness_percent > 100 || settings.contrast_percent > 100
        || !(60..=150).contains(&settings.tool_size_percent)
        || !(1..=100).contains(&settings.bedside_brightness_percent) {
        return Err("Veličina ili svjetlina/kontrast izvan granica".into());
    }
    if !DATE_FORMATS.contains(&settings.date_format.as_str()) {
        return Err("Nepodržani format datuma".into());
    }
    if !crate::i18n::supported(&settings.language) {
        return Err("Unsupported language".into());
    }
    if !FONT_FAMILIES.contains(&settings.font_family.as_str()) {
        return Err("Nepodržani font".into());
    }
    Ok(())
}

pub fn validate(state: &P2State) -> Result<(), String> {
    if state.format_version != P2_FORMAT { return Err("Unsupported P2 state version".into()); }
    AlarmBook::from_alarms(state.alarms.clone())?;
    validate_cities(&state.cities)?;
    validate_appearance(&state.settings)?;
    if state.profiles.len() > 100 { return Err("Previše profila (maks. 100)".into()); }
    let mut names = HashSet::new();
    for profile in &state.profiles {
        if profile.name.trim().is_empty() || profile.name.chars().count() > 100 {
            return Err("Naziv profila mora imati 1–100 znakova".into());
        }
        if !names.insert(profile.name.as_str()) { return Err("Duplikat naziva profila".into()); }
        validate_appearance(&profile.settings)?;
    }
    Ok(())
}

pub fn load(file: &Path) -> Result<P2State, String> {
    if !file.exists() { return Ok(P2State::default()); }
    let raw = fs::read_to_string(file).map_err(|e| format!("Cannot read P2 settings: {e}"))?;
    let result: P2State = serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid P2 JSON. File untouched: {e}"))?;
    validate(&result)?;
    Ok(result)
}

pub fn save(file: &Path, state: &P2State) -> io::Result<()> {
    validate(state).map_err(|msg| io::Error::new(io::ErrorKind::InvalidData, msg))?;
    let parent = file.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput,
        "Missing state directory"))?;
    fs::create_dir_all(parent)?;
    let serialized = serde_json::to_vec_pretty(state)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|error| io::Error::other(error.to_string()))?.as_nanos();
    let temp = file.with_extension(format!("json.{}.{nonce}.tmp", std::process::id()));
    let result = (|| -> io::Result<()> {
        let mut out = fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
        out.write_all(&serialized)?;
        out.sync_all()?;
        drop(out);
        // std::fs::rename replaces the destination on supported Windows targets.
        // A failed rename leaves the original state file unchanged.
        fs::rename(&temp, file)?;
        Ok(())
    })();
    if result.is_err() { let _ = fs::remove_file(&temp); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_file() -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("digital-clock-p21-tests-{}-{stamp}", std::process::id()))
            .join("app-state-native-p28.json")
    }

    #[test]
    fn default_state_is_valid() { validate(&P2State::default()).unwrap(); }

    #[test]
    fn default_clock_size_matches_new_90_percent_baseline() {
        assert_eq!(ClockSettings::default().clock_size_percent, 90);
        // Existing saved settings and profiles keep their stored percentages.
    }

    #[test]
    fn state_round_trip_and_replacement() {
        let path = unique_file();
        let mut state = P2State::default();
        save(&path, &state).unwrap();
        state.settings.dark_mode = false;
        state.alarms.push(Alarm { id: 1, time: "07:30".into(), enabled: true });
        save(&path, &state).unwrap();
        assert_eq!(load(&path).unwrap(), state);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn tone_round_trip_and_legacy_p1_field_default() {
        let path = unique_file();
        let mut state = P2State::default();
        state.alarm_tone = AlarmTone::Pulse;
        save(&path, &state).unwrap();
        assert_eq!(load(&path).unwrap().alarm_tone, AlarmTone::Pulse);
        // Old candidate records without alarm_tone default safely to Chime.
        let json = fs::read_to_string(&path).unwrap();
        // Construct canonical serialized data without the optional field.
        let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
        value.as_object_mut().unwrap().remove("alarm_tone");
        let defaulted: P2State = serde_json::from_value(value).unwrap();
        assert_eq!(defaulted.alarm_tone, AlarmTone::Chime);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn invalid_file_is_not_overwritten() {
        let path = unique_file();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "{ invalid json").unwrap();
        assert!(load(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "{ invalid json");
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn invalid_alarms_and_cities_fail_closed() {
        let mut state = P2State::default();
        state.alarms.push(Alarm { id: 1, time: "25:00".into(), enabled: true });
        assert!(validate(&state).is_err());
        state.alarms.clear();
        state.cities.push("Europe/Zagreb".into());
        assert!(validate(&state).is_err());
    }
    #[test]
    fn valid_settings_and_profiles_save_and_load() {
        let path = unique_file();
        let mut state = P2State::default();
        state.settings.background_color = "#AB12Ef".into();
        state.settings.date_format = "day dd.mm.yyyy.".into();
        state.profiles.push(AppearanceProfile {
            name: "Noć".into(), settings: state.settings.clone()
        });
        save(&path, &state).unwrap();
        assert_eq!(load(&path).unwrap(), state);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn reject_duplicate_profiles_and_invalid_hex_without_overwriting() {
        let path = unique_file();
        let mut state = P2State::default();
        save(&path, &state).unwrap();
        let before = fs::read(&path).unwrap();
        state.settings.clock_color = "#OOFFFF".into();
        assert!(save(&path, &state).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        state.settings.clock_color = "#FFFFFF".into();
        state.profiles.push(AppearanceProfile { name:"A".into(), settings: state.settings.clone() });
        state.profiles.push(AppearanceProfile { name:"A".into(), settings: state.settings.clone() });
        assert!(validate(&state).is_err());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn appearance_size_limit_100_and_light_palette_validation() {
        let mut settings = ClockSettings::default();
        settings.clock_size_percent = 101;
        assert!(validate_appearance(&settings).is_err());
        settings.clock_size_percent = 100;
        settings.date_size_percent = 101;
        assert!(validate_appearance(&settings).is_err());
        settings.date_size_percent = 100;
        settings.light_clock_color = "#FF8800".into();
        assert!(validate_appearance(&settings).is_ok());
        settings.light_clock_color = "invalid".into();
        assert!(validate_appearance(&settings).is_err());
    }

    #[test]
    fn rgb_validation_and_values() {
        assert_eq!(rgb_from_hex("#DAFF00"), Some((218,255,0)));
        for bad in ["FFFF00", "#GGDDFF", "#1234", "#000000FF", ""] {
            assert!(rgb_from_hex(bad).is_none());
        }
    }

}
