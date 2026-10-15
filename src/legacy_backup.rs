//! Read-only validation of the published Digital Clock backup format v1.
//! Exact keys/rules are taken from src/js/backup.js at the audited legacy HEAD.
//! Not wired to the P2 UI. No files are opened or modified by this module.

use crate::domain::{KNOWN_CITIES, parse_alarm_time};
use serde_json::{Map, Value};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
pub struct BackupCounts {
    pub profiles: usize,
    pub cities: usize,
    pub alarms: usize,
}

fn object<'a>(v: &'a Value, field: &str) -> Result<&'a Map<String, Value>, String> {
    v.as_object().ok_or_else(|| format!("Invalid object: {field}"))
}

fn array<'a>(v: &'a Value, field: &str, limit: usize) -> Result<&'a Vec<Value>, String> {
    let arr = v.as_array().ok_or_else(|| format!("Invalid array: {field}"))?;
    if arr.len() > limit { return Err(format!("Too many entries: {field}")); }
    Ok(arr)
}

fn str_required<'a>(v: &'a Map<String, Value>, field: &str) -> Result<&'a str, String> {
    v.get(field).and_then(Value::as_str).ok_or_else(|| format!("Missing string: {field}"))
}

fn bool_required(v: &Map<String, Value>, field: &str) -> Result<(), String> {
    if !v.get(field).is_some_and(Value::is_boolean) { return Err(format!("Invalid boolean: {field}")); }
    Ok(())
}

fn allowed(v: &Map<String, Value>, field: &str, values: &[&str]) -> Result<(), String> {
    let value = str_required(v, field)?;
    if !values.contains(&value) { return Err(format!("Invalid value for {field}")); }
    Ok(())
}

fn color(v: &Map<String, Value>, field: &str) -> Result<(), String> {
    let s = str_required(v, field)?;
    if s.len() != 7 || !s.starts_with('#') || !s[1..].bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("Invalid color: {field}"));
    }
    Ok(())
}

fn range_string(v: &Map<String, Value>, field: &str, min: f64, max: f64) -> Result<(), String> {
    let s = str_required(v, field)?;
    let num: f64 = s.trim().parse().map_err(|_| format!("Invalid number: {field}"))?;
    if !num.is_finite() || num < min || num > max { return Err(format!("Out of range: {field}")); }
    Ok(())
}

fn sound(v: Option<&Value>, field: &str) -> Result<(), String> {
    let Some(sound) = v else { return Ok(()); };
    if sound.is_null() { return Ok(()); }
    let o = object(sound, field)?;
    match (str_required(o, "kind")?, str_required(o, "value")?) {
        ("builtin", "chime" | "bell" | "pulse") => Ok(()),
        ("custom", "custom") => {
            let name = str_required(o, "name")?;
            if name.trim().is_empty() || name.encode_utf16().count() > 260 {
                return Err(format!("Invalid custom sound name: {field}"));
            }
            // Legacy JS sanitizes overlong or non-string optional paths to "";
            // this read-only validator has no need to retain or open them.
            let _ = o.get("path");
            Ok(())
        }
        _ => Err(format!("Invalid sound setting: {field}")),
    }
}

fn settings(v: &Value) -> Result<(), String> {
    let o = object(v, "settings")?;
    for field in ["backgroundColor", "satFontColor", "datumFontColor"] { color(o, field)?; }
    allowed(o, "fontSelect", &[
        "Arial, sans-serif", "'Orbitron', sans-serif", "'Roboto Mono', monospace",
        "'Rajdhani', sans-serif", "'Play', sans-serif", "'JetBrains Mono', monospace",
        "'Digital Numbers', sans-serif", "Times New Roman, serif", "Courier New, monospace",
        "Verdana, sans-serif", "'Segment7Standard', monospace", "monospace",
    ])?;
    for (field, lo, hi) in [
        ("satFontSize", 5.0, 40.0), ("datumFontSize", 3.0, 25.0),
        ("brightness", 0.0, 1.0), ("contrast", 0.0, 1.0),
    ] { range_string(o, field, lo, hi)?; }
    allowed(o, "timeFormat", &["12", "24"])?;
    allowed(o, "dateFormat", &[
        "dd.mm.yyyy.", "mm.dd.yyyy.", "dd.mmm.yyyy.", "ddd dd.mm.yyyy.", "day dd.mm.yyyy.",
    ])?;
    allowed(o, "language", &["en", "hr", "de", "it", "es"])?;
    for field in ["showSeconds", "showDate", "isNightModeActive", "isAutoSizeActive"] { bool_required(o, field)?; }
    if o.contains_key("bedsideBrightness") { range_string(o, "bedsideBrightness", 10.0, 70.0)?; }
    if o.contains_key("languageWasSelectedByUser") { bool_required(o, "languageWasSelectedByUser")?; }
    sound(o.get("alarmSound"), "alarmSound")?;
    sound(o.get("timerSound"), "timerSound")?;
    Ok(())
}

pub fn validate_backup_json(raw: &str) -> Result<BackupCounts, String> {
    let parsed: Value = serde_json::from_str(raw).map_err(|e| format!("Invalid JSON: {e}"))?;
    let envelope = object(&parsed, "backup envelope")?;
    if str_required(envelope, "app")? != "digital-clock" ||
       envelope.get("formatVersion").and_then(Value::as_f64) != Some(1.0) {
        return Err("Not a supported Digital Clock v1 backup".into());
    }
    let data = object(envelope.get("data").ok_or("Backup data missing")?, "data")?;
    settings(data.get("settings").ok_or("Backup settings missing")?)?;

    let profiles = array(data.get("profiles").ok_or("Profiles missing")?, "profiles", 100)?;
    let mut profile_names = HashSet::new();
    for p in profiles {
        let obj = object(p, "profile")?;
        let raw_name = str_required(obj, "name")?;
        let name = raw_name.trim();
        if name.is_empty() || raw_name.encode_utf16().count() > 100 || !profile_names.insert(name.to_owned()) {
            return Err("Invalid or duplicate profile name".into());
        }
        settings(obj.get("settings").ok_or("Profile settings missing")?)?;
    }

    let cities = array(data.get("cities").ok_or("Cities missing")?, "cities", KNOWN_CITIES.len())?;
    let mut seen_cities = HashSet::new();
    for city in cities {
        let c = city.as_str().ok_or("City not a string")?;
        if !KNOWN_CITIES.contains(&c) || !seen_cities.insert(c.to_owned()) {
            return Err("Invalid or duplicate world clock city".into());
        }
    }

    let alarms = array(data.get("alarms").ok_or("Alarms missing")?, "alarms", 100)?;
    let mut seen_alarm_ids = HashSet::new();
    for alarm in alarms {
        let o = object(alarm, "alarm")?;
        let id = o.get("id").ok_or("Alarm ID missing")?;
        let key = if let Some(s) = id.as_str() {
            if s.is_empty() || s.encode_utf16().count() > 100 { return Err("Invalid alarm ID".into()); }
            s.to_owned()
        } else if let Some(number) = id.as_number() {
            let n = number.as_f64().ok_or("Invalid numeric alarm ID")?;
            if !n.is_finite() { return Err("Invalid numeric alarm ID".into()); }
            if n.fract() == 0.0 && n.abs() <= 9_007_199_254_740_991.0 {
                format!("{n:.0}")
            } else { number.to_string() }
        } else { return Err("Invalid alarm ID type".into()); };
        if !seen_alarm_ids.insert(key) { return Err("Duplicate alarm ID".into()); }
        parse_alarm_time(str_required(o, "time")?).map_err(|err| err.to_owned())?;
        bool_required(o, "isActive")?;
    }
    Ok(BackupCounts { profiles: profiles.len(), cities: cities.len(), alarms: alarms.len() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> Value {
        serde_json::json!({
            "app": "digital-clock",
            "formatVersion": 1,
            "data": {
                "settings": {
                    "backgroundColor": "#ffffff", "satFontColor": "#000000", "datumFontColor": "#000000",
                    "fontSelect": "Arial, sans-serif", "satFontSize": "20", "datumFontSize": "10",
                    "brightness": "1", "contrast": "1", "timeFormat": "24", "dateFormat": "dd.mm.yyyy.",
                    "showSeconds": true, "showDate": true, "language": "hr",
                    "isNightModeActive": false, "isAutoSizeActive": true
                },
                "profiles": [],
                "cities": ["Europe/Zagreb", "Asia/Tokyo"],
                "alarms": [{"id": 1, "time": "07:00", "isActive": true}]
            }
        })
    }

    #[test]
    fn recognizes_old_backup_without_changing_data() {
        let fixture = valid().to_string();
        assert_eq!(validate_backup_json(&fixture).unwrap(), BackupCounts { profiles: 0, cities: 2, alarms: 1 });
    }

    #[test]
    fn rejects_other_app_or_format() {
        let mut v = valid();
        v["app"] = Value::String("untrusted".into());
        assert!(validate_backup_json(&v.to_string()).is_err());
        v["app"] = Value::String("digital-clock".into());
        v["formatVersion"] = Value::from(2);
        assert!(validate_backup_json(&v.to_string()).is_err());
    }

    #[test]
    fn rejects_duplicate_alarm_and_invalid_color() {
        let mut v = valid();
        v["data"]["alarms"] = serde_json::json!([
            {"id": 1, "time": "07:00", "isActive": true},
            {"id": 1, "time": "08:00", "isActive": true}
        ]);
        assert!(validate_backup_json(&v.to_string()).is_err());
        v = valid();
        v["data"]["settings"]["backgroundColor"] = Value::String("not-a-color".into());
        assert!(validate_backup_json(&v.to_string()).is_err());
    }

    #[test]
    fn rejects_duplicate_city_and_invalid_sound() {
        let mut v = valid();
        v["data"]["cities"] = serde_json::json!(["Europe/Zagreb", "Europe/Zagreb"]);
        assert!(validate_backup_json(&v.to_string()).is_err());
        v = valid();
        v["data"]["settings"]["alarmSound"] = serde_json::json!({"kind": "custom", "value": "custom", "name": ""});
        assert!(validate_backup_json(&v.to_string()).is_err());
    }
}
