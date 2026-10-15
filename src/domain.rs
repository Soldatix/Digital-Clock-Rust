//! Local, deterministic clock utilities for the P2 candidate.
//! There is deliberately no network, system registry, or legacy WebView2 access here.

use chrono::{DateTime, Local, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::{Duration, Instant}};

pub const KNOWN_CITIES: [&str; 12] = [
    "America/New_York", "America/Los_Angeles", "Europe/London",
    "Europe/Paris", "Europe/Moscow", "Europe/Zagreb",
    "Asia/Tokyo", "Asia/Dubai", "Asia/Kolkata",
    "Australia/Sydney", "Pacific/Auckland", "Africa/Cairo",
];


#[derive(Clone, Copy, Debug)]
pub struct WorldCity {
    pub name: &'static str,
    pub zone: &'static str,
}

/// Representative city list spanning UTC offsets, DST and 30/45-minute zones.
/// Zone keys are unique so older portable P2 data remains compatible.
/// Some offsets (UTC-12, +14) have fewer than two inhabited reference cities.
pub const WORLD_CITIES: &[WorldCity] = &[
    WorldCity { name: "Honolulu", zone: "Pacific/Honolulu" },
    WorldCity { name: "Rarotonga", zone: "Pacific/Rarotonga" },
    WorldCity { name: "Pago Pago", zone: "Pacific/Pago_Pago" },
    WorldCity { name: "Alofi", zone: "Pacific/Niue" },
    WorldCity { name: "Anchorage", zone: "America/Anchorage" },
    WorldCity { name: "Juneau", zone: "America/Juneau" },
    WorldCity { name: "Marquesas", zone: "Pacific/Marquesas" },
    WorldCity { name: "Los Angeles", zone: "America/Los_Angeles" },
    WorldCity { name: "Vancouver", zone: "America/Vancouver" },
    WorldCity { name: "Tijuana", zone: "America/Tijuana" },
    WorldCity { name: "Denver", zone: "America/Denver" },
    WorldCity { name: "Phoenix", zone: "America/Phoenix" },
    WorldCity { name: "Edmonton", zone: "America/Edmonton" },
    WorldCity { name: "Chicago", zone: "America/Chicago" },
    WorldCity { name: "Mexico City", zone: "America/Mexico_City" },
    WorldCity { name: "Guatemala City", zone: "America/Guatemala" },
    WorldCity { name: "New York", zone: "America/New_York" },
    WorldCity { name: "Toronto", zone: "America/Toronto" },
    WorldCity { name: "Bogota", zone: "America/Bogota" },
    WorldCity { name: "Lima", zone: "America/Lima" },
    WorldCity { name: "Halifax", zone: "America/Halifax" },
    WorldCity { name: "Santiago", zone: "America/Santiago" },
    WorldCity { name: "Caracas", zone: "America/Caracas" },
    WorldCity { name: "La Paz", zone: "America/La_Paz" },
    WorldCity { name: "St Johns", zone: "America/St_Johns" },
    WorldCity { name: "Sao Paulo", zone: "America/Sao_Paulo" },
    WorldCity { name: "Buenos Aires", zone: "America/Argentina/Buenos_Aires" },
    WorldCity { name: "Montevideo", zone: "America/Montevideo" },
    WorldCity { name: "Noronha", zone: "America/Noronha" },
    WorldCity { name: "Nuuk", zone: "America/Nuuk" },
    WorldCity { name: "Praia", zone: "Atlantic/Cape_Verde" },
    WorldCity { name: "Azores", zone: "Atlantic/Azores" },
    WorldCity { name: "Reykjavik", zone: "Atlantic/Reykjavik" },
    WorldCity { name: "London", zone: "Europe/London" },
    WorldCity { name: "Dublin", zone: "Europe/Dublin" },
    WorldCity { name: "Accra", zone: "Africa/Accra" },
    WorldCity { name: "Zagreb", zone: "Europe/Zagreb" },
    WorldCity { name: "Berlin", zone: "Europe/Berlin" },
    WorldCity { name: "Paris", zone: "Europe/Paris" },
    WorldCity { name: "Rome", zone: "Europe/Rome" },
    WorldCity { name: "Zurich", zone: "Europe/Zurich" },
    WorldCity { name: "Madrid", zone: "Europe/Madrid" },
    WorldCity { name: "Lagos", zone: "Africa/Lagos" },
    WorldCity { name: "Athens", zone: "Europe/Athens" },
    WorldCity { name: "Helsinki", zone: "Europe/Helsinki" },
    WorldCity { name: "Cairo", zone: "Africa/Cairo" },
    WorldCity { name: "Johannesburg", zone: "Africa/Johannesburg" },
    WorldCity { name: "Kyiv", zone: "Europe/Kyiv" },
    WorldCity { name: "Moscow", zone: "Europe/Moscow" },
    WorldCity { name: "Istanbul", zone: "Europe/Istanbul" },
    WorldCity { name: "Riyadh", zone: "Asia/Riyadh" },
    WorldCity { name: "Nairobi", zone: "Africa/Nairobi" },
    WorldCity { name: "Tehran", zone: "Asia/Tehran" },
    WorldCity { name: "Dubai", zone: "Asia/Dubai" },
    WorldCity { name: "Muscat", zone: "Asia/Muscat" },
    WorldCity { name: "Baku", zone: "Asia/Baku" },
    WorldCity { name: "Kabul", zone: "Asia/Kabul" },
    WorldCity { name: "Karachi", zone: "Asia/Karachi" },
    WorldCity { name: "Tashkent", zone: "Asia/Tashkent" },
    WorldCity { name: "Yekaterinburg", zone: "Asia/Yekaterinburg" },
    WorldCity { name: "Mumbai / Delhi", zone: "Asia/Kolkata" },
    WorldCity { name: "Colombo", zone: "Asia/Colombo" },
    WorldCity { name: "Kathmandu", zone: "Asia/Kathmandu" },
    WorldCity { name: "Dhaka", zone: "Asia/Dhaka" },
    WorldCity { name: "Thimphu", zone: "Asia/Thimphu" },
    WorldCity { name: "Yangon", zone: "Asia/Yangon" },
    WorldCity { name: "Cocos Islands", zone: "Indian/Cocos" },
    WorldCity { name: "Bangkok", zone: "Asia/Bangkok" },
    WorldCity { name: "Jakarta", zone: "Asia/Jakarta" },
    WorldCity { name: "Ho Chi Minh City", zone: "Asia/Ho_Chi_Minh" },
    WorldCity { name: "Beijing", zone: "Asia/Shanghai" },
    WorldCity { name: "Singapore", zone: "Asia/Singapore" },
    WorldCity { name: "Manila", zone: "Asia/Manila" },
    WorldCity { name: "Perth", zone: "Australia/Perth" },
    WorldCity { name: "Eucla", zone: "Australia/Eucla" },
    WorldCity { name: "Tokyo", zone: "Asia/Tokyo" },
    WorldCity { name: "Seoul", zone: "Asia/Seoul" },
    WorldCity { name: "Adelaide", zone: "Australia/Adelaide" },
    WorldCity { name: "Darwin", zone: "Australia/Darwin" },
    WorldCity { name: "Sydney", zone: "Australia/Sydney" },
    WorldCity { name: "Brisbane", zone: "Australia/Brisbane" },
    WorldCity { name: "Port Moresby", zone: "Pacific/Port_Moresby" },
    WorldCity { name: "Lord Howe Island", zone: "Australia/Lord_Howe" },
    WorldCity { name: "Noumea", zone: "Pacific/Noumea" },
    WorldCity { name: "Honiara", zone: "Pacific/Guadalcanal" },
    WorldCity { name: "Auckland", zone: "Pacific/Auckland" },
    WorldCity { name: "Suva", zone: "Pacific/Fiji" },
    WorldCity { name: "Tarawa", zone: "Pacific/Tarawa" },
    WorldCity { name: "Chatham Islands", zone: "Pacific/Chatham" },
    WorldCity { name: "Apia", zone: "Pacific/Apia" },
    WorldCity { name: "Nukualofa", zone: "Pacific/Tongatapu" },
    WorldCity { name: "Kiritimati", zone: "Pacific/Kiritimati" },
    WorldCity { name: "Baker Island (UTC-12)", zone: "Etc/GMT+12" },
];

pub fn city_for_zone(zone: &str) -> Option<&'static WorldCity> {
    WORLD_CITIES.iter().find(|city| city.zone == zone)
}

pub fn zone_for_city_option(option: &str) -> Option<&'static str> {
    WORLD_CITIES.iter().find(|city| city.option_label() == option).map(|city| city.zone)
}

impl WorldCity {
    pub fn option_label(&self) -> String {
        format!("{} ({})", self.name, self.zone)
    }
}

pub fn world_city_time(zone: &str, now: DateTime<Utc>) -> Option<String> {
    let tz: Tz = zone.parse().ok()?;
    Some(now.with_timezone(&tz).format("%H:%M:%S  %d.%m.%Y.  UTC%:z").to_string())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alarm {
    pub id: u32,
    pub time: String,
    pub enabled: bool,
}

/// Strict 24-hour HH:MM parser; rejects whitespace, non-ASCII digits, and malformed times.
pub fn parse_alarm_time(value: &str) -> Result<(u8, u8), &'static str> {
    let bytes = value.as_bytes();
    if bytes.len() != 5 || bytes[2] != b':' ||
        ![0usize, 1, 3, 4].iter().all(|&i| bytes[i].is_ascii_digit()) {
        return Err("Alarm must use HH:MM (00:00–23:59)");
    }
    let h = (bytes[0] - b'0') * 10 + bytes[1] - b'0';
    let m = (bytes[3] - b'0') * 10 + bytes[4] - b'0';
    if h > 23 || m > 59 { return Err("Alarm must use HH:MM (00:00–23:59)"); }
    Ok((h, m))
}

pub fn parse_timer_duration(value: &str) -> Result<Duration, &'static str> {
    let parts: Vec<&str> = value.split(':').collect();
    if !(2..=3).contains(&parts.len()) ||
       parts.iter().any(|part| part.len() != 2 || !part.as_bytes().iter().all(u8::is_ascii_digit)) {
        return Err("Timer must use MM:SS or HH:MM:SS, two digits per field");
    }
    let components: Vec<u64> = parts.iter().map(|part| part.parse::<u64>().unwrap_or(999)).collect();
    let (hours, minutes, seconds) = if parts.len() == 3 {
        (components[0], components[1], components[2])
    } else {
        (0, components[0], components[1])
    };
    if hours > 99 || seconds > 59 || (parts.len() == 3 && minutes > 59) ||
        (parts.len() == 2 && minutes > 99) {
        return Err("Timer value out of range");
    }
    let total = hours * 3600 + minutes * 60 + seconds;
    if total == 0 { return Err("Timer must be longer than zero"); }
    Ok(Duration::from_secs(total))
}

pub fn format_duration(value: Duration) -> String {
    let n = value.as_secs();
    format!("{:02}:{:02}:{:02}", n / 3600, (n / 60) % 60, n % 60)
}

#[derive(Debug, Default)]
pub struct AlarmBook {
    pub alarms: Vec<Alarm>,
    fired_minute: HashMap<u32, i64>,
}

impl AlarmBook {
    pub fn from_alarms(alarms: Vec<Alarm>) -> Result<Self, String> {
        if alarms.len() > 100 { return Err("Too many alarms (max 100)".into()); }
        let mut seen = std::collections::HashSet::new();
        for a in &alarms {
            parse_alarm_time(&a.time).map_err(|error| error.to_owned())?;
            if a.id == 0 || a.id > i32::MAX as u32 || !seen.insert(a.id) { return Err("Duplicate or invalid alarm ID".into()); }
        }
        Ok(Self { alarms, fired_minute: HashMap::new() })
    }

    pub fn add(&mut self, time: &str) -> Result<(), String> {
        parse_alarm_time(time).map_err(|error| error.to_owned())?;
        if self.alarms.len() >= 100 { return Err("Maximum 100 alarms".into()); }
        let next = self.alarms.iter().map(|a| a.id).max().unwrap_or(0)
            .checked_add(1).ok_or("Alarm ID space exhausted")?;
        if next > i32::MAX as u32 { return Err("Alarm ID space exhausted".into()); }
        self.alarms.push(Alarm { id: next, time: time.to_owned(), enabled: true });
        Ok(())
    }

    pub fn remove_last(&mut self) -> bool {
        if let Some(last) = self.alarms.pop() {
            self.fired_minute.remove(&last.id);
            true
        } else { false }
    }

    pub fn toggle_last(&mut self) -> bool {
        if let Some(last) = self.alarms.last_mut() {
            last.enabled = !last.enabled;
            true
        } else { false }
    }

    pub fn remove(&mut self, id: u32) -> bool {
        let before = self.alarms.len();
        self.alarms.retain(|a| a.id != id);
        self.fired_minute.remove(&id);
        self.alarms.len() != before
    }

    pub fn toggle(&mut self, id: u32) -> bool {
        if let Some(alarm) = self.alarms.iter_mut().find(|a| a.id == id) {
            alarm.enabled = !alarm.enabled;
            if !alarm.enabled { self.fired_minute.remove(&id); }
            true
        } else { false }
    }

    pub fn update(&mut self, id: u32, time: &str) -> Result<(), String> {
        parse_alarm_time(time).map_err(str::to_owned)?;
        let alarm = self.alarms.iter_mut().find(|a| a.id == id)
            .ok_or("Alarm nije pronađen")?;
        alarm.time = time.to_string();
        self.fired_minute.remove(&id);
        Ok(())
    }

    /// Triggers once per alarm per UTC minute, including when the UI refreshes repeatedly.
    /// Requires the application to be running; this is not a background scheduled alarm.
    pub fn due(&mut self, now: DateTime<Local>) -> Vec<Alarm> {
        let minute_key = now.timestamp().div_euclid(60);
        let current_time = now.format("%H:%M").to_string();
        let mut due = Vec::new();
        for alarm in &self.alarms {
            if !alarm.enabled || alarm.time != current_time { continue; }
            if self.fired_minute.get(&alarm.id) == Some(&minute_key) { continue; }
            self.fired_minute.insert(alarm.id, minute_key);
            due.push(alarm.clone());
        }
        due
    }

    pub fn summary(&self) -> String {
        if self.alarms.is_empty() { return "Nema alarma. Unesi HH:MM i klikni Dodaj.".into(); }
        self.alarms.iter().map(|a| format!("#{:02}   {}   {}", a.id, a.time,
            if a.enabled { "UKLJUČEN" } else { "ISKLJUČEN" }))
            .collect::<Vec<_>>().join("\n")
    }
}

#[derive(Debug, Default)]
pub struct Countdown {
    remaining: Duration,
    started: Option<Instant>,
    completed: bool,
}

impl Countdown {
    pub fn configure(&mut self, duration: Duration) {
        self.remaining = duration;
        self.started = None;
        self.completed = false;
    }

    pub fn start_pause(&mut self, now: Instant) -> Result<bool, &'static str> {
        if let Some(started) = self.started.take() {
            self.remaining = self.remaining.saturating_sub(now.saturating_duration_since(started));
            return Ok(false);
        }
        if self.remaining.is_zero() { return Err("Prvo postavi trajanje timera"); }
        self.completed = false;
        self.started = Some(now);
        Ok(true)
    }

    pub fn reset(&mut self) { *self = Self::default(); }

    pub fn remaining_at(&self, now: Instant) -> Duration {
        match self.started {
            Some(started) => self.remaining.saturating_sub(now.saturating_duration_since(started)),
            None => self.remaining,
        }
    }

    pub fn is_running(&self) -> bool { self.started.is_some() }

    /// Returns true once when this active countdown reaches zero.
    pub fn tick(&mut self, now: Instant) -> bool {
        if self.started.is_some() && self.remaining_at(now).is_zero() {
            self.remaining = Duration::ZERO;
            self.started = None;
            if !self.completed {
                self.completed = true;
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Default)]
pub struct Stopwatch {
    elapsed: Duration,
    started: Option<Instant>,
    laps: Vec<Duration>,
}

impl Stopwatch {
    pub fn start_pause(&mut self, now: Instant) -> bool {
        if let Some(started) = self.started.take() {
            self.elapsed += now.saturating_duration_since(started);
            false
        } else {
            self.started = Some(now);
            true
        }
    }

    pub fn elapsed_at(&self, now: Instant) -> Duration {
        self.elapsed + self.started.map(|s| now.saturating_duration_since(s)).unwrap_or_default()
    }

    pub fn lap(&mut self, now: Instant) -> bool {
        if self.started.is_none() || self.laps.len() >= 100 { return false; }
        self.laps.push(self.elapsed_at(now));
        true
    }

    pub fn reset(&mut self) { *self = Self::default(); }

    pub fn is_running(&self) -> bool { self.started.is_some() }

    pub fn display(&self, now: Instant) -> String {
        let elapsed = self.elapsed_at(now);
        let seconds = elapsed.as_secs();
        format!("{:02}:{:02}:{:02}.{:02}", seconds / 3600,
            (seconds / 60) % 60, seconds % 60, elapsed.subsec_millis() / 10)
    }

    pub fn lap_summary(&self) -> String { self.lap_summary_localized("Nema krugova.", "Krug") }

    pub fn lap_summary_localized(&self, empty: &str, label: &str) -> String {
        if self.laps.is_empty() { return empty.into(); }
        self.laps.iter().enumerate().rev().take(8)
            .map(|(index, lap)| format!("{label} {:02}: {}", index + 1, format_duration(*lap)))
            .collect::<Vec<_>>().join("\n")
    }
}

pub fn validate_cities(values: &[String]) -> Result<(), String> {
    if values.len() > WORLD_CITIES.len() { return Err("Too many cities".into()); }
    let mut seen = std::collections::HashSet::new();
    for city in values {
        if city_for_zone(city).is_none() || !seen.insert(city) {
            return Err(format!("Unsupported or duplicate city: {city}"));
        }
    }
    Ok(())
}

pub fn world_summary(cities: &[String], now: DateTime<Utc>) -> String {
    if cities.is_empty() { return "Nema odabranih gradova.".into(); }
    cities.iter().filter_map(|city| {
        let descriptor = city_for_zone(city)?;
        let time = world_city_time(city, now)?;
        Some(format!("{:<23}  {}", descriptor.name, time))
    }).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    #[test]
    fn alarm_time_boundaries() {
        assert_eq!(parse_alarm_time("00:00"), Ok((0, 0)));
        assert_eq!(parse_alarm_time("23:59"), Ok((23, 59)));
        for invalid in ["24:00", "10:60", "9:15", " 09:15", "09:15 ", "aa:bb", "09:1a", "09:15\n"] {
            assert!(parse_alarm_time(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn timer_parse_and_limits() {
        assert_eq!(parse_timer_duration("05:00").unwrap().as_secs(), 300);
        assert_eq!(parse_timer_duration("01:02:03").unwrap().as_secs(), 3723);
        for invalid in ["00:00", "100:00:00", "00:60:00", "00:12:60", "1:20", "--:10"] {
            assert!(parse_timer_duration(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn countdown_pause_and_single_completion_event() {
        let start = Instant::now();
        let mut countdown = Countdown::default();
        countdown.configure(Duration::from_secs(10));
        assert_eq!(countdown.start_pause(start), Ok(true));
        assert_eq!(countdown.remaining_at(start + Duration::from_secs(4)).as_secs(), 6);
        assert_eq!(countdown.start_pause(start + Duration::from_secs(4)), Ok(false));
        assert_eq!(countdown.remaining_at(start + Duration::from_secs(99)).as_secs(), 6);
        assert_eq!(countdown.start_pause(start + Duration::from_secs(5)), Ok(true));
        assert!(!countdown.tick(start + Duration::from_secs(10)));
        assert!(countdown.tick(start + Duration::from_secs(11)));
        assert!(!countdown.tick(start + Duration::from_secs(12)));
    }

    #[test]
    fn stopwatch_laps_and_pause() {
        let t = Instant::now();
        let mut sw = Stopwatch::default();
        assert!(sw.start_pause(t));
        assert!(sw.lap(t + Duration::from_millis(1200)));
        assert!(!sw.start_pause(t + Duration::from_secs(2)));
        assert_eq!(sw.elapsed_at(t + Duration::from_secs(20)).as_secs(), 2);
        assert!(sw.lap_summary().contains("00:00:01"));
    }

    #[test]
    fn alarm_fires_once_per_minute() {
        let mut book = AlarmBook::default();
        book.add("15:20").unwrap();
        let date = Local.with_ymd_and_hms(2026, 10, 13, 15, 20, 7).single().unwrap();
        assert_eq!(book.due(date).len(), 1);
        assert_eq!(book.due(date + chrono::Duration::seconds(10)).len(), 0);
        assert!(book.due(date + chrono::Duration::minutes(1)).is_empty());
        assert_eq!(date.hour(), 15);
    }

    #[test]
    fn edit_toggle_delete_specific_alarm() {
        let mut book = AlarmBook::default();
        book.add("16:30").unwrap();
        book.add("17:30").unwrap();
        book.add("18:30").unwrap();
        assert!(book.update(2, "20:45").is_ok());
        assert_eq!(book.alarms[1].time, "20:45");
        assert!(book.toggle(1));
        assert!(!book.alarms[0].enabled);
        assert!(book.remove(2));
        assert_eq!(book.alarms.iter().map(|x| x.id).collect::<Vec<_>>(), vec![1, 3]);
        assert!(book.update(3, "25:00").is_err());
        assert_eq!(book.alarms[1].time, "18:30");
        assert!(!book.remove(99));
        assert!(!book.toggle(99));
    }

    #[test]
    fn world_clock_observes_summer_and_winter_offsets() {
        let london: Tz = "Europe/London".parse().unwrap();
        let winter = Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).single().unwrap();
        let summer = Utc.with_ymd_and_hms(2026, 7, 15, 12, 0, 0).single().unwrap();
        assert_eq!(winter.with_timezone(&london).hour(), 12);
        assert_eq!(summer.with_timezone(&london).hour(), 13);
    }

    #[test]
    fn city_allowlist_rejects_duplicates() {
        assert!(validate_cities(&["Europe/Zagreb".into(), "Asia/Tokyo".into()]).is_ok());
        assert!(validate_cities(&["Europe/Zagreb".into(), "Europe/Zagreb".into()]).is_err());
        assert!(validate_cities(&["Untrusted/City".into()]).is_err());
    }
}

#[cfg(test)]
mod p21_world_tests {
    use super::*;
    use chrono::{TimeZone, Offset};

    #[test]
    fn city_catalog_has_unique_valid_iana_zones() {
        assert!(WORLD_CITIES.len() >= 80);
        let mut zones = std::collections::HashSet::new();
        for city in WORLD_CITIES {
            assert!(!city.name.is_empty());
            assert!(zones.insert(city.zone));
            assert!(city.zone.parse::<Tz>().is_ok(), "{}", city.zone);
            assert_eq!(zone_for_city_option(&city.option_label()), Some(city.zone));
        }
    }

    #[test]
    fn p2_saved_cities_still_validate() {
        validate_cities(&vec!["Europe/Zagreb".into(), "Asia/Tokyo".into()]).unwrap();
        assert!(validate_cities(&vec!["Europe/Zagreb".into(), "Europe/Zagreb".into()]).is_err());
        assert!(validate_cities(&vec!["Invalid/Zone".into()]).is_err());
    }

    #[test]
    fn winter_summer_zagreb_and_half_hour_zones() {
        let jan = Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).single().unwrap();
        let jul = Utc.with_ymd_and_hms(2026, 7, 15, 12, 0, 0).single().unwrap();
        let zagreb: Tz = "Europe/Zagreb".parse().unwrap();
        assert_eq!(jan.with_timezone(&zagreb).offset().fix().local_minus_utc(), 3600);
        assert_eq!(jul.with_timezone(&zagreb).offset().fix().local_minus_utc(), 7200);
        let ktm: Tz = "Asia/Kathmandu".parse().unwrap();
        assert_eq!(jan.with_timezone(&ktm).offset().fix().local_minus_utc(), 5*3600+45*60);
        let chatham: Tz = "Pacific/Chatham".parse().unwrap();
        assert_eq!(jul.with_timezone(&chatham).offset().fix().local_minus_utc(), 12*3600+45*60);
    }
}

#[cfg(test)]
mod rc3_alarm_id_regression {
    use super::{Alarm, AlarmBook};

    #[test]
    fn alarm_id_must_fit_ui_signed_integer() {
        let mut book = AlarmBook::from_alarms(vec![Alarm {
            id: i32::MAX as u32,
            time: "12:34".to_owned(),
            enabled: true,
        }]).unwrap();
        assert!(book.add("13:35").is_err());
        assert_eq!(book.alarms.len(), 1);
    }
}
