#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod config;
#[cfg(all(windows, feature = "windows-installer"))]
mod windows_custom_audio;
#[cfg(all(windows, feature = "windows-installer"))]
mod windows_tray;
#[cfg(all(windows, feature = "windows-installer"))]
mod windows_screensaver;
#[cfg(windows)]
mod single_instance;
mod domain;
mod sound;
mod backup;
mod file_dialog;
mod i18n;
mod display;
#[cfg(test)]
mod legacy_backup;

use chrono::{DateTime, Local, Utc, Datelike};
use config::{AlarmTone, ClockSettings, AppearanceProfile, P2State};
use domain::{AlarmBook, Countdown, Stopwatch};
use slint::{Color, ComponentHandle, ModelRc, Timer, TimerMode};
use std::{cell::RefCell, path::Path, rc::Rc, time::{Duration, Instant}};

slint::slint! {
    import { Button, LineEdit, ComboBox, ScrollView, Slider } from "std-widgets.slint";
    import "../assets/fonts/Orbitron-Regular.ttf";
    import "../assets/fonts/RobotoMono-Regular.ttf";
    import "../assets/fonts/DigitalNumbers-Regular.ttf";
    import "../assets/fonts/Segment7Standard.otf";


    // RC3.1: single keyboard/pointer/accessibility entry point, without changing icon art.
    component AccessibleTap inherits FocusScope {
        in property <string> label;
        // Delayed property-change focus executes within this component's own scope.
        in property <int> refocus-request: 0;
        changed refocus-request => {
            if (self.refocus-request > 0) { self.focus(); }
        }
        callback activated();
        accessible-role: button;
        accessible-label: root.label;
        accessible-action-default => { root.activated(); }
        focus-on-tab-navigation: true;
        key-pressed(event) => {
            if (event.text == Key.Return || event.text == Key.Space) {
                root.activated();
                return accept;
            }
            return reject;
        }
        Rectangle {
            width: parent.width;
            height: parent.height;
            background: transparent;
            border-color: #FFE27A;
            border-width: root.has-focus ? 2px : 0px;
        }
        TouchArea {
            width: parent.width;
            height: parent.height;
            clicked => { root.activated(); }
        }
    }

    export struct AlarmRow { id: int, time: string, enabled: bool }
    export struct CityRow { zone: string, name: string, local-time: string }
    export struct DonationRow { coin: string, address: string }

    export component DigitalClockUI inherits Window {
        title: "Digital Clock";
        preferred-width: 1080px;
        preferred-height: 780px;
        min-width: 840px;
        background: root.selected-page == 0 ? (root.display-mode == 2 ? #000000 : (root.dark-mode ? root.home-background : rgb(242, 244, 255))) : root.tool-canvas;
        full-screen: root.fullscreen-mode;
        forward-focus: hotkeys;

        in-out property <string> clock-text: "00:00:00";
        // Clock face is rendered as two fragments. HH:MM keeps its size when seconds toggle.
        in-out property <string> clock-main-text: "00:00";
        in-out property <string> clock-suffix-text: ":00";
        in-out property <string> date-text: "01.01.2026.";
        in-out property <string> status-text: "Digital Clock";
        in-out property <bool> storage-warning: false;
        in-out property <[AlarmRow]> alarm-rows;
        in-out property <int> selected-alarm-id: 0;
        // Time is selected, never typed: neither digits nor ':' can be erased.
        in-out property <[string]> alarm-hours;
        in-out property <[string]> alarm-minutes;
        in-out property <string> add-alarm-hour: "07";
        in-out property <string> add-alarm-minute: "00";
        in-out property <string> edit-alarm-hour: "07";
        in-out property <string> edit-alarm-minute: "00";
        in-out property <string> alarm-tone: "Chime";
        in-out property <[string]> timer-hours;
        in-out property <[string]> timer-minutes;
        in-out property <string> timer-hour: "00";
        in-out property <string> timer-minute: "05";
        in-out property <string> timer-second: "00";
        in-out property <string> countdown-text: "00:00:00";
        in-out property <string> countdown-button: "Start";
        in-out property <string> stopwatch-text: "00:00:00.00";
        in-out property <string> stopwatch-button: "Start";
        in-out property <string> laps-text: "";
        in-out property <[CityRow]> city-rows;
        in-out property <[string]> city-options;
        in-out property <string> selected-city: "Zagreb (Europe/Zagreb)";
        in-out property <string> alert-text: "";
        in-out property <bool> alert-active: false;
        in-out property <bool> hour-24: true;
        in-out property <bool> show-seconds: true;
        in-out property <bool> show-date: true;
        in-out property <bool> dark-mode: true;
        in-out property <bool> fullscreen-mode: false;
        in-out property <int> display-mode: 0; // 0 normal, 1 in-app saver, 2 bedside
        in-out property <int> saver-shift-x: 0;
        in-out property <int> saver-shift-y: 0;
        in-out property <int> bedside-brightness-pct: 35;
        in-out property <color> bedside-clock-color: #555555;
        in-out property <int> tool-scale: 100;
        in-out property <string> language-code: "en";
        in-out property <bool> language-menu: false;
        in-out property <[string]> backup-files;
        in-out property <string> selected-backup: "";
        in-out property <string> backup-status: "";
        in-out property <[DonationRow]> donation-rows;
        in-out property <string> info-copy-feedback: "";
        in-out property <string> last-copy-address: "";
        in-out property <bool> last-copy-success: false;
        in-out property <string> txt-copy: "Copy";
        in-out property <string> txt-copied: "Address copied";
        in-out property <string> txt-copy-failed: "Could not copy";
        in-out property <string> txt-about-app-name: "Multifunctional Digital Clock";
        in-out property <string> txt-visit-apps: "Visit Apps & Games ↗";
        in-out property <string> txt-paypal-methods: "PayPal · Debit / Credit Card · Apple Pay";
        in-out property <string> txt-stripe-methods: "Debit / Credit Card · Link · Digital wallets";
        in-out property <string> txt-donation-methods: "You can make donations to:";
        in-out property <string> txt-paypal-description: "Pay securely with PayPal or other payment options offered by PayPal Checkout.";
        in-out property <string> txt-stripe-description: "Pay securely by card or with payment methods available through Stripe Checkout.";
        in-out property <string> txt-paypal-action: "Donate with PayPal ↗";
        in-out property <string> txt-stripe-action: "Donate with Stripe ↗";
        in-out property <string> txt-crypto-wallet: "Crypto Wallet";
        in-out property <string> txt-payment-note: "Available payment methods may vary by country, device and payment provider.";
        in-out property <string> txt-crypto-description: "You can also support the project using cryptocurrency.";
        in-out property <string> txt-timer-hours: "Hours";
        in-out property <string> txt-timer-minutes: "Minutes";
        in-out property <string> txt-timer-seconds: "Seconds";
        in-out property <string> txt-alarm: "Alarmi";
        in-out property <string> txt-timer: "Timer";
        in-out property <string> txt-stopwatch: "Štoperica";
        in-out property <string> txt-world: "Svjetski sat";
        in-out property <string> txt-settings: "Postavke";
        in-out property <string> txt-backup: "Backup";
        in-out property <string> txt-close: "Zatvori";
        in-out property <string> txt-add: "Dodaj";
        in-out property <string> txt-remove: "Ukloni";
        in-out property <string> txt-start: "Pokreni";
        in-out property <string> txt-reset: "Reset";
        in-out property <string> txt-font: "Font";
        in-out property <string> txt-new: "Novi";
        in-out property <string> txt-edit: "Uredi";
        in-out property <string> txt-save-time: "Spremi vrijeme";
        in-out property <string> txt-tone: "Ton alarma";
        // RC3: separate installer-only audio selection without changing Portable state.
        in-out property <bool> installed-audio-enabled: false;
        in-out property <string> custom-alarm-label: "Built-in tone";
        in-out property <string> custom-timer-label: "Built-in tone";
        in-out property <string> txt-custom-audio: "Custom audio file";
        in-out property <string> txt-choose-sound: "Choose file";
        in-out property <string> txt-reset-sound: "Use built-in";
        in-out property <string> txt-preview: "Isprobaj zvuk";
        in-out property <string> txt-enable: "Uključi";
        in-out property <string> txt-disable: "Isključi";
        in-out property <string> txt-add-city: "Dodaj grad";
        in-out property <string> txt-set: "Postavi";
        in-out property <string> txt-lap: "Krug";
        in-out property <string> txt-clock-date: "Sat i datum";
        in-out property <string> txt-format-date: "Format datuma";
        in-out property <string> txt-background: "Podloga";
        in-out property <string> txt-clock: "Sat";
        in-out property <string> txt-date: "Datum";
        in-out property <string> txt-clock-size: "Veličina sata";
        in-out property <string> txt-date-size: "Veličina datuma";
        in-out property <string> txt-brightness: "Svjetlina";
        in-out property <string> txt-contrast: "Kontrast";
        in-out property <string> txt-profiles: "Profili";
        in-out property <string> txt-save-profile: "Spremi novi profil";
        in-out property <string> txt-load-profile: "Učitaj profil";
        in-out property <string> txt-delete-profile: "Izbriši profil";
        in-out property <string> txt-seconds: "Sekunde";
        in-out property <string> txt-hide-hex: "Hide HEX";
        in-out property <string> txt-custom-hex: "Custom HEX color";
        in-out property <string> txt-profile-placeholder: "Enter profile name";
        in-out property <string> txt-windowed: "Exit fullscreen";
        in-out property <string> txt-fullscreen: "Fullscreen";
        in-out property <string> txt-light-theme: "Light theme";
        in-out property <string> txt-dark-theme: "Dark theme";
        in-out property <string> txt-backup-export: "Export / Spremi backup";
        in-out property <string> txt-backup-save-as: "Save backup as...";
        in-out property <string> txt-backup-open-file: "Open backup file...";
        in-out property <string> txt-backup-import: "Import / Vrati backup";
        in-out property <string> txt-timer-note: "Timer je završen";
        in-out property <string> txt-time-format: "Format vremena";
        in-out property <string> txt-theme: "Tema";
        in-out property <string> txt-restore-default: "Vrati početni izgled";
        in-out property <string> txt-exit-mode: "Exit (Esc)";
        in-out property <string> txt-bedside-brightness: "Bedside brightness";
        in-out property <string> txt-tool-size: "Digits size";
        in-out property <string> txt-language: "Language";
        in-out property <string> txt-backup-folder: "Local copies in Data/Backups";
        in-out property <string> txt-backup-select: "Select a backup to restore";
        in-out property <string> txt-color-hint: "Colors: choose a swatch for the current theme";
        in-out property <string> txt-hex-hint: "Custom HEX colors (#RRGGBB)";
        in-out property <string> txt-apply-hex: "Apply HEX colors";
        in-out property <string> txt-alarm-help: "To edit an alarm, click Edit. Sound stops after 5 min.";
        in-out property <string> txt-world-help: "Remove any city individually. DST is automatic.";
        in-out property <string> txt-clock-view: "CLOCK VIEW";
        in-out property <string> txt-portable-hint: "Native Windows UI without WebView2. Local data beside EXE.";
        in-out property <string> txt-confirm: "Confirm";
        in-out property <string> txt-dev-build: "Digital Clock v0.3.8";
        in-out property <string> txt-limit-short: "Windows .scr, custom sounds and Save As are not finished.";
        in-out property <string> txt-24-hours: "24-hour";
        in-out property <string> txt-12-hours: "12-hour";
        in-out property <string> txt-yes: "ON";
        in-out property <string> txt-no: "OFF";
        in-out property <string> txt-about-title: "Information / Donations";
        in-out property <string> txt-about-intro: "Free digital clock with a customizable display and useful time tools.";
        in-out property <string> txt-about-features: "Main features";
        in-out property <string> txt-about-list: "Digital clock and date · multiple alarms · timer · stopwatch with laps · world clock · profiles · backup";
        in-out property <string> txt-about-portable: "Portable edition: settings, profiles and cities are stored locally in Data beside the EXE.";
        in-out property <string> txt-about-limits: "Status of this build";
        in-out property <string> txt-about-limit-note: "Portable screensaver works inside the app. Windows .scr, custom sounds and some Windows integrations are not included.";
        in-out property <string> txt-about-donate: "Support the project";
        in-out property <string> txt-about-donate-note: "The app is free. Learn more about Apps & Games and supporting the project:";
        in-out property <string> txt-timer-help: "Countdown complete";
        in-out property <int> selected-page: 0;
        // Home overlay: 0 = none, 1 = info/feature notice, 2 = appearance settings.
        in-out property <int> home-panel: 0;
        // Monotonically increasing requests; each Esc requests another refocus.
        in-out property <int> return-language-focus: 0;
        in-out property <int> return-info-focus: 0;
        in-out property <int> return-settings-focus: 0;
        in-out property <string> home-message: "";
        // Editable native appearance; data is kept in a P2.6-only state file.
        in-out property <color> home-background: #2400d0;
        in-out property <color> home-clock-color: #daff00;
        in-out property <color> home-date-color: #daff00;
        // RC3.2: derived from active appearance; never persisted separately.
        in-out property <color> tool-canvas: #0e1234;
        in-out property <color> tool-panel: #1b1a55;
        in-out property <color> tool-card: #26285d;
        in-out property <color> tool-display: #13164f;
        in-out property <color> tool-accent: #daff00;
        in-out property <color> tool-text: #f4f7ff;
        in-out property <color> tool-muted: #c5d2e8;
        in-out property <color> tool-display-text: #daff00;
        in-out property <string> bg-hex: "#2400D0";
        in-out property <string> time-hex: "#DAFF00";
        in-out property <string> date-hex: "#DAFF00";
        in-out property <int> clock-scale: 100;
        in-out property <int> date-scale: 75;
        in-out property <int> brightness-pct: 100;
        in-out property <int> contrast-pct: 100;
        in-out property <string> font-family-choice: "Consolas";
        in-out property <string> date-format-choice: "ddd dd.mm.yyyy.";
        in-out property <[string]> font-choices;
        in-out property <[string]> date-format-choices;
        in-out property <[string]> profile-names;
        in-out property <string> selected-profile: "";
        in-out property <string> profile-name-input: "";
        in-out property <bool> advanced-colors-visible: false;
        in-out property <string> profile-feedback: "";

        callback toggle-hour-mode();
        callback toggle-seconds();
        callback toggle-date();
        callback toggle-theme();
        callback apply-colors(string, string, string);
        callback choose-color(string, string);
        callback set-clock-scale(float);
        callback set-date-scale(float);
        callback set-brightness(float);
        callback set-contrast(float);
        // Fast in-memory color preview while the mouse is moving; no disk writes.
        callback preview-lighting(float, float);
        callback set-date-format(string);
        callback set-font-family(string);
        callback save-profile(string);
        callback load-profile(string);
        callback remove-profile(string);
        callback reset-appearance();
        callback add-alarm(string, string);
        callback select-alarm(int);
        callback update-alarm(int, string, string);
        callback remove-alarm(int);
        callback toggle-alarm(int);
        callback select-alarm-tone(string);
        callback preview-alarm-tone();
        callback choose-alarm-audio();
        callback clear-alarm-audio();
        callback choose-timer-audio();
        callback clear-timer-audio();
        callback set-timer(string);
        callback start-pause-timer();
        callback reset-timer();
        callback start-pause-stopwatch();
        callback lap-stopwatch();
        callback reset-stopwatch();
        callback add-world-city();
        callback remove-world-city(string);
        callback dismiss-alert();
        callback enter-display-mode(int);
        callback exit-display-mode();
        callback focus-display-hotkeys();
        callback preview-bedside(float);
        callback set-bedside-brightness(float);
        callback set-tool-scale(float);
        callback select-language(string);
        callback create-backup();
        callback save-backup-as();
        callback open-external-backup();
        callback restore-backup(string);
        callback copy-donation-address(string);

        focus-display-hotkeys => { hotkeys.focus(); }
        hotkeys := FocusScope {
            width: 100%;
            height: 100%;
            capture-key-pressed(event) => {
                if (event.text == Key.Escape && root.display-mode != 0) {
                    root.exit-display-mode();
                    return accept;
                }
                if (event.text == Key.Escape && root.language-menu) {
                    root.language-menu = false;
                    root.return-language-focus += 1;
                    return accept;
                }
                if (event.text == Key.Escape && root.home-panel == 1) {
                    root.home-panel = 0;
                    root.return-info-focus += 1;
                    return accept;
                }
                if (event.text == Key.Escape && root.home-panel == 2) {
                    root.home-panel = 0;
                    root.return-settings-focus += 1;
                    return accept;
                }
                if (event.text == Key.Escape && root.home-panel != 0) {
                    root.home-panel = 0;
                    return accept;
                }
                if (event.text == Key.Escape && root.fullscreen-mode) {
                    root.fullscreen-mode = false;
                    return accept;
                }
                return reject;
            }
            VerticalLayout {
            width: parent.width;
            height: parent.height;
            padding: root.selected-page == 0 ? 0px : 20px;
            spacing: root.selected-page == 0 ? 0px : 12px;

            // P2.4 - faithful home surface, separate from the already-tested tool panels.
            if root.selected-page == 0 : Rectangle {
                vertical-stretch: 1;
                background: root.display-mode == 2 ? #000000 : root.home-background;

                // Original-style fullscreen control, top-left.
                Rectangle {
                    x: 18px; y: 23px; width: 44px; height: 44px; visible: root.display-mode == 0;
                    background: rgb(245, 246, 250);
                    border-radius: 22px;
                    Image { source: @image-url("../assets/icons/fullscreen.svg"); x: 10px; y: 10px; width: 24px; height: 24px; image-fit: contain; }
                    AccessibleTap { width: parent.width; height: parent.height; label: root.fullscreen-mode ? root.txt-windowed : root.txt-fullscreen; activated => { root.fullscreen-mode = !root.fullscreen-mode; } }
                }

                // Clock features in the same order and position as the original app.
                Rectangle {
                    x: parent.width / 2 - 167px;
                    visible: root.display-mode == 0;
                    y: 23px;
                    width: 334px;
                    height: 44px;
                    HorizontalLayout { spacing: 14px;
Rectangle {
                        width: 44px; height: 44px;
                        background: rgb(245, 246, 250);
                        border-radius: 22px;
                        Image {
                            source: @image-url("../assets/icons/alarm.svg");
                            x: 10px; y: 10px;
                            width: 24px; height: 24px;
                            image-fit: contain;
                        }
                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-alarm; activated => { root.selected-page = 1; root.home-panel = 0; } }
                    }
Rectangle {
                        width: 44px; height: 44px;
                        background: rgb(245, 246, 250);
                        border-radius: 22px;
                        Image {
                            source: @image-url("../assets/icons/world.svg");
                            x: 10px; y: 10px;
                            width: 24px; height: 24px;
                            image-fit: contain;
                        }
                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-world; activated => { root.selected-page = 4; root.home-panel = 0; } }
                    }
Rectangle {
                        width: 44px; height: 44px;
                        background: rgb(245, 246, 250);
                        border-radius: 22px;
                        Image {
                            source: @image-url("../assets/icons/stopwatch.svg");
                            x: 10px; y: 10px;
                            width: 24px; height: 24px;
                            image-fit: contain;
                        }
                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-stopwatch; activated => { root.selected-page = 3; root.home-panel = 0; } }
                    }
Rectangle {
                        width: 44px; height: 44px;
                        background: rgb(245, 246, 250);
                        border-radius: 22px;
                        Image {
                            source: @image-url("../assets/icons/hourglass.svg");
                            x: 10px; y: 10px;
                            width: 24px; height: 24px;
                            image-fit: contain;
                        }
                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-timer; activated => { root.selected-page = 2; root.home-panel = 0; } }
                    }
Rectangle {
                        width: 44px; height: 44px;
                        background: rgb(245, 246, 250);
                        border-radius: 22px;
                        Image {
                            source: @image-url("../assets/icons/screen.svg");
                            x: 10px; y: 10px;
                            width: 24px; height: 24px;
                            image-fit: contain;
                        }
                        AccessibleTap { width: parent.width; height: parent.height; label: "Screen saver"; activated => { root.enter-display-mode(1); } }
                    }
Rectangle {
                        width: 44px; height: 44px;
                        background: rgb(245, 246, 250);
                        border-radius: 22px;
                        Image {
                            source: @image-url("../assets/icons/bed.svg");
                            x: 10px; y: 10px;
                            width: 24px; height: 24px;
                            image-fit: contain;
                        }
                        AccessibleTap { width: parent.width; height: parent.height; label: "Bedside mode"; activated => { root.enter-display-mode(2); } }
                    }
                    }
                }

                // Information, appearance/night theme and settings in the top-right.
                Rectangle {
                    x: parent.width - 238px;
                    visible: root.display-mode == 0;
                    y: 23px;
                    width: 220px;
                    height: 44px;
                    HorizontalLayout { spacing: 15px;
Rectangle {
                        width: 44px; height: 44px;
                        background: rgb(245, 246, 250);
                        border-radius: 22px;
                        Image {
                            source: root.language-code == "hr" ? @image-url("../assets/flags/hr.svg") :
                                root.language-code == "de" ? @image-url("../assets/flags/de.svg") :
                                root.language-code == "it" ? @image-url("../assets/flags/it.svg") :
                                root.language-code == "es" ? @image-url("../assets/flags/es.svg") : @image-url("../assets/flags/en.svg");
                            x: 8px; y: 13px; width: 28px; height: 18px; image-fit: contain;
                        }
                        language-trigger := AccessibleTap { width: parent.width; height: parent.height; refocus-request: root.return-language-focus; label: root.txt-language; activated => { root.language-menu = !root.language-menu; root.home-panel = 0; } }
                    }
Rectangle {
                        width: 44px; height: 44px;
                        background: rgb(245, 246, 250);
                        border-radius: 22px;
                        Image {
                            source: @image-url("../assets/icons/info.svg");
                            x: 10px; y: 10px;
                            width: 24px; height: 24px;
                            image-fit: contain;
                        }
                        info-trigger := AccessibleTap { width: parent.width; height: parent.height; refocus-request: root.return-info-focus; label: "Info"; activated => { root.home-panel = 1; root.language-menu = false; root.last-copy-address = ""; root.info-copy-feedback = ""; } }
                    }
Rectangle {
                        width: 44px; height: 44px;
                        background: rgb(245, 246, 250);
                        border-radius: 22px;
                        Image {
                            source: root.dark-mode ? @image-url("../assets/icons/moon.svg") : @image-url("../assets/icons/sun.svg");
                            x: 10px; y: 10px;
                            width: 24px; height: 24px;
                            image-fit: contain;
                        }
                        AccessibleTap { width: parent.width; height: parent.height; label: root.dark-mode ? root.txt-light-theme : root.txt-dark-theme; activated => { root.toggle-theme(); } }
                    }
Rectangle {
                        width: 44px; height: 44px;
                        background: rgb(245, 246, 250);
                        border-radius: 22px;
                        Image {
                            source: @image-url("../assets/icons/gear.svg");
                            x: 10px; y: 10px;
                            width: 24px; height: 24px;
                            image-fit: contain;
                        }
                        AccessibleTap { width: parent.width; height: parent.height; refocus-request: root.return-settings-focus; label: root.txt-settings; activated => { root.home-panel = root.home-panel == 2 ? 0 : 2; } }
                    }
                    }
                }

                // P2.6 original-ratio clock fit. Text metrics are measured by Slint
                // at 100px for the selected font. HH:MM and :SS use ONE font size.
                // Unlike the earlier fixed-font solution, 100% fills the usable area.
                // The sample reserves for the widest digits, keeping time updates stable.
                clock-fit-sample := Text {
                    x: -10000px;
                    y: -10000px;
                    text: root.hour-24 ?
                        (root.show-seconds ? "88:88:88" : "88:88") :
                        (root.show-seconds ? "88:88:88 PM" : "88:88 PM");
                    font-family: root.font-family-choice;
                    font-size: 100px;
                }
                Rectangle {
                    x: 20px + (root.display-mode == 1 ? root.saver-shift-x * 1px : 0px);
                    y: parent.height * 0.13 + (root.display-mode == 1 ? root.saver-shift-y * 1px : 0px);
                    width: parent.width - 40px;
                    height: parent.height * 0.55;
                    background: transparent;
                    Text {
                        width: parent.width;
                        height: parent.height;
                        text: root.clock-text;
                        font-family: root.font-family-choice;
                        // P2.6 clock scaling: new 90% equals the previous 100% size.
                        // Use the full clock text with equal-sized HH:MM[:SS] glyphs.
                        // At 100%, permit ~11% growth, but never exceed the measured
                        // width/height available for the selected font and screen size.
                        font-size: Math.min(
                            Math.min(
                                ((parent.width * (root.show-seconds ? 0.82 : 0.72)) /
                                    Math.max(clock-fit-sample.preferred-width, 1px)) * 100px,
                                ((root.height * 0.43 * 0.95) /
                                    Math.max(clock-fit-sample.preferred-height, 1px)) * 100px
                            ) * root.clock-scale / 90,
                            Math.min(
                                ((parent.width * 0.94) /
                                    Math.max(clock-fit-sample.preferred-width, 1px)) * 100px,
                                ((parent.height * 0.94) /
                                    Math.max(clock-fit-sample.preferred-height, 1px)) * 100px
                            )
                        );
                        color: root.display-mode == 2 ? root.bedside-clock-color : root.home-clock-color;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                        overflow: clip;
                    }
                }
                // Actual date metrics, measured for the current font and translated date.
                date-fit-sample := Text {
                    x: -10000px;
                    y: -10000px;
                    text: root.date-text;
                    font-family: root.font-family-choice;
                    font-size: 100px;
                }
                Text {
                    x: 20px + (root.display-mode == 1 ? root.saver-shift-x * 1px : 0px);
                    y: parent.height * 0.625 + (root.display-mode == 1 ? root.saver-shift-y * 1px : 0px);
                    width: parent.width - 40px;
                    height: parent.height * 0.19;
                    visible: root.show-date;
                    text: root.date-text;
                    font-family: root.font-family-choice;
                    // 100% can be ~20-25% larger than P2.6, but still fits long
                    // date strings and smaller windows without clipping.
                    font-size: Math.min(
                        (root.width > 1400px ? 140px : root.width > 1050px ? 102px : 72px) * root.date-scale / 100,
                        Math.min(
                            ((parent.width * 0.92) / Math.max(date-fit-sample.preferred-width, 1px)) * 100px,
                            ((parent.height * 0.92) / Math.max(date-fit-sample.preferred-height, 1px)) * 100px
                        )
                    );
                    color: root.display-mode == 2 ? root.bedside-clock-color : root.home-date-color;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }

                // Screensaver drift, emergency exit button and bedside dimmer.
                // TouchArea occupies the mode surface. Controls are painted above it.
                if root.display-mode != 0 : Rectangle {
                    width: parent.width; height: parent.height; background: transparent;
                    TouchArea { width: parent.width; height: parent.height;
                        clicked => { root.exit-display-mode(); }
                    }
                    Rectangle {
                        x: parent.width - 160px; y: 16px;
                        width: 142px; height: 43px; border-radius: 8px;
                        background: root.display-mode == 2 ? #202020 : #263663;
                        Button { width: parent.width; height: parent.height;
                            text: root.txt-exit-mode; clicked => { root.exit-display-mode(); }
                        }
                    }
                    if root.display-mode == 2 : Rectangle {
                        x: parent.width / 2 - 180px;
                        y: parent.height - 95px;
                        width: 360px; height: 76px;
                        border-radius: 10px;
                        background: #202020;
                        VerticalLayout { padding: 9px; spacing: 4px;
                            Text { text: root.txt-bedside-brightness + ": " + round(bedside-slider.value) + "%";
                                color: #eeeeee; horizontal-alignment: center; font-size: 14px; }
                            bedside-slider := Slider { minimum: 1; maximum: 100; step: 1;
                                value: root.bedside-brightness-pct * 1.0;
                                changed(value) => { root.preview-bedside(value); }
                                released => { root.set-bedside-brightness(bedside-slider.value); }
                            }
                        }
                    }
                }
                // Standard Apps & Games information / donations. Payment links and
                // wallet addresses are copied from the original Windows Portable web assets.
                if root.home-panel == 1 && root.display-mode == 0 : Rectangle {
                    x: Math.max(10px, (parent.width - self.width) / 2);
                    y: 20px;
                    width: Math.min(900px, parent.width - 20px);
                    height: parent.height - 40px;
                    background: #101a38;
                    border-radius: 13px;
                    VerticalLayout { padding: 16px; spacing: 10px;
                        HorizontalLayout { spacing: 8px;
                            Text { text: root.txt-about-title; color: #f2f6ff; font-size: 23px; font-weight: 700; horizontal-stretch: 1; }
                            Button { width: 90px; text: root.txt-close; init => { self.focus(); } clicked => { root.home-panel = 0; info-trigger.focus(); } }
                        }
                        ScrollView { vertical-stretch: 1;
                            VerticalLayout { spacing: 13px;
                                Rectangle { height: 330px; background: rgb(11, 19, 48); border-radius: 12px;
                                    VerticalLayout { padding: 14px; spacing: 9px;
                                        Text { text: "APPS & GAMES"; color: #52e5ff; font-size: 13px; font-weight: 700; }
                                        Text { text: root.txt-about-app-name; color: #ffffff; font-size: 21px; font-weight: 700; }
                                        Text { text: ""; color: #a0bddd; font-size: 12px; }
                                        Text { text: root.txt-about-intro; color: #cbd7ec; wrap: word-wrap; font-size: 14px; }
                                        Text { text: root.txt-about-features; color: #ffffff; font-size: 16px; font-weight: 700; }
                                        Text { text: root.txt-about-list; color: #cbd7ec; wrap: word-wrap; font-size: 14px; }
                                        Text { text: root.txt-about-portable; color: #9db2d4; wrap: word-wrap; font-size: 13px; }
                                        Button { width: 220px; text: root.txt-visit-apps;
                                            clicked => { Platform.open-url("https://appsandgames.org/"); }
                                        }
                                    }
                                }
                                Rectangle { height: 140px; background: rgb(11, 19, 48); border-radius: 12px;
                                    VerticalLayout { padding: 14px; spacing: 9px;
                                        Text { text: root.txt-about-donate; color: #ffffff; font-size: 19px; font-weight: 700; }
                                        Text { text: root.txt-about-donate-note; color: #cbd7ec; wrap: word-wrap; font-size: 15px; }
                                    }
                                }
                                Text { text: root.txt-donation-methods; color: #ffffff; font-size: 17px; font-weight: 700; }
                                HorizontalLayout { height: 222px; spacing: 12px;
                                    Rectangle { horizontal-stretch: 1; background: rgb(11, 22, 52); border-radius: 12px;
                                        VerticalLayout { padding: 13px; spacing: 11px;
                                            Text { text: "PayPal"; font-size: 18px; font-weight: 700; color: #f2f6ff; }
                                            Text { text: root.txt-paypal-description; font-size: 14px; wrap: word-wrap; color: #bbcee9; vertical-stretch: 1; }
                                            Text { text: root.txt-paypal-methods; font-size: 11px; color: #a1c0ee; }
                                            Button { text: root.txt-paypal-action;
                                                clicked => { Platform.open-url("https://www.paypal.com/ncp/payment/RU2CWCNVQ7XD6"); }
                                            }
                                        }
                                    }
                                    Rectangle { horizontal-stretch: 1; background: rgb(11, 22, 52); border-radius: 12px;
                                        VerticalLayout { padding: 13px; spacing: 11px;
                                            Text { text: "Stripe"; font-size: 18px; font-weight: 700; color: #f2f6ff; }
                                            Text { text: root.txt-stripe-description; font-size: 14px; wrap: word-wrap; color: #bbcee9; vertical-stretch: 1; }
                                            Text { text: root.txt-stripe-methods; font-size: 11px; color: #a1c0ee; }
                                            Button { text: root.txt-stripe-action;
                                                clicked => { Platform.open-url("https://buy.stripe.com/7sYeVd7Blfe89cm0k02kw00"); }
                                            }
                                        }
                                    }
                                }
                                Text { text: root.txt-payment-note; wrap: word-wrap; font-size: 12px; color: #a8bbd7; }
                                Rectangle { background: rgb(11, 19, 48); border-radius: 12px;
                                    VerticalLayout { padding: 13px; spacing: 10px;
                                        Text { text: root.txt-crypto-wallet; color: #52e5ff; font-weight: 700; font-size: 17px; }
                                        Text { text: root.txt-crypto-description; color: #bdcde9; wrap: word-wrap; font-size: 13px; }
                                        for coin in root.donation-rows : Rectangle {
                                            height: 66px;
                                            background: #152342;
                                            border-radius: 7px;
                                            HorizontalLayout { padding: 7px; spacing: 8px;
                                                Text { text: coin.coin; width: 56px; color: #54e5ff; font-size: 14px; font-weight: 700;
                                                    vertical-alignment: center; }
                                                Text { text: coin.address; horizontal-stretch: 1; color: #d9e6ff;
                                                    font-size: 11px; vertical-alignment: center; overflow: elide; }
                                                VerticalLayout { width: 162px; spacing: 2px;
                                                    Button { height: 31px; text: root.txt-copy;
                                                        clicked => { root.copy-donation-address(coin.address); }
                                                    }
                                                    Text {
                                                        height: 15px;
                                                        text: root.last-copy-address == coin.address ?
                                                            (root.last-copy-success ? root.txt-copied : root.txt-copy-failed) : "";
                                                        color: root.last-copy-success ? #57e3d3 : #ffacac;
                                                        font-size: 11px;
                                                        horizontal-alignment: center;
                                                        vertical-alignment: center;
                                                        overflow: elide;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Text { text: root.txt-about-limits + ": " + root.txt-about-limit-note;
                                    color: #9db2d4; wrap: word-wrap; font-size: 12px; }
                            }
                        }
                    }
                }
                // Transactional, locally stored JSON snapshots. Never reads WebView2/legacy data.
                if root.home-panel == 3 && root.display-mode == 0 : Rectangle {
                    x: parent.width - 520px; y: 84px; width: 500px; height: 434px;
                    background: #f7f9ff; border-radius: 12px;
                    VerticalLayout { padding: 16px; spacing: 12px;
                        HorizontalLayout {
                            Text { text: root.txt-backup; color: #142033; font-size: 19px; font-weight: 700; horizontal-stretch: 1; }
                            Button { text: root.txt-close; clicked => { root.home-panel = 0; } }
                        }
                        Text { text: root.txt-backup-folder; color: #405d6f; font-size: 14px; }
                        Button { width: 250px; text: root.txt-backup-export; clicked => { root.create-backup(); } }
                        Button { width: 250px; text: root.txt-backup-save-as; clicked => { root.save-backup-as(); } }
                        Text { text: root.txt-backup-select; color: #243b55; font-size: 14px; }
                        ComboBox { width: 420px; model: root.backup-files; current-value <=> root.selected-backup; }
                        Button { width: 250px; text: root.txt-backup-import; enabled: root.selected-backup != "";
                                 clicked => { root.restore-backup(root.selected-backup); } }
                        Button { width: 250px; text: root.txt-backup-open-file; clicked => { root.open-external-backup(); } }
                        Text { text: root.backup-status; color: #243b55; font-size: 14px; wrap: word-wrap; vertical-stretch: 1; }
                    }
                }

                // P2.6 - native scrollable settings, inspired by the original side panel.
                if root.home-panel == 2 && root.display-mode == 0 : Rectangle {
                    x: parent.width - 492px;
                    y: 84px;
                    width: 472px;
                    height: parent.height - 102px;
                    background: #f7f9ff;
                    border-radius: 12px;
                    VerticalLayout {
                        padding: 12px;
                        spacing: 7px;
                        HorizontalLayout {
                            height: 34px;
                            Text { text: root.txt-settings; font-size: 18px; font-weight: 700; color: #142033; horizontal-stretch: 1; }
                            Button { width: 90px; text: root.txt-backup; clicked => { root.home-panel = 3; root.language-menu = false; } }
                            Button { width: 76px; text: root.txt-close; clicked => { root.home-panel = 0; } }
                        }
                        ScrollView {
                            vertical-stretch: 1;
                            VerticalLayout {
                                padding: 8px;
                                spacing: 9px;
                                Text { text: root.txt-clock-date; font-size: 16px; font-weight: 700; color: #142033; }
                                HorizontalLayout { spacing: 6px;
                                    Button { width: 126px; text: root.hour-24 ? root.txt-24-hours : root.txt-12-hours; clicked => { root.toggle-hour-mode(); } }
                                    Button { width: 126px; text: root.txt-seconds + ": " + (root.show-seconds ? root.txt-yes : root.txt-no); clicked => { root.toggle-seconds(); } }
                                    Button { width: 126px; text: root.txt-date + ": " + (root.show-date ? root.txt-yes : root.txt-no); clicked => { root.toggle-date(); } }
                                }
                                Text { text: root.txt-format-date; color: #243b55; }
                                ComboBox { width: 375px; model: root.date-format-choices;
                                           current-value <=> root.date-format-choice;
                                           selected(value) => { root.set-date-format(value); } }
                                Text { text: root.txt-font; color: #243b55; }
                                ComboBox { width: 375px; model: root.font-choices;
                                           current-value <=> root.font-family-choice;
                                           selected(value) => { root.set-font-family(value); } }
                                Text { text: root.txt-color-hint; font-size: 15px; font-weight: 700; color: #142033; }
                                Text { text: root.txt-background + ": " + root.bg-hex; color: #243b55; font-size: 14px; }
                                HorizontalLayout { spacing: 7px; height: 34px;
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #2400d0;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-background + " #2400D0"; activated => { root.choose-color("background", "#2400D0"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #daff00;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-background + " #DAFF00"; activated => { root.choose-color("background", "#DAFF00"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #000000;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-background + " #000000"; activated => { root.choose-color("background", "#000000"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #ffffff;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-background + " #FFFFFF"; activated => { root.choose-color("background", "#FFFFFF"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #ff3366;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-background + " #FF3366"; activated => { root.choose-color("background", "#FF3366"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #08c4be;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-background + " #08C4BE"; activated => { root.choose-color("background", "#08C4BE"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #3273ff;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-background + " #3273FF"; activated => { root.choose-color("background", "#3273FF"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #ff9f1c;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-background + " #FF9F1C"; activated => { root.choose-color("background", "#FF9F1C"); } }
                                    }
                                }
                                Text { text: root.txt-clock + ": " + root.time-hex; color: #243b55; font-size: 14px; }
                                HorizontalLayout { spacing: 7px; height: 34px;
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #2400d0;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-clock + " #2400D0"; activated => { root.choose-color("clock", "#2400D0"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #daff00;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-clock + " #DAFF00"; activated => { root.choose-color("clock", "#DAFF00"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #000000;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-clock + " #000000"; activated => { root.choose-color("clock", "#000000"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #ffffff;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-clock + " #FFFFFF"; activated => { root.choose-color("clock", "#FFFFFF"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #ff3366;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-clock + " #FF3366"; activated => { root.choose-color("clock", "#FF3366"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #08c4be;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-clock + " #08C4BE"; activated => { root.choose-color("clock", "#08C4BE"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #3273ff;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-clock + " #3273FF"; activated => { root.choose-color("clock", "#3273FF"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #ff9f1c;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-clock + " #FF9F1C"; activated => { root.choose-color("clock", "#FF9F1C"); } }
                                    }
                                }
                                Text { text: root.txt-date + ": " + root.date-hex; color: #243b55; font-size: 14px; }
                                HorizontalLayout { spacing: 7px; height: 34px;
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #2400d0;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-date + " #2400D0"; activated => { root.choose-color("date", "#2400D0"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #daff00;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-date + " #DAFF00"; activated => { root.choose-color("date", "#DAFF00"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #000000;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-date + " #000000"; activated => { root.choose-color("date", "#000000"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #ffffff;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-date + " #FFFFFF"; activated => { root.choose-color("date", "#FFFFFF"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #ff3366;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-date + " #FF3366"; activated => { root.choose-color("date", "#FF3366"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #08c4be;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-date + " #08C4BE"; activated => { root.choose-color("date", "#08C4BE"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #3273ff;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-date + " #3273FF"; activated => { root.choose-color("date", "#3273FF"); } }
                                    }
                                    Rectangle { width: 31px; height: 31px; border-radius: 5px; border-color: #77849a; border-width: 1px; background: #ff9f1c;
                                        AccessibleTap { width: parent.width; height: parent.height; label: root.txt-date + " #FF9F1C"; activated => { root.choose-color("date", "#FF9F1C"); } }
                                    }
                                }
                                Button { width: 210px; text: root.advanced-colors-visible ? root.txt-hide-hex : root.txt-custom-hex;
                                    clicked => { root.advanced-colors-visible = !root.advanced-colors-visible; } }
                                if root.advanced-colors-visible : VerticalLayout { spacing: 6px;
                                    Text { text: root.txt-hex-hint; color: #243b55; }
                                    HorizontalLayout { spacing: 7px;
                                        Text { text: root.txt-background; width: 90px; color: #243b55; }
                                        LineEdit { width: 132px; text <=> root.bg-hex; }
                                    }
                                    HorizontalLayout { spacing: 7px;
                                        Text { text: root.txt-clock; width: 90px; color: #243b55; }
                                        LineEdit { width: 132px; text <=> root.time-hex; }
                                    }
                                    HorizontalLayout { spacing: 7px;
                                        Text { text: root.txt-date; width: 90px; color: #243b55; }
                                        LineEdit { width: 132px; text <=> root.date-hex; }
                                    }
                                    Button { width: 160px; text: root.txt-apply-hex;
                                        clicked => { root.apply-colors(root.bg-hex, root.time-hex, root.date-hex); } }
                                }
                                HorizontalLayout { width: 350px; height: 23px; spacing: 0px;
                                    Text { text: root.txt-clock-size; color: #243b55; font-size: 13px; }
                                    Rectangle { horizontal-stretch: 1; }
                                    Rectangle { width: 56px; height: 23px; border-radius: 5px; background: #dceafd;
                                        Text { width: parent.width; height: parent.height; text: round(clock-size-slider.value) + "%";
                                               color: #124b9c; font-size: 13px; font-weight: 700;
                                               horizontal-alignment: center; vertical-alignment: center; }
                                    }
                                }
                                clock-size-slider := Slider { width: 350px; minimum: 50; maximum: 100; step: 5;
                                         value: root.clock-scale; changed(v) => { root.clock-scale = round(v); } released(v) => { root.set-clock-scale(v); } }
                                Rectangle { width: 350px; height: 16px; background: transparent;
                                    Text { x: 0px; y: 0px; width: 34px; height: 16px; text: "50%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 63px; y: 0px; width: 34px; height: 16px; text: "60%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 126px; y: 0px; width: 34px; height: 16px; text: "70%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 190px; y: 0px; width: 34px; height: 16px; text: "80%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 253px; y: 0px; width: 34px; height: 16px; text: "90%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 316px; y: 0px; width: 34px; height: 16px; text: "100%"; font-size: 11px; color: #52677d; horizontal-alignment: right; }
                                }
                                HorizontalLayout { width: 350px; height: 23px; spacing: 0px;
                                    Text { text: root.txt-date-size; color: #243b55; font-size: 13px; }
                                    Rectangle { horizontal-stretch: 1; }
                                    Rectangle { width: 56px; height: 23px; border-radius: 5px; background: #dceafd;
                                        Text { width: parent.width; height: parent.height; text: round(date-size-slider.value) + "%";
                                               color: #124b9c; font-size: 13px; font-weight: 700;
                                               horizontal-alignment: center; vertical-alignment: center; }
                                    }
                                }
                                date-size-slider := Slider { width: 350px; minimum: 40; maximum: 100; step: 5;
                                         value: root.date-scale; changed(v) => { root.date-scale = round(v); } released(v) => { root.set-date-scale(v); } }
                                Rectangle { width: 350px; height: 16px; background: transparent;
                                    Text { x: 0px; y: 0px; width: 34px; height: 16px; text: "40%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 53px; y: 0px; width: 34px; height: 16px; text: "50%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 105px; y: 0px; width: 34px; height: 16px; text: "60%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 158px; y: 0px; width: 34px; height: 16px; text: "70%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 211px; y: 0px; width: 34px; height: 16px; text: "80%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 263px; y: 0px; width: 34px; height: 16px; text: "90%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 316px; y: 0px; width: 34px; height: 16px; text: "100%"; font-size: 11px; color: #52677d; horizontal-alignment: right; }
                                }
                                HorizontalLayout { width: 350px; height: 23px; spacing: 0px;
                                    Text { text: root.txt-brightness; color: #243b55; font-size: 13px; }
                                    Rectangle { horizontal-stretch: 1; }
                                    Rectangle { width: 56px; height: 23px; border-radius: 5px; background: #dceafd;
                                        Text { width: parent.width; height: parent.height; text: round(brightness-slider.value) + "%";
                                               color: #124b9c; font-size: 13px; font-weight: 700;
                                               horizontal-alignment: center; vertical-alignment: center; }
                                    }
                                }
                                brightness-slider := Slider { width: 350px; minimum: 0; maximum: 100; step: 5;
                                         value: root.brightness-pct; changed(v) => { root.brightness-pct = round(v); root.preview-lighting(root.brightness-pct, root.contrast-pct); } released(v) => { root.set-brightness(v); } }
                                Rectangle { width: 350px; height: 16px; background: transparent;
                                    Text { x: 0px; y: 0px; width: 34px; height: 16px; text: "0%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 63px; y: 0px; width: 34px; height: 16px; text: "20%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 126px; y: 0px; width: 34px; height: 16px; text: "40%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 190px; y: 0px; width: 34px; height: 16px; text: "60%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 253px; y: 0px; width: 34px; height: 16px; text: "80%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 316px; y: 0px; width: 34px; height: 16px; text: "100%"; font-size: 11px; color: #52677d; horizontal-alignment: right; }
                                }
                                HorizontalLayout { width: 350px; height: 23px; spacing: 0px;
                                    Text { text: root.txt-contrast; color: #243b55; font-size: 13px; }
                                    Rectangle { horizontal-stretch: 1; }
                                    Rectangle { width: 56px; height: 23px; border-radius: 5px; background: #dceafd;
                                        Text { width: parent.width; height: parent.height; text: round(contrast-slider.value) + "%";
                                               color: #124b9c; font-size: 13px; font-weight: 700;
                                               horizontal-alignment: center; vertical-alignment: center; }
                                    }
                                }
                                contrast-slider := Slider { width: 350px; minimum: 0; maximum: 100; step: 5;
                                         value: root.contrast-pct; changed(v) => { root.contrast-pct = round(v); root.preview-lighting(root.brightness-pct, root.contrast-pct); } released(v) => { root.set-contrast(v); } }
                                Rectangle { width: 350px; height: 16px; background: transparent;
                                    Text { x: 0px; y: 0px; width: 34px; height: 16px; text: "0%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 63px; y: 0px; width: 34px; height: 16px; text: "20%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 126px; y: 0px; width: 34px; height: 16px; text: "40%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 190px; y: 0px; width: 34px; height: 16px; text: "60%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 253px; y: 0px; width: 34px; height: 16px; text: "80%"; font-size: 11px; color: #52677d; horizontal-alignment: left; }
                                    Text { x: 316px; y: 0px; width: 34px; height: 16px; text: "100%"; font-size: 11px; color: #52677d; horizontal-alignment: right; }
                                }
                                Text { text: root.txt-profiles; font-size: 16px; font-weight: 700; color: #142033; }
                                profile-entry := LineEdit { width: 350px; placeholder-text: root.txt-profile-placeholder;
                                    text <=> root.profile-name-input; }
                                Button { width: 185px; text: root.txt-save-profile;
                                    clicked => { root.save-profile(profile-entry.text); } }
                                Text { text: root.profile-feedback; wrap: word-wrap; color: #243b55; font-size: 12px; }
                                ComboBox { width: 350px; model: root.profile-names;
                                           current-value <=> root.selected-profile; }
                                HorizontalLayout { spacing: 8px;
                                    Button { width: 145px; text: root.txt-load-profile;
                                             clicked => { root.load-profile(root.selected-profile); } }
                                    Button { width: 145px; text: root.txt-delete-profile;
                                             clicked => { root.remove-profile(root.selected-profile); } }
                                }
                                Button { width: 170px; text: root.txt-restore-default;
                                         clicked => { root.reset-appearance(); } }
                                Text { text: root.txt-limit-short;
                                       wrap: word-wrap; font-size: 12px; color: #486b80; }
                            }
                        }
                    }
                }
                // RC2.1: Language overlay must be painted AFTER clock/date/panels.
                // P2.8 language menu (own SVG flag assets). No system font emoji dependency.
                if root.language-menu && root.display-mode == 0 : Rectangle {
                    x: parent.width - 268px; y: 78px; width: 244px; height: 276px;
                    background: #f7f9ff; border-radius: 10px;
                    VerticalLayout {
                        padding: 10px; spacing: 6px;
                        Text { text: root.txt-language; font-size: 16px; color: #142033; font-weight: 700; }
                        HorizontalLayout { height: 37px; spacing: 9px;
                            Image { source: @image-url("../assets/flags/en.svg"); width: 29px; height: 20px; image-fit: contain; }
                            Button { text: "English"; init => { self.focus(); } clicked => { root.select-language("en"); root.language-menu = false; language-trigger.focus(); } }
                        }
                        HorizontalLayout { height: 37px; spacing: 9px;
                            Image { source: @image-url("../assets/flags/hr.svg"); width: 29px; height: 20px; image-fit: contain; }
                            Button { text: "Hrvatski"; clicked => { root.select-language("hr"); root.language-menu = false; language-trigger.focus(); } }
                        }
                        HorizontalLayout { height: 37px; spacing: 9px;
                            Image { source: @image-url("../assets/flags/de.svg"); width: 29px; height: 20px; image-fit: contain; }
                            Button { text: "Deutsch"; clicked => { root.select-language("de"); root.language-menu = false; language-trigger.focus(); } }
                        }
                        HorizontalLayout { height: 37px; spacing: 9px;
                            Image { source: @image-url("../assets/flags/it.svg"); width: 29px; height: 20px; image-fit: contain; }
                            Button { text: "Italiano"; clicked => { root.select-language("it"); root.language-menu = false; language-trigger.focus(); } }
                        }
                        HorizontalLayout { height: 37px; spacing: 9px;
                            Image { source: @image-url("../assets/flags/es.svg"); width: 29px; height: 20px; image-fit: contain; }
                            Button { text: "Español"; clicked => { root.select-language("es"); root.language-menu = false; language-trigger.focus(); } }
                        }
                    }
                }

                Text {
                    text: root.txt-dev-build;
                    x: parent.width - 172px; y: parent.height - 24px;
                    width: 155px; height: 14px;
                    font-size: 10px;
                    color: root.dark-mode ? rgb(157, 130, 255) : #8e91a4;
                    horizontal-alignment: right;
                }
            }

            if root.selected-page != 0 : VerticalLayout {
                vertical-stretch: 1;
                spacing: 12px;
            Rectangle {
                height: 62px;
                background: root.tool-panel;
                border-radius: 14px;
                border-color: root.tool-accent;
                border-width: 1px;
                HorizontalLayout {
                    padding: 16px;
                    spacing: 10px;
                    Text {
                        text: "DIGITAL CLOCK";
                        font-size: 23px;
                        font-weight: 700;
                        color: root.tool-text;
                        vertical-alignment: center;
                        horizontal-stretch: 1;
                    }
                    Text {
                        text: "";
                        font-size: 13px;
                        color: root.dark-mode ? #8cabc4 : #577080;
                        vertical-alignment: center;
                    }
                    Button {
                        text: root.fullscreen-mode ? root.txt-windowed : root.txt-fullscreen;
                        clicked => { root.fullscreen-mode = !root.fullscreen-mode; }
                    }
                }
            }

            Rectangle {
                height: root.selected-page == 0 ? 270px : 150px;
                background: root.tool-panel;
                border-radius: 18px;
                border-color: root.tool-accent;
                border-width: root.selected-page == 0 ? 0px : 1px;
                VerticalLayout {
                    padding: 12px;
                    spacing: 5px;
                    Rectangle { vertical-stretch: 1; background: transparent; }
                    Text {
                        text: root.clock-text;
                        font-family: "Consolas";
                        font-size: root.selected-page == 0 ?
                            (root.show-seconds ? 104px : 135px) :
                            (root.show-seconds ? 58px : 74px);
                        font-weight: 700;
                        horizontal-alignment: center;
                        color: root.tool-display-text;
                        height: root.selected-page == 0 ? 140px : 76px;
                    }
                    Text {
                        text: root.date-text;
                        visible: root.show-date;
                        font-size: root.selected-page == 0 ? 31px : 19px;
                        color: root.tool-muted;
                        horizontal-alignment: center;
                        height: root.show-date ? (root.selected-page == 0 ? 39px : 27px) : 0px;
                    }
                    Rectangle { vertical-stretch: 1; background: transparent; }
                }
            }

            Rectangle {
                height: 58px;
                background: root.tool-panel;
                border-radius: 13px;
                border-color: root.tool-accent;
                border-width: 1px;
                HorizontalLayout {
                    padding: 9px;
                    spacing: 10px;
                    Button { text: root.selected-page == 0 ? "● " + root.txt-clock : root.txt-clock; clicked => { root.selected-page = 0; } }
                    Button { text: root.selected-page == 1 ? "● " + root.txt-alarm : root.txt-alarm; clicked => { root.selected-page = 1; } }
                    Button { text: root.selected-page == 2 ? "● " + root.txt-timer : root.txt-timer; clicked => { root.selected-page = 2; } }
                    Button { text: root.selected-page == 3 ? "● " + root.txt-stopwatch : root.txt-stopwatch; clicked => { root.selected-page = 3; } }
                    Button { text: root.selected-page == 4 ? "● " + root.txt-world : root.txt-world; clicked => { root.selected-page = 4; } }
                }
            }

            Rectangle {
                vertical-stretch: 1;
                background: root.tool-panel;
                border-radius: 15px;
                border-color: root.tool-accent;
                border-width: 1px;
                VerticalLayout {
                    padding: 18px;
                    spacing: 8px;

                    if root.selected-page == 0 : Rectangle {
                        vertical-stretch: 1;
                        background: transparent;
                        VerticalLayout {
                            spacing: 14px;
                            Text { text: root.txt-clock-view; font-size: 18px; font-weight: 700; color: root.tool-text; }
                            HorizontalLayout {
                                spacing: 10px;
                                Button { width: 142px; text: root.hour-24 ? "Format 24 h" : "Format 12 h"; clicked => { root.toggle-hour-mode(); } }
                                Button { width: 142px; text: root.txt-seconds + ": " + (root.show-seconds ? root.txt-yes : root.txt-no); clicked => { root.toggle-seconds(); } }
                                Button { width: 142px; text: root.txt-date + ": " + (root.show-date ? root.txt-yes : root.txt-no); clicked => { root.toggle-date(); } }
                                Button { width: 142px; text: root.dark-mode ? root.txt-light-theme : root.txt-dark-theme; clicked => { root.toggle-theme(); } }
                                Rectangle { horizontal-stretch: 1; background: transparent; }
                            }
                            Text {
                                text: root.txt-portable-hint;
                                font-size: 15px;
                                color: root.tool-muted;
                            }
                            Rectangle { vertical-stretch: 1; background: transparent; }
                        }
                    }

                    if root.selected-page == 1 : Rectangle {
                        vertical-stretch: 1;
                        background: transparent;
                        VerticalLayout {
                            spacing: 9px;
                            Text { text: root.txt-alarm + " · 24 h"; font-size: 18px; font-weight: 700; color: root.tool-accent; }
                            // P2.3: HorizontalLayout + fixed item widths + stretchable trailing spacer.
                            // This prevents controls from splitting the entire window width.
                            Rectangle {
                                height: 42px;
                                background: transparent;
                                HorizontalLayout {
                                    spacing: 8px;
                                    Text { text: root.txt-new + ":"; width: 56px; height: 40px;
                                           color: root.tool-muted;
                                           vertical-alignment: center; }
                                    ComboBox { width: 90px; height: 40px;
                                               model: root.alarm-hours; current-value <=> root.add-alarm-hour; }
                                    Text { text: ":"; width: 12px; height: 40px;
                                           font-size: 23px; color: root.tool-muted;
                                           horizontal-alignment: center; vertical-alignment: center; }
                                    ComboBox { width: 90px; height: 40px;
                                               model: root.alarm-minutes; current-value <=> root.add-alarm-minute; }
                                    Button { width: 116px; height: 40px; text: root.txt-add + " " + root.txt-alarm;
                                             clicked => { root.add-alarm(root.add-alarm-hour, root.add-alarm-minute); } }
                                    Rectangle { horizontal-stretch: 1; background: transparent; }
                                }
                            }
                            if root.selected-alarm-id > 0 : Rectangle {
                                height: 42px;
                                background: transparent;
                                HorizontalLayout {
                                    spacing: 8px;
                                    Text { text: root.txt-edit + " #" + root.selected-alarm-id; width: 90px; height: 40px;
                                           color: root.tool-muted;
                                           vertical-alignment: center; }
                                    ComboBox { width: 90px; height: 40px;
                                               model: root.alarm-hours; current-value <=> root.edit-alarm-hour; }
                                    Text { text: ":"; width: 12px; height: 40px;
                                           font-size: 23px; color: root.tool-muted;
                                           horizontal-alignment: center; vertical-alignment: center; }
                                    ComboBox { width: 90px; height: 40px;
                                               model: root.alarm-minutes; current-value <=> root.edit-alarm-minute; }
                                    Button { width: 135px; height: 40px; text: root.txt-save-time;
                                             clicked => { root.update-alarm(root.selected-alarm-id, root.edit-alarm-hour, root.edit-alarm-minute); } }
                                    Rectangle { horizontal-stretch: 1; background: transparent; }
                                }
                            }
                            Rectangle {
                                height: 42px;
                                background: transparent;
                                HorizontalLayout {
                                    spacing: 8px;
                                    Text { text: root.txt-tone + ":"; width: 90px; height: 40px;
                                           color: root.tool-muted;
                                           vertical-alignment: center; }
                                    tone-choice := ComboBox { width: 160px; height: 40px;
                                        model: ["Chime", "Bell", "Pulse"];
                                        current-value <=> root.alarm-tone;
                                        selected(value) => { root.select-alarm-tone(value); }
                                    }
                                    Button { width: 140px; height: 40px;
                                             text: root.txt-preview; clicked => { root.preview-alarm-tone(); } }
                                    Rectangle { horizontal-stretch: 1; background: transparent; }
                                }
                            }
                            if root.installed-audio-enabled : HorizontalLayout {
                                height: 36px; spacing: 8px;
                                Text { width: 125px; color: root.tool-muted; text: root.txt-custom-audio; }
                                Button { width: 128px; text: root.txt-choose-sound; clicked => { root.choose-alarm-audio(); } }
                                Button { width: 130px; text: root.txt-reset-sound; clicked => { root.clear-alarm-audio(); } }
                                Text { color: root.tool-muted; text: root.custom-alarm-label; horizontal-stretch: 1; }
                            }
                            Text { text: root.txt-alarm-help;
                                   font-size: 13px; color: root.tool-muted; }
                            ScrollView {
                                vertical-stretch: 1;
                                VerticalLayout {
                                    spacing: 5px;
                                    for alarm in root.alarm-rows : Rectangle {
                                        height: 65px;
                                        background: root.tool-card;
                                        border-radius: 7px;
                                        border-color: root.tool-accent;
                                        border-width: 1px;
                                        HorizontalLayout {
                                            padding: 4px;
                                            spacing: 8px;
                                            Text { text: "#" + alarm.id + "  " + alarm.time;
                                                   color: root.tool-accent;
                                                   font-family: "Consolas"; font-size: 28px;
                                                   vertical-alignment: center;
                                                   horizontal-stretch: 1; }
                                            Button { width: 74px; text: root.txt-edit; clicked => { root.select-alarm(alarm.id); } }
                                            Button { width: 88px; text: alarm.enabled ? root.txt-disable : root.txt-enable;
                                                     clicked => { root.toggle-alarm(alarm.id); } }
                                            Button { width: 74px; text: root.txt-remove; clicked => { root.remove-alarm(alarm.id); } }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if root.selected-page == 2 : Rectangle {
                        vertical-stretch: 1;
                        background: transparent;
                        VerticalLayout {
                            spacing: 9px;
                            Text { text: root.txt-timer; height: 28px; vertical-stretch: 0; vertical-alignment: center; font-size: 18px; font-weight: 700; color: root.tool-accent; }
                            // Protected timer time: individual bounded selectors (no editable text).
                            HorizontalLayout { height: 72px; vertical-stretch: 0; spacing: 10px;
                                VerticalLayout { width: 90px; spacing: 3px;
                                    Text { text: root.txt-timer-hours; color: root.tool-text; font-size: 12px; }
                                    ComboBox { width: 90px; model: root.timer-hours; current-value <=> root.timer-hour; }
                                }
                                VerticalLayout { width: 90px; spacing: 3px;
                                    Text { text: root.txt-timer-minutes; color: root.tool-text; font-size: 12px; }
                                    ComboBox { width: 90px; model: root.timer-minutes; current-value <=> root.timer-minute; }
                                }
                                VerticalLayout { width: 90px; spacing: 3px;
                                    Text { text: root.txt-timer-seconds; color: root.tool-text; font-size: 12px; }
                                    ComboBox { width: 90px; model: root.timer-minutes; current-value <=> root.timer-second; }
                                }
                                Button { width: 84px; text: root.txt-set;
                                    clicked => { root.set-timer(root.timer-hour + ":" + root.timer-minute + ":" + root.timer-second); }
                                }
                                Button { width: 100px; text: root.countdown-button; clicked => { root.start-pause-timer(); } }
                                Button { width: 84px; text: root.txt-reset; clicked => { root.reset-timer(); } }
                                Rectangle { horizontal-stretch: 1; background: transparent; }
                            }
                            HorizontalLayout { spacing: 10px; height: 32px; vertical-stretch: 0;
                                Text { text: root.txt-tool-size + ": " + round(timer-size-slider.value) + "%";
                                    color: root.tool-text; width: 164px; }
                                timer-size-slider := Slider { width: 250px; minimum: 60; maximum: 150; step: 5;
                                    value: root.tool-scale * 1.0;
                                    released => { root.set-tool-scale(timer-size-slider.value); }
                                }
                                Rectangle { horizontal-stretch: 1; background: transparent; }
                            }
                            if root.installed-audio-enabled : HorizontalLayout {
                                height: 36px; spacing: 8px;
                                Text { width: 125px; color: root.tool-muted; text: root.txt-custom-audio; }
                                Button { width: 128px; text: root.txt-choose-sound; clicked => { root.choose-timer-audio(); } }
                                Button { width: 130px; text: root.txt-reset-sound; clicked => { root.clear-timer-audio(); } }
                                Text { color: root.tool-muted; text: root.custom-timer-label; horizontal-stretch: 1; }
                            }
                            // Keep time controls at the top in fullscreen: the remaining free
                            // height belongs below the digits, not above the timer inputs.
                            // Fixed display geometry also prevents font/layout binding loops.
                            Rectangle {
                                height: 250px;
                                vertical-stretch: 0;
                                background: root.tool-display;
                                border-radius: 12px;
                                border-color: root.tool-accent;
                                border-width: 1px;
                                Text {
                                    x: 0px; y: 0px;
                                    width: parent.width;
                                    height: parent.height;
                                    text: root.countdown-text;
                                    font-family: "Consolas";
                                    font-size: Math.min(250px,
                                        Math.min(parent.width * 0.18, parent.height * 0.9) * timer-size-slider.value / 100);
                                    horizontal-alignment: center;
                                    vertical-alignment: center;
                                    color: root.tool-display-text;
                                    overflow: clip;
                                }
                            }
                            Rectangle { vertical-stretch: 1; background: transparent; }
                        }
                    }

                    if root.selected-page == 3 : Rectangle {
                        vertical-stretch: 1;
                        background: transparent;
                        VerticalLayout {
                            spacing: 9px;
                            Text { text: root.txt-stopwatch; font-size: 18px; font-weight: 700; color: root.tool-accent; }
                            HorizontalLayout {
                                spacing: 10px;
                                Button { width: 110px; text: root.stopwatch-button; clicked => { root.start-pause-stopwatch(); } }
                                Button { width: 110px; text: root.txt-lap; clicked => { root.lap-stopwatch(); } }
                                Button { width: 110px; text: root.txt-reset; clicked => { root.reset-stopwatch(); } }
                                Rectangle { horizontal-stretch: 1; background: transparent; }
                            }
                            HorizontalLayout { spacing: 10px; height: 32px;
                                Text { text: root.txt-tool-size + ": " + round(stopwatch-size-slider.value) + "%";
                                    color: root.tool-text; width: 164px; }
                                stopwatch-size-slider := Slider { width: 250px; minimum: 60; maximum: 150; step: 5;
                                    value: root.tool-scale * 1.0;
                                    released => { root.set-tool-scale(stopwatch-size-slider.value); }
                                }
                                Rectangle { horizontal-stretch: 1; background: transparent; }
                            }
                            Rectangle {
                                height: 205px;
                                background: root.tool-display;
                                border-radius: 12px;
                                border-color: root.tool-accent;
                                border-width: 1px;
                                Text {
                                    x: 0px; y: 0px;
                                    width: parent.width;
                                    height: parent.height;
                                    text: root.stopwatch-text;
                                    font-family: "Consolas";
                                    font-size: Math.min(190px,
                                        Math.min(parent.width * 0.120, parent.height * 0.86) * stopwatch-size-slider.value / 100);
                                    color: root.tool-display-text;
                                    horizontal-alignment: center;
                                    vertical-alignment: center;
                                    overflow: clip;
                                }
                            }
                            Text { text: root.laps-text; font-family: "Consolas"; font-size: 15px;
                                   color: root.tool-muted; vertical-stretch: 1; }
                        }
                    }

                    if root.selected-page == 4 : Rectangle {
                        vertical-stretch: 1;
                        background: transparent;
                        VerticalLayout {
                            spacing: 8px;
                            Text { text: root.txt-world + " · IANA / DST";
                                   font-size: 18px; font-weight: 700; color: root.tool-accent; }
                            HorizontalLayout {
                                height: 42px;
                                spacing: 10px;
                                ComboBox {
                                    width: 365px;
                                    model: root.city-options;
                                    current-value <=> root.selected-city;
                                }
                                Button { width: 116px; text: root.txt-add-city; clicked => { root.add-world-city(); } }
                                Rectangle { horizontal-stretch: 1; background: transparent; }
                            }
                            Text { text: root.txt-world-help;
                                   font-size: 13px; color: root.tool-muted; }
                            ScrollView {
                                vertical-stretch: 1;
                                VerticalLayout {
                                    spacing: 5px;
                                    for city in root.city-rows : Rectangle {
                                        height: 90px;
                                        background: root.tool-card;
                                        border-radius: 7px;
                                        border-color: root.tool-accent;
                                        border-width: 1px;
                                        HorizontalLayout {
                                            padding: 5px;
                                            spacing: 8px;
                                            VerticalLayout {
                                                width: 600px;
                                                spacing: 1px;
                                                Text { text: city.name; color: root.tool-text;
                                                       font-size: 20px; font-weight: 600; }
                                                Text { text: city.local-time; color: root.tool-accent;
                                                       font-size: 29px; font-family: "Consolas"; }
                                            }
                                            Button { width: 90px; text: root.txt-remove; clicked => { root.remove-world-city(city.zone); } }
                                            Rectangle { horizontal-stretch: 1; background: transparent; }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            }

            Rectangle {
                height: root.alert-active ? 90px : 0px;
                visible: root.alert-active;
                background: #842f37;
                border-radius: 10px;
                HorizontalLayout {
                    padding: 8px;
                    spacing: 9px;
                    Text { text: root.alert-text; font-size: 30px; font-weight: 700; color: #ffffff;
                           vertical-alignment: center; horizontal-stretch: 1; }
                    Button { text: root.txt-confirm; clicked => { root.dismiss-alert(); } }
                }
            }

            Text {
                text: root.status-text;
                visible: root.selected-page != 0 || root.storage-warning;
                color: root.storage-warning ? #ff6262 : (root.dark-mode ? #9db9ce : #4d6b78);
                font-size: 13px;
                horizontal-alignment: center;
                wrap: word-wrap;
                height: root.selected-page == 0 && !root.storage-warning ? 0px : 42px;
            }
            Text {
                visible: root.selected-page != 0;
                text: root.txt-dev-build;
                color: root.dark-mode ? #8295aa : #577080;
                font-size: 12px;
                horizontal-alignment: center;
                height: root.selected-page == 0 ? 0px : 20px;
            }
            }
        }
    }
}

struct RuntimeState {
    stored: P2State,
    alarms: AlarmBook,
    countdown: Countdown,
    stopwatch: Stopwatch,
    alert: Option<String>,
    last_world_refresh_second: i64,
}

impl RuntimeState {
    fn new(stored: P2State) -> Result<Self, String> {
        let alarms = AlarmBook::from_alarms(stored.alarms.clone())?;
        Ok(Self { stored, alarms, countdown: Countdown::default(), stopwatch: Stopwatch::default(), alert: None, last_world_refresh_second: i64::MIN })
    }
}

// Separate fragments so adding seconds never changes the HH:MM font size.
// Includes 12h AM/PM in the smaller suffix, whether seconds are shown or not.
fn clock_face_parts(now: &DateTime<Local>, settings: &ClockSettings) -> (String, String) {
    let main = now.format(if settings.hour_24 { "%H:%M" } else { "%I:%M" }).to_string();
    let suffix = match (settings.hour_24, settings.show_seconds) {
        (true, true) => now.format(":%S").to_string(),
        (true, false) => String::new(),
        (false, true) => now.format(":%S %p").to_string(),
        (false, false) => now.format(" %p").to_string(),
    };
    (main, suffix)
}

fn format_time(now: DateTime<Local>, settings: &ClockSettings) -> String {
    match (settings.hour_24, settings.show_seconds) {
        (true, true) => now.format("%H:%M:%S").to_string(),
        (true, false) => now.format("%H:%M").to_string(),
        (false, true) => now.format("%I:%M:%S %p").to_string(),
        (false, false) => now.format("%I:%M %p").to_string(),
    }
}

fn format_date(now: DateTime<Local>, settings: &ClockSettings) -> String {
    let day = now.format("%d").to_string();
    let month = now.format("%m").to_string();
    let year = now.format("%Y").to_string();
    let full_day = i18n::weekday(&settings.language, now.weekday().num_days_from_monday() as usize);
    let short_day = i18n::weekday_short(&settings.language, now.weekday().num_days_from_monday() as usize);
    let month_name = i18n::month(&settings.language, now.month0() as usize);
    match settings.date_format.as_str() {
        "dd.mm.yyyy." => format!("{day}.{month}.{year}."),
        "mm.dd.yyyy." => format!("{month}.{day}.{year}."),
        "dd.mmm.yyyy." => format!("{}. {month_name} {year}.", now.day()),
        "ddd dd.mm.yyyy." => format!("{short_day} {day}.{month}.{year}."),
        _ => format!("{full_day} {}. {month_name} {year}.", now.day()),
    }
}

fn adjusted_color(hex: &str, brightness: u8, contrast: u8) -> Color {
    let (r, g, b) = config::rgb_from_hex(hex).unwrap_or((255, 255, 255));
    let convert = |channel: u8| -> u8 {
        let brightened = f64::from(channel) * f64::from(brightness) / 100.0;
        let centered = (brightened - 127.5) * f64::from(contrast) / 100.0 + 127.5;
        centered.round().clamp(0.0, 255.0) as u8
    };
    Color::from_rgb_u8(convert(r), convert(g), convert(b))
}

fn active_colors(settings: &ClockSettings) -> (&str, &str, &str) {
    if settings.dark_mode {
        (&settings.background_color, &settings.clock_color, &settings.date_color)
    } else {
        (&settings.light_background_color, &settings.light_clock_color, &settings.light_date_color)
    }
}

fn set_active_color(settings: &mut ClockSettings, channel: &str, color: &str) -> Result<(), String> {
    if config::rgb_from_hex(color).is_none() { return Err("Invalid HEX color".into()); }
    let destination = match (settings.dark_mode, channel) {
        (true, "background") => &mut settings.background_color,
        (true, "clock") => &mut settings.clock_color,
        (true, "date") => &mut settings.date_color,
        (false, "background") => &mut settings.light_background_color,
        (false, "clock") => &mut settings.light_clock_color,
        (false, "date") => &mut settings.light_date_color,
        _ => return Err("Unknown color channel".into()),
    };
    *destination = color.to_ascii_uppercase();
    Ok(())
}

// RC3.2: User-palette tool colors. Every surface is a restrained tint of the
// active background; text and accent are contrast-checked against their surface.
#[derive(Clone, Copy)]
struct ToolPalette {
    canvas: Color,
    panel: Color,
    card: Color,
    display: Color,
    accent: Color,
    text: Color,
    muted: Color,
    display_text: Color,
}

fn blend_colors(base: Color, brand: Color, strength: f64) -> Color {
    let mix = |a: u8, b: u8| -> u8 {
        (f64::from(a) * (1.0 - strength) + f64::from(b) * strength)
            .round().clamp(0.0, 255.0) as u8
    };
    Color::from_rgb_u8(
        mix(base.red(), brand.red()),
        mix(base.green(), brand.green()),
        mix(base.blue(), brand.blue()),
    )
}

fn color_luminance(c: Color) -> f64 {
    let channel = |v: u8| -> f64 {
        let s = f64::from(v) / 255.0;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * channel(c.red()) + 0.7152 * channel(c.green()) + 0.0722 * channel(c.blue())
}

fn color_contrast(a: Color, b: Color) -> f64 {
    let (a, b) = (color_luminance(a), color_luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn readable_color(candidate: Color, surface: Color) -> Color {
    if color_contrast(candidate, surface) >= 4.5 { return candidate; }
    let white = Color::from_rgb_u8(250, 250, 255);
    let black = Color::from_rgb_u8(16, 22, 34);
    if color_contrast(white, surface) >= color_contrast(black, surface) {
        white
    } else {
        black
    }
}

fn tool_palette(settings: &ClockSettings) -> ToolPalette {
    let (bg_hex, clock_hex, date_hex) = active_colors(settings);
    let brand = adjusted_color(bg_hex, settings.brightness_percent, settings.contrast_percent);
    let clock = adjusted_color(clock_hex, settings.brightness_percent, settings.contrast_percent);
    let date = adjusted_color(date_hex, settings.brightness_percent, settings.contrast_percent);
    let dark = settings.dark_mode;
    let rgb = |r, g, b| Color::from_rgb_u8(r, g, b);
    let canvas = blend_colors(if dark { rgb(8, 13, 22) } else { rgb(242, 245, 250) }, brand, if dark { 0.28 } else { 0.20 });
    let panel = blend_colors(if dark { rgb(20, 32, 51) } else { rgb(255, 255, 255) }, brand, if dark { 0.23 } else { 0.13 });
    let card = blend_colors(if dark { rgb(28, 43, 64) } else { rgb(237, 243, 248) }, brand, if dark { 0.24 } else { 0.18 });
    let display = blend_colors(if dark { rgb(10, 22, 38) } else { rgb(245, 249, 253) }, brand, if dark { 0.38 } else { 0.21 });
    let accent = readable_color(clock, card);
    // One accent must remain readable on every surface that uses it; fall back
    // to the accessible light/dark text color if a custom HEX combination clashes.
    let surfaces = [panel, card, display];
    let accent = if surfaces.iter().all(|s| color_contrast(accent, *s) >= 4.5) {
        accent
    } else {
        let white = rgb(250, 250, 255);
        let black = rgb(16, 22, 34);
        let min_ratio = |fg| surfaces.iter().map(|s| color_contrast(fg, *s)).fold(f64::INFINITY, f64::min);
        if min_ratio(white) >= min_ratio(black) { white } else { black }
    };
    let text = readable_color(if dark { rgb(224, 244, 255) } else { rgb(20, 50, 71) }, panel);
    let muted = readable_color(date, panel);
    let display_text = readable_color(clock, display);
    ToolPalette { canvas, panel, card, display, accent, text, muted, display_text }
}

#[cfg(test)]
mod rc32_palette_tests {
    use super::*;

    fn assert_legible(settings: ClockSettings) {
        let p = tool_palette(&settings);
        assert!(color_contrast(p.text, p.panel) >= 4.5);
        assert!(color_contrast(p.muted, p.panel) >= 4.5);
        assert!(color_contrast(p.accent, p.card) >= 4.5);
        assert!(color_contrast(p.accent, p.panel) >= 4.5);
        assert!(color_contrast(p.accent, p.display) >= 4.5);
        assert!(color_contrast(p.display_text, p.display) >= 4.5);
    }

    #[test]
    fn palettes_remain_readable_with_custom_hex() {
        let mut settings = ClockSettings::default();
        assert_legible(settings.clone());
        settings.dark_mode = false;
        assert_legible(settings.clone());
        settings.background_color = "#FFFFFF".into();
        settings.clock_color = "#FFFFFF".into();
        settings.date_color = "#FFFFFF".into();
        settings.dark_mode = true;
        assert_legible(settings.clone());
        settings.light_background_color = "#000000".into();
        settings.light_clock_color = "#000000".into();
        settings.light_date_color = "#000000".into();
        settings.dark_mode = false;
        assert_legible(settings.clone());
        settings.brightness_percent = 40;
        settings.contrast_percent = 150;
        assert_legible(settings);
    }
}

fn sync_appearance(ui: &DigitalClockUI, settings: &ClockSettings) {
    let (bg, time, date) = active_colors(settings);
    ui.set_bg_hex(bg.into());
    ui.set_time_hex(time.into());
    ui.set_date_hex(date.into());
    ui.set_clock_scale(i32::from(settings.clock_size_percent));
    ui.set_date_scale(i32::from(settings.date_size_percent));
    ui.set_brightness_pct(i32::from(settings.brightness_percent));
    ui.set_tool_scale(i32::from(settings.tool_size_percent));
    ui.set_bedside_brightness_pct(i32::from(settings.bedside_brightness_percent));
    ui.set_bedside_clock_color(display::bedside_color(settings.bedside_brightness_percent));
    ui.set_contrast_pct(i32::from(settings.contrast_percent));
    ui.set_date_format_choice(settings.date_format.as_str().into());
    ui.set_font_family_choice(settings.font_family.as_str().into());
    ui.set_home_background(adjusted_color(bg, settings.brightness_percent, settings.contrast_percent));
    ui.set_home_clock_color(adjusted_color(time, settings.brightness_percent, settings.contrast_percent));
    ui.set_home_date_color(adjusted_color(date, settings.brightness_percent, settings.contrast_percent));
    let palette = tool_palette(settings);
    ui.set_tool_canvas(palette.canvas);
    ui.set_tool_panel(palette.panel);
    ui.set_tool_card(palette.card);
    ui.set_tool_display(palette.display);
    ui.set_tool_accent(palette.accent);
    ui.set_tool_text(palette.text);
    ui.set_tool_muted(palette.muted);
    ui.set_tool_display_text(palette.display_text);
}

fn sync_profiles(ui: &DigitalClockUI, state: &P2State) {
    let names: Vec<slint::SharedString> = state.profiles.iter().map(|p| p.name.as_str().into()).collect();
    ui.set_profile_names(ModelRc::from(names.as_slice()));
    // Selection remains stable if it still exists.
    if !state.profiles.iter().any(|p| p.name == ui.get_selected_profile().as_str()) {
        ui.set_selected_profile(names.first().cloned().unwrap_or_default());
    }
}


fn sync_locale(ui: &DigitalClockUI, lang: &str) {
    ui.set_language_code(lang.into());
    ui.set_txt_alarm(i18n::tr(lang,"alarm").into());
    ui.set_txt_timer(i18n::tr(lang,"timer").into());
    ui.set_txt_stopwatch(i18n::tr(lang,"stopwatch").into());
    ui.set_txt_world(i18n::tr(lang,"world").into());
    ui.set_txt_settings(i18n::tr(lang,"settings").into());
    ui.set_txt_backup(i18n::tr(lang,"backup").into());
    ui.set_txt_close(i18n::tr(lang,"close").into());
    ui.set_txt_add(i18n::tr(lang,"add").into());
    ui.set_txt_remove(i18n::tr(lang,"remove").into());
    ui.set_txt_start(i18n::tr(lang,"start").into());
    ui.set_txt_reset(i18n::tr(lang,"reset").into());
    ui.set_txt_font(i18n::tr(lang,"font").into());
    ui.set_txt_new(i18n::tr(lang,"new").into());
    ui.set_txt_edit(i18n::tr(lang,"edit").into());
    ui.set_txt_save_time(i18n::tr(lang,"save-time").into());
    ui.set_txt_tone(i18n::tr(lang,"tone").into());
    ui.set_txt_preview(i18n::tr(lang,"preview").into());
    ui.set_txt_enable(i18n::tr(lang,"enable").into());
    ui.set_txt_disable(i18n::tr(lang,"disable").into());
    ui.set_txt_add_city(i18n::tr(lang,"add-city").into());
    ui.set_txt_set(i18n::tr(lang,"set").into());
    ui.set_txt_lap(i18n::tr(lang,"lap").into());
    ui.set_txt_clock_date(i18n::tr(lang,"clock-date").into());
    ui.set_txt_format_date(i18n::tr(lang,"format-date").into());
    ui.set_txt_background(i18n::tr(lang,"background").into());
    ui.set_txt_clock(i18n::tr(lang,"clock").into());
    ui.set_txt_date(i18n::tr(lang,"date").into());
    ui.set_txt_clock_size(i18n::tr(lang,"clock-size").into());
    ui.set_txt_date_size(i18n::tr(lang,"date-size").into());
    ui.set_txt_brightness(i18n::tr(lang,"brightness").into());
    ui.set_txt_contrast(i18n::tr(lang,"contrast").into());
    ui.set_txt_profiles(i18n::tr(lang,"profiles").into());
    ui.set_txt_save_profile(i18n::tr(lang,"save-profile").into());
    ui.set_txt_load_profile(i18n::tr(lang,"load-profile").into());
    ui.set_txt_delete_profile(i18n::tr(lang,"delete-profile").into());
    ui.set_txt_seconds(i18n::tr(lang,"seconds").into());
    ui.set_txt_hide_hex(i18n::tr(lang,"hide-hex").into());
    ui.set_txt_custom_hex(i18n::tr(lang,"custom-hex").into());
    ui.set_txt_profile_placeholder(i18n::tr(lang,"profile-placeholder").into());
    ui.set_txt_windowed(i18n::tr(lang,"windowed").into());
    ui.set_txt_fullscreen(i18n::tr(lang,"fullscreen").into());
    ui.set_txt_light_theme(i18n::tr(lang,"light-theme").into());
    ui.set_txt_dark_theme(i18n::tr(lang,"dark-theme").into());
    ui.set_txt_backup_export(i18n::tr(lang,"backup-export").into());
    ui.set_txt_backup_save_as(i18n::tr(lang,"backup-save-as").into());
    ui.set_txt_backup_open_file(i18n::tr(lang,"backup-open-file").into());
    ui.set_txt_backup_import(i18n::tr(lang,"backup-import").into());
    ui.set_txt_timer_note(i18n::tr(lang,"timer-note").into());
    ui.set_txt_time_format(i18n::tr(lang,"time-format").into());
    ui.set_txt_theme(i18n::tr(lang,"theme").into());
    ui.set_txt_restore_default(i18n::tr(lang,"restore-default").into());
    ui.set_txt_exit_mode(i18n::tr(lang,"exit-mode").into());
    ui.set_txt_bedside_brightness(i18n::tr(lang,"bedside-brightness").into());
    ui.set_txt_tool_size(i18n::tr(lang,"tool-size").into());
    ui.set_txt_language(i18n::tr(lang,"language").into());
    ui.set_txt_backup_folder(i18n::tr(lang,"backup-folder").into());
    ui.set_txt_backup_select(i18n::tr(lang,"backup-select").into());
    ui.set_txt_color_hint(i18n::tr(lang,"color-hint").into());
    ui.set_txt_hex_hint(i18n::tr(lang,"hex-hint").into());
    ui.set_txt_apply_hex(i18n::tr(lang,"apply-hex").into());
    ui.set_txt_alarm_help(i18n::tr(lang,"alarm-help").into());
    ui.set_txt_custom_audio(i18n::tr(lang,"custom-audio").into());
    ui.set_txt_choose_sound(i18n::tr(lang,"choose-sound").into());
    ui.set_txt_reset_sound(i18n::tr(lang,"reset-sound").into());
    ui.set_txt_world_help(i18n::tr(lang,"world-help").into());
    ui.set_txt_clock_view(i18n::tr(lang,"clock-view").into());
    ui.set_txt_portable_hint(i18n::tr(lang,"portable-hint").into());
    ui.set_txt_confirm(i18n::tr(lang,"confirm").into());
    ui.set_txt_dev_build(i18n::tr(lang,"dev-build").into());
    ui.set_txt_limit_short(i18n::tr(lang,"limit-short").into());
    ui.set_txt_24_hours(i18n::tr(lang,"24-hours").into());
    ui.set_txt_12_hours(i18n::tr(lang,"12-hours").into());
    ui.set_txt_yes(i18n::tr(lang,"yes").into());
    ui.set_txt_no(i18n::tr(lang,"no").into());
    ui.set_txt_about_title(i18n::tr(lang,"about-title").into());
    ui.set_txt_about_intro(i18n::tr(lang,"about-intro").into());
    ui.set_txt_about_features(i18n::tr(lang,"about-features").into());
    ui.set_txt_about_list(i18n::tr(lang,"about-list").into());
    ui.set_txt_about_portable(i18n::tr(lang,"about-portable").into());
    ui.set_txt_about_limits(i18n::tr(lang,"about-limits").into());
    ui.set_txt_about_limit_note(i18n::tr(lang,"about-limit-note").into());
    ui.set_txt_about_donate(i18n::tr(lang,"about-donate").into());
    ui.set_txt_about_donate_note(i18n::tr(lang,"about-donate-note").into());
    ui.set_txt_about_app_name(i18n::tr(lang,"about-app-name").into());
    ui.set_txt_visit_apps(i18n::tr(lang,"visit-apps").into());
    ui.set_txt_paypal_methods(i18n::tr(lang,"paypal-methods").into());
    ui.set_txt_stripe_methods(i18n::tr(lang,"stripe-methods").into());
    ui.set_txt_donation_methods(i18n::tr(lang,"donation-methods").into());
    ui.set_txt_paypal_description(i18n::tr(lang,"paypal-description").into());
    ui.set_txt_stripe_description(i18n::tr(lang,"stripe-description").into());
    ui.set_txt_paypal_action(i18n::tr(lang,"paypal-action").into());
    ui.set_txt_stripe_action(i18n::tr(lang,"stripe-action").into());
    ui.set_txt_crypto_wallet(i18n::tr(lang,"crypto-wallet").into());
    ui.set_txt_payment_note(i18n::tr(lang,"payment-note").into());
    ui.set_txt_crypto_description(i18n::tr(lang,"crypto-description").into());
    ui.set_txt_copy(i18n::tr(lang,"copy").into());
    ui.set_txt_copied(i18n::tr(lang,"copied").into());
    ui.set_txt_copy_failed(i18n::tr(lang,"copy-failed").into());
    ui.set_txt_timer_hours(i18n::tr(lang,"hours").into());
    ui.set_txt_timer_minutes(i18n::tr(lang,"minutes").into());
    ui.set_txt_timer_seconds(i18n::tr(lang,"seconds").into());
    ui.set_txt_timer_help(i18n::tr(lang,"timer-help").into());
}

fn sync_backups(ui: &DigitalClockUI, state_file: &Path) {
    match backup::list(state_file) {
        Ok(names) => {
            let selected = ui.get_selected_backup().to_string();
            let items: Vec<slint::SharedString> = names.iter().map(|v| v.as_str().into()).collect();
            ui.set_backup_files(ModelRc::from(items.as_slice()));
            if !names.iter().any(|name| name == &selected) {
                ui.set_selected_backup(names.first().map(String::as_str).unwrap_or("").into());
            }
        },
        Err(error) => ui.set_backup_status(format!("{}: {error}",i18n::tr(ui.get_language_code().as_str(),"backup-error")).into()),
    }
}

/// Converts two read-only ComboBox selections into the exact stored HH:MM format.
/// The original domain validator remains authoritative.
fn build_alarm_time(hour: &str, minute: &str) -> Result<String, String> {
    if hour.len() != 2 || minute.len() != 2 ||
        !hour.bytes().all(|b| b.is_ascii_digit()) ||
        !minute.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Neispravan odabir sati ili minuta".into());
    }
    let combined = format!("{hour}:{minute}");
    domain::parse_alarm_time(&combined).map_err(str::to_string)?;
    Ok(combined)
}

fn sync_alarm_ui(ui: &DigitalClockUI, state: &RuntimeState) {
    let rows: Vec<AlarmRow> = state.alarms.alarms.iter().map(|alarm| AlarmRow {
        id: i32::try_from(alarm.id).unwrap_or(0),
        time: alarm.time.as_str().into(),
        enabled: alarm.enabled,
    }).collect();
    ui.set_alarm_rows(ModelRc::from(rows.as_slice()));
}


fn sync_world_ui(ui: &DigitalClockUI, state: &RuntimeState, now: DateTime<Utc>) {
    let rows: Vec<CityRow> = state.stored.cities.iter().filter_map(|zone| {
        let descriptor = domain::city_for_zone(zone)?;
        Some(CityRow {
            zone: zone.as_str().into(),
            name: descriptor.name.into(),
            local_time: domain::world_city_time(zone, now)?.into(),
        })
    }).collect();
    ui.set_city_rows(ModelRc::from(rows.as_slice()));
}

fn refresh_ui(ui: &DigitalClockUI, state: &mut RuntimeState) {
    let local_now = Local::now();
    let instant = Instant::now();
    let (clock_main, clock_suffix) = clock_face_parts(&local_now, &state.stored.settings);
    ui.set_clock_main_text(clock_main.into());
    ui.set_clock_suffix_text(clock_suffix.into());
    ui.set_clock_text(format_time(local_now, &state.stored.settings).into());
    ui.set_date_text(format_date(local_now, &state.stored.settings).into());
    ui.set_hour_24(state.stored.settings.hour_24);
    ui.set_show_seconds(state.stored.settings.show_seconds);
    ui.set_show_date(state.stored.settings.show_date);
    ui.set_dark_mode(state.stored.settings.dark_mode);

    if local_now.timestamp() != state.last_world_refresh_second {
        sync_world_ui(ui, state, Utc::now());
        state.last_world_refresh_second = local_now.timestamp();
    }
    ui.set_stopwatch_text(state.stopwatch.display(instant).into());
    ui.set_stopwatch_button(i18n::tr(&state.stored.settings.language, if state.stopwatch.is_running() { "pause" } else { "start" }).into());
    ui.set_laps_text(state.stopwatch.lap_summary_localized(i18n::tr(&state.stored.settings.language,"no-laps"),i18n::tr(&state.stored.settings.language,"lap-label")).into());
    ui.set_countdown_text(domain::format_duration(state.countdown.remaining_at(instant)).into());
    ui.set_countdown_button(i18n::tr(&state.stored.settings.language, if state.countdown.is_running() { "pause" } else { "start" }).into());

    // Render-only: never consume alarm/timer events from UI handlers.
    ui.set_alert_active(state.alert.is_some());
    ui.set_alert_text(state.alert.clone().unwrap_or_default().into());
}

/// Called only from the periodic 100 ms timer. All due events are dispatched
/// in the same call-site that starts the sound, so UI refreshes cannot lose it.
fn poll_alert_events(state: &mut RuntimeState) -> Option<bool> {
    let now = Local::now();
    let mut channel = None;
    let due = state.alarms.due(now);
    if !due.is_empty() {
        state.alert = Some(format!("ALARM! {}", due.iter().map(|a| a.time.as_str()).collect::<Vec<_>>().join(", ")));
        channel = Some(false);
    }
    if state.countdown.tick(Instant::now()) {
        state.alert = Some(i18n::tr(&state.stored.settings.language, "countdown-done").into());
        channel = Some(true);
    }
    channel
}

fn persist(ui: &DigitalClockUI, state: &mut RuntimeState, path: &Path, may_save: bool) -> bool {
    if !may_save {
        ui.set_storage_warning(true);
        ui.set_status_text(i18n::tr(&state.stored.settings.language,"save-blocked").into());
        return false;
    }
    state.stored.alarms = state.alarms.alarms.clone();
    match config::save(path, &state.stored) {
        Ok(()) => {
            ui.set_storage_warning(false);
            ui.set_status_text(format!("{}: Data/app-state-native-p28.json",i18n::tr(&state.stored.settings.language,"state-saved")).into());
            true
        },
        Err(error) => {
            ui.set_storage_warning(true);
            ui.set_status_text(format!("{}: {error}",i18n::tr(&state.stored.settings.language,"save-failed")).into());
            false
        },
    }
}

#[cfg(windows)]
fn copy_to_clipboard(value: &str) -> std::io::Result<()> {
    use std::{ffi::OsString, io::Write, os::windows::{ffi::OsStringExt, process::CommandExt},
        path::PathBuf, process::{Command, Stdio}};
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    }
    let mut system_dir = [0u16; 32768];
    let len = unsafe { GetSystemDirectoryW(system_dir.as_mut_ptr(), system_dir.len() as u32) } as usize;
    if len == 0 || len >= system_dir.len() {
        return Err(std::io::Error::other("Cannot resolve Windows system directory"));
    }
    let clip = PathBuf::from(OsString::from_wide(&system_dir[..len])).join("clip.exe");
    let mut process = Command::new(clip)
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null())
        .spawn()?;
    let result = process.stdin.take().ok_or_else(|| std::io::Error::other("Clipboard pipe unavailable"))?
        .write_all(value.as_bytes());
    if let Err(error) = result { let _ = process.kill(); return Err(error); }
    if process.wait()?.success() { Ok(()) } else { Err(std::io::Error::other("Clipboard command failed")) }
}
#[cfg(not(windows))]
fn copy_to_clipboard(_value: &str) -> std::io::Result<()> {
    Err(std::io::Error::other("Windows-only clipboard"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(windows, feature = "windows-installer"))]
    let screensaver_mode = windows_screensaver::mode();
    #[cfg(all(windows, feature = "windows-installer"))]
    if let windows_screensaver::Mode::Preview(parent) = screensaver_mode {
        return windows_screensaver::run_preview(parent);
    }
    #[cfg(windows)]
    let is_saver_launch = {
        #[cfg(feature = "windows-installer")]
        { matches!(screensaver_mode, windows_screensaver::Mode::Screen) }
        #[cfg(not(feature = "windows-installer"))]
        { false }
    };
    let path = config::data_file()?;

// The screensaver renders without writing shared settings and must coexist
    // with the running app. Normal application and /c remain single-instance.
    #[cfg(windows)]
    let _single_instance_guard = if is_saver_launch {
        None
    } else {
        match single_instance::InstanceGuard::acquire(&path)? {
            Some(guard) => Some(guard),
            None => { single_instance::show_already_open_message(); return Ok(()); }
        }
    };

    let (stored, initial_status, may_save) = match config::load(&path) {
        Ok(stored) => (stored, "Digital Clock · Local data".to_string(), true),
        Err(error) => (P2State::default(), format!("{}: {error}",i18n::tr("en","json-warning")), false),
    };
#[cfg(windows)]
    let may_save = may_save && !is_saver_launch;
    let state = Rc::new(RefCell::new(RuntimeState::new(stored)?));

    let ui = DigitalClockUI::new()?;
    let hours: Vec<slint::SharedString> = (0..24).map(|n| format!("{n:02}").into()).collect();
    let minutes: Vec<slint::SharedString> = (0..60).map(|n| format!("{n:02}").into()).collect();
    ui.set_alarm_hours(ModelRc::from(hours.as_slice()));
    ui.set_alarm_minutes(ModelRc::from(minutes.as_slice()));
    let timer_hours: Vec<slint::SharedString> = (0..100).map(|n| format!("{n:02}").into()).collect();
    ui.set_timer_hours(ModelRc::from(timer_hours.as_slice()));
    ui.set_timer_minutes(ModelRc::from(minutes.as_slice()));
    let wallet_rows: Vec<DonationRow> = [
        ("BTC", "bc1qwlrxrh64peukga0fp59m9yg7gpf0yj8q7fxnsc"),
        ("ETH", "0xA99A52085c6725854daa46bb302041569c8bA4E3"),
        ("XRP", "rP43SsrkhPkxTsFohMAm32sAQg7vqwmDpr"),
        ("SOL", "8xkdVTEaDGuWu4aE3HpEx8r9Aux98JZbdsMiDQvJWBWR"),
        ("DOGE", "DGAT32ku8WmFaTDxCgVuRuVpUFmfdmD5Jb"),
        ("XLM", "GCYH4OD4I2GNRKFFOYROE3N3S2HCT5RXIML3TZV5DP3TLTLXPXQXIJZ3"),
        ("LTC", "LWtaFniqdYpv2xJtqo9WqDwCsQ2cW6PYWi"),
        ("RVN", "RAtXzKZyB3awfq2u2cK8YppC9kJamU5tPQ"),
    ].iter().map(|(coin, address)| DonationRow {coin:(*coin).into(), address:(*address).into()}).collect();
    ui.set_donation_rows(ModelRc::from(wallet_rows.as_slice()));
    ui.set_status_text(initial_status.into());
    ui.set_storage_warning(!may_save);
    refresh_ui(&ui, &mut state.borrow_mut());
    sync_alarm_ui(&ui, &state.borrow());
    let city_options: Vec<slint::SharedString> = domain::WORLD_CITIES.iter()
        .map(|city| city.option_label().into()).collect();
    ui.set_city_options(ModelRc::from(city_options.as_slice()));
    ui.set_selected_city("Zagreb (Europe/Zagreb)".into());
    ui.set_alarm_tone(state.borrow().stored.alarm_tone.as_label().into());
    let formats: Vec<slint::SharedString> = config::DATE_FORMATS.iter().map(|s| (*s).into()).collect();
    let fonts: Vec<slint::SharedString> = config::FONT_FAMILIES.iter().map(|s| (*s).into()).collect();
    ui.set_date_format_choices(ModelRc::from(formats.as_slice()));
    ui.set_font_choices(ModelRc::from(fonts.as_slice()));
    sync_appearance(&ui, &state.borrow().stored.settings);
    sync_profiles(&ui, &state.borrow().stored);
    sync_locale(&ui, &state.borrow().stored.settings.language);
    sync_backups(&ui, &path);

    let audio = Rc::new(sound::SoundEngine::default());
    #[cfg(all(windows, feature = "windows-installer"))]
    ui.set_installed_audio_enabled(true);
    #[cfg(all(windows, feature = "windows-installer"))]
    let audio_path = path.with_file_name("windows-audio.json");
    #[cfg(all(windows, feature = "windows-installer"))]
    let audio_files = Rc::new(RefCell::new(match windows_custom_audio::load(&audio_path) {
        Ok(prefs) => prefs,
        Err(error) => {
            ui.set_status_text(format!("Audio settings not loaded: {error}").into());
            windows_custom_audio::CustomAudioPrefs::default()
        }
    }));
    #[cfg(all(windows, feature = "windows-installer"))]
    {
        let saved = audio_files.borrow();
        ui.set_custom_alarm_label(windows_custom_audio::label(&saved.alarm).into());
        ui.set_custom_timer_label(windows_custom_audio::label(&saved.timer).into());
    }

    macro_rules! bind_toggle {
        ($method:ident, $field:ident) => {{
            let weak = ui.as_weak();
            let state = state.clone();
            let path = path.clone();
            ui.$method(move || {
                if let Some(ui) = weak.upgrade() {
                    let mut data = state.borrow_mut();
                    data.stored.settings.$field = !data.stored.settings.$field;
                    refresh_ui(&ui, &mut data);
                    persist(&ui, &mut data, &path, may_save);
                }
            });
        }};
    }
    bind_toggle!(on_toggle_hour_mode, hour_24);
    bind_toggle!(on_toggle_seconds, show_seconds);
    bind_toggle!(on_toggle_date, show_date);
    {
        let weak = ui.as_weak(); let state = state.clone(); let path = path.clone();
        ui.on_toggle_theme(move || {
            if let Some(ui) = weak.upgrade() {
                let mut data = state.borrow_mut();
                data.stored.settings.dark_mode = !data.stored.settings.dark_mode;
                sync_appearance(&ui, &data.stored.settings);
                refresh_ui(&ui, &mut data);
                persist(&ui, &mut data, &path, may_save);
            }
        });
    }

    // Appearance changes are explicit: no writes from Slint rendering/effects.
    {
        let weak=ui.as_weak(); let state=state.clone(); let path=path.clone();
        ui.on_apply_colors(move |bg, clock, date| {
            if let Some(ui)=weak.upgrade() {
                let colors=[bg.to_string(), clock.to_string(), date.to_string()];
                if colors.iter().any(|v| config::rgb_from_hex(v).is_none()) {
                    ui.set_home_message(i18n::tr(ui.get_language_code().as_str(),"invalid-color").into());
                    ui.set_home_panel(1);
                    return;
                }
                let mut data=state.borrow_mut();
                set_active_color(&mut data.stored.settings,"background", &colors[0]).unwrap();
                set_active_color(&mut data.stored.settings,"clock", &colors[1]).unwrap();
                set_active_color(&mut data.stored.settings,"date", &colors[2]).unwrap();
                sync_appearance(&ui,&data.stored.settings);
                persist(&ui,&mut data,&path,may_save);
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone(); let path = path.clone();
        ui.on_choose_color(move |channel, value| {
            if let Some(ui) = weak.upgrade() {
                let mut data = state.borrow_mut();
                if set_active_color(&mut data.stored.settings, channel.as_str(), value.as_str()).is_ok() {
                    sync_appearance(&ui, &data.stored.settings);
                    persist(&ui, &mut data, &path, may_save);
                }
            }
        });
    }
    // Live contrast/brightness preview: reuse the actual RGB transform, but
    // never change persisted settings until the Slider.released callback.
    {
        let weak = ui.as_weak();
        let state = state.clone();
        ui.on_preview_lighting(move |brightness, contrast| {
            if let Some(ui) = weak.upgrade() {
                let data = state.borrow();
                let (bg, clock, date) = active_colors(&data.stored.settings);
                let b = (brightness.round() as i32).clamp(0, 100) as u8;
                let c = (contrast.round() as i32).clamp(0, 100) as u8;
                ui.set_home_background(adjusted_color(bg, b, c));
                ui.set_home_clock_color(adjusted_color(clock, b, c));
                ui.set_home_date_color(adjusted_color(date, b, c));
            }
        });
    }
    macro_rules! bind_percent {
        ($callback:ident, $field:ident, $min:expr, $max:expr) => {{
            let weak=ui.as_weak(); let state=state.clone(); let path=path.clone();
            ui.$callback(move |raw| {
                if let Some(ui)=weak.upgrade() {
                    let bounded=(raw.round() as i32).clamp($min,$max) as u8;
                    let mut data=state.borrow_mut();
                    data.stored.settings.$field=bounded;
                    sync_appearance(&ui,&data.stored.settings);
                    persist(&ui,&mut data,&path,may_save);
                }
            });
        }};
    }
    bind_percent!(on_set_clock_scale,clock_size_percent,50,100);
    bind_percent!(on_set_tool_scale,tool_size_percent,60,150);
    bind_percent!(on_set_bedside_brightness,bedside_brightness_percent,1,100);
    {
        let weak=ui.as_weak();
        ui.on_preview_bedside(move |raw| {
            if let Some(ui)=weak.upgrade() {
                ui.set_bedside_clock_color(display::bedside_color((raw.round() as i32).clamp(1,100) as u8));
            }
        });
    }
    bind_percent!(on_set_date_scale,date_size_percent,40,100);
    bind_percent!(on_set_brightness,brightness_percent,0,100);
    bind_percent!(on_set_contrast,contrast_percent,0,100);
    {
        let weak=ui.as_weak(); let state=state.clone(); let path=path.clone();
        ui.on_set_date_format(move |selected| {
            if !config::DATE_FORMATS.contains(&selected.as_str()) { return; }
            if let Some(ui)=weak.upgrade() {
                let mut data=state.borrow_mut();
                data.stored.settings.date_format=selected.to_string();
                refresh_ui(&ui,&mut data);
                persist(&ui,&mut data,&path,may_save);
            }
        });
    }
    {
        let weak=ui.as_weak(); let state=state.clone(); let path=path.clone();
        ui.on_set_font_family(move |selected| {
            if !config::FONT_FAMILIES.contains(&selected.as_str()) { return; }
            if let Some(ui)=weak.upgrade() {
                let mut data=state.borrow_mut();
                data.stored.settings.font_family=selected.to_string();
                sync_appearance(&ui,&data.stored.settings);
                persist(&ui,&mut data,&path,may_save);
            }
        });
    }
    {
        let weak=ui.as_weak(); let state=state.clone(); let path=path.clone();
        ui.on_save_profile(move |name| {
            if let Some(ui)=weak.upgrade() {
                let name=name.trim().to_string();
                let mut data=state.borrow_mut();
                let validation_error = if name.is_empty() { Some(i18n::tr(&data.stored.settings.language,"profile-name")) }
                    else if name.chars().count() > 100 { Some(i18n::tr(&data.stored.settings.language,"profile-long")) }
                    else if data.stored.profiles.iter().any(|p| p.name.eq_ignore_ascii_case(&name)) {
                        Some(i18n::tr(&data.stored.settings.language,"profile-exists"))
                    } else if data.stored.profiles.len() >= 100 { Some(i18n::tr(&data.stored.settings.language,"profile-limit")) }
                    else { None };
                if let Some(message) = validation_error {
                    ui.set_profile_feedback(message.into());
                    return;
                }
                let settings=data.stored.settings.clone();
                data.stored.profiles.push(AppearanceProfile{name:name.clone(),settings});
                sync_profiles(&ui,&data.stored);
                ui.set_selected_profile(name.clone().into());
                ui.set_profile_name_input("".into());
                if persist(&ui,&mut data,&path,may_save) {
                    ui.set_profile_feedback(format!("{}: {}",i18n::tr(&data.stored.settings.language,"profile-saved"),name).into());
                } else {
                    ui.set_profile_feedback(i18n::tr(&data.stored.settings.language,"profile-save-error").into());
                }
            }
        });
    }
    {
        let weak=ui.as_weak(); let state=state.clone(); let path=path.clone();
        ui.on_load_profile(move |name| {
            if let Some(ui)=weak.upgrade() {
                let mut data=state.borrow_mut();
                if let Some(settings)=data.stored.profiles.iter().find(|p| p.name==name.as_str()).map(|p| p.settings.clone()) {
                    data.stored.settings=settings;
                    sync_appearance(&ui,&data.stored.settings);
                    sync_locale(&ui,&data.stored.settings.language);
                    refresh_ui(&ui,&mut data);
                    persist(&ui,&mut data,&path,may_save);
                }
            }
        });
    }
    {
        let weak=ui.as_weak(); let state=state.clone(); let path=path.clone();
        ui.on_remove_profile(move |name| {
            if let Some(ui)=weak.upgrade() {
                let mut data=state.borrow_mut();
                let before=data.stored.profiles.len();
                data.stored.profiles.retain(|p| p.name!=name.as_str());
                if data.stored.profiles.len()!=before {
                    sync_profiles(&ui,&data.stored);
                    persist(&ui,&mut data,&path,may_save);
                }
            }
        });
    }
    {
        let weak=ui.as_weak(); let state=state.clone(); let path=path.clone();
        ui.on_reset_appearance(move || {
            if let Some(ui)=weak.upgrade() {
                let mut data=state.borrow_mut();
                data.stored.settings=ClockSettings::default();
                sync_appearance(&ui,&data.stored.settings);
                sync_locale(&ui,&data.stored.settings.language);
                refresh_ui(&ui,&mut data);
                persist(&ui,&mut data,&path,may_save);
            }
        });
    }

    {
        let weak = ui.as_weak(); let state = state.clone(); let path = path.clone();
        ui.on_add_alarm(move |hour, minute| {
            if let Some(ui) = weak.upgrade() {
                let mut data = state.borrow_mut();
                let outcome = build_alarm_time(hour.as_str(), minute.as_str())
                    .and_then(|time| data.alarms.add(&time));
                match outcome {
                    Ok(()) => { persist(&ui, &mut data, &path, may_save); sync_alarm_ui(&ui, &data); },
                    Err(error) => ui.set_status_text(format!("{}: {error}",i18n::tr(&data.stored.settings.language,"alarm-add-error")).into()),
                }
                refresh_ui(&ui, &mut data);
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone();
        ui.on_select_alarm(move |id| {
            if let Some(ui) = weak.upgrade() {
                let data = state.borrow();
                if let Some(alarm) = data.alarms.alarms.iter().find(|a| a.id == id as u32) {
                    if let Some((hour, minute)) = alarm.time.split_once(':') {
                        ui.set_edit_alarm_hour(hour.into());
                        ui.set_edit_alarm_minute(minute.into());
                        ui.set_selected_alarm_id(id);
                    }
                }
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone(); let path = path.clone();
        ui.on_update_alarm(move |id, hour, minute| {
            if let Some(ui) = weak.upgrade() {
                let mut data = state.borrow_mut();
                let outcome = build_alarm_time(hour.as_str(), minute.as_str())
                    .and_then(|time| data.alarms.update(id as u32, &time));
                match outcome {
                    Ok(()) => { persist(&ui, &mut data, &path, may_save); sync_alarm_ui(&ui, &data); }
                    Err(error) => ui.set_status_text(format!("{}: {error}",i18n::tr(&data.stored.settings.language,"alarm-edit-error")).into()),
                }
                refresh_ui(&ui, &mut data);
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone(); let path = path.clone();
        ui.on_remove_alarm(move |id| {
            if let Some(ui) = weak.upgrade() {
                let mut data = state.borrow_mut();
                if data.alarms.remove(id as u32) {
                    if ui.get_selected_alarm_id() == id { ui.set_selected_alarm_id(0); }
                    persist(&ui, &mut data, &path, may_save);
                    sync_alarm_ui(&ui, &data);
                } else { ui.set_status_text(i18n::tr(&data.stored.settings.language,"no-alarm").into()); }
                refresh_ui(&ui, &mut data);
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone(); let path = path.clone();
        ui.on_toggle_alarm(move |id| {
            if let Some(ui) = weak.upgrade() {
                let mut data = state.borrow_mut();
                if data.alarms.toggle(id as u32) {
                    persist(&ui, &mut data, &path, may_save);
                    sync_alarm_ui(&ui, &data);
                } else { ui.set_status_text(i18n::tr(&data.stored.settings.language,"no-alarm").into()); }
                refresh_ui(&ui, &mut data);
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone(); let path = path.clone();
        ui.on_select_alarm_tone(move |choice| {
            if let Some(ui) = weak.upgrade() {
                if let Some(tone) = AlarmTone::from_label(choice.as_str()) {
                    let mut data = state.borrow_mut();
                    data.stored.alarm_tone = tone;
                    persist(&ui, &mut data, &path, may_save);
                }
            }
        });
    }
    {
        let state = state.clone(); let audio = audio.clone();
        #[cfg(all(windows, feature = "windows-installer"))]
        let saved = audio_files.clone();
        ui.on_preview_alarm_tone(move || {
            let tone = state.borrow().stored.alarm_tone;
            #[cfg(all(windows, feature = "windows-installer"))]
            audio.preview_with_file(tone, saved.borrow().alarm.clone());
            #[cfg(not(all(windows, feature = "windows-installer")))]
            audio.preview(tone);
        });
    }
    #[cfg(all(windows, feature = "windows-installer"))]
    {
        let weak = ui.as_weak(); let paths = audio_files.clone(); let store = audio_path.clone();
        ui.on_choose_alarm_audio(move || {
            if let Some(ui) = weak.upgrade() {
                match file_dialog::select_audio_file() {
                    Ok(Some(chosen)) => match windows_custom_audio::validate_audio_file(&chosen) {
                        Ok(path) => {
                            let mut prefs = paths.borrow().clone();
                            prefs.alarm = Some(path.to_string_lossy().into_owned());
                            match windows_custom_audio::save(&store, &prefs) {
                                Ok(()) => { ui.set_custom_alarm_label(windows_custom_audio::label(&prefs.alarm).into()); *paths.borrow_mut() = prefs; },
                                Err(err) => ui.set_status_text(format!("Audio preferences: {err}").into()),
                            }
                        }
                        Err(err) => ui.set_status_text(format!("Audio file: {err}").into()),
                    },
                    Ok(None) => {},
                    Err(err) => ui.set_status_text(format!("Audio selection: {err}").into()),
                }
            }
        });
    }
    #[cfg(all(windows, feature = "windows-installer"))]
    {
        let weak = ui.as_weak(); let paths = audio_files.clone(); let store = audio_path.clone();
        ui.on_choose_timer_audio(move || {
            if let Some(ui) = weak.upgrade() {
                match file_dialog::select_audio_file() {
                    Ok(Some(chosen)) => match windows_custom_audio::validate_audio_file(&chosen) {
                        Ok(path) => {
                            let mut prefs = paths.borrow().clone();
                            prefs.timer = Some(path.to_string_lossy().into_owned());
                            match windows_custom_audio::save(&store, &prefs) {
                                Ok(()) => { ui.set_custom_timer_label(windows_custom_audio::label(&prefs.timer).into()); *paths.borrow_mut() = prefs; },
                                Err(err) => ui.set_status_text(format!("Audio preferences: {err}").into()),
                            }
                        }
                        Err(err) => ui.set_status_text(format!("Audio file: {err}").into()),
                    },
                    Ok(None) => {},
                    Err(err) => ui.set_status_text(format!("Audio selection: {err}").into()),
                }
            }
        });
    }
    #[cfg(all(windows, feature = "windows-installer"))]
    {
        let weak = ui.as_weak(); let paths = audio_files.clone(); let store = audio_path.clone();
        ui.on_clear_alarm_audio(move || {
            if let Some(ui) = weak.upgrade() {
                let mut prefs = paths.borrow().clone(); prefs.alarm = None;
                match windows_custom_audio::save(&store, &prefs) {
                    Ok(()) => { ui.set_custom_alarm_label(windows_custom_audio::label(&prefs.alarm).into()); *paths.borrow_mut() = prefs; },
                    Err(err) => ui.set_status_text(format!("Audio preferences: {err}").into()),
                }
            }
        });
    }
    #[cfg(all(windows, feature = "windows-installer"))]
    {
        let weak = ui.as_weak(); let paths = audio_files.clone(); let store = audio_path.clone();
        ui.on_clear_timer_audio(move || {
            if let Some(ui) = weak.upgrade() {
                let mut prefs = paths.borrow().clone(); prefs.timer = None;
                match windows_custom_audio::save(&store, &prefs) {
                    Ok(()) => { ui.set_custom_timer_label(windows_custom_audio::label(&prefs.timer).into()); *paths.borrow_mut() = prefs; },
                    Err(err) => ui.set_status_text(format!("Audio preferences: {err}").into()),
                }
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone();
        ui.on_set_timer(move |raw| {
            if let Some(ui) = weak.upgrade() {
                let mut data = state.borrow_mut();
                match domain::parse_timer_duration(raw.as_str()) {
                    Ok(duration) => {
                        data.countdown.configure(duration);
                        ui.set_status_text(i18n::tr(&data.stored.settings.language,"timer-set").into());
                    },
                    Err(error) => ui.set_status_text(error.into()),
                }
                refresh_ui(&ui, &mut data);
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone();
        ui.on_start_pause_timer(move || {
            if let Some(ui) = weak.upgrade() {
                let mut data = state.borrow_mut();
                if let Err(error) = data.countdown.start_pause(Instant::now()) {
                    ui.set_status_text(error.into());
                }
                refresh_ui(&ui, &mut data);
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone();
        ui.on_reset_timer(move || {
            if let Some(ui) = weak.upgrade() {
                let mut data = state.borrow_mut();
                data.countdown.reset();
                refresh_ui(&ui, &mut data);
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone();
        ui.on_start_pause_stopwatch(move || {
            if let Some(ui) = weak.upgrade() {
                let mut data = state.borrow_mut();
                data.stopwatch.start_pause(Instant::now());
                refresh_ui(&ui, &mut data);
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone();
        ui.on_lap_stopwatch(move || {
            if let Some(ui) = weak.upgrade() {
                let mut data = state.borrow_mut();
                if !data.stopwatch.lap(Instant::now()) {
                    ui.set_status_text(i18n::tr(&data.stored.settings.language,"lap-first").into());
                }
                refresh_ui(&ui, &mut data);
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone();
        ui.on_reset_stopwatch(move || {
            if let Some(ui) = weak.upgrade() {
                let mut data = state.borrow_mut();
                data.stopwatch.reset();
                refresh_ui(&ui, &mut data);
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone(); let path = path.clone();
        ui.on_add_world_city(move || {
            if let Some(ui) = weak.upgrade() {
                let choice = ui.get_selected_city().to_string();
                let mut data = state.borrow_mut();
                if let Some(city_zone) = domain::zone_for_city_option(&choice) {
                    if data.stored.cities.iter().any(|z| z == city_zone) {
                        ui.set_status_text(i18n::tr(&data.stored.settings.language,"city-exists").into());
                    } else if data.stored.cities.len() >= domain::WORLD_CITIES.len() {
                        ui.set_status_text(i18n::tr(&data.stored.settings.language,"city-limit").into());
                    } else {
                        data.stored.cities.push(city_zone.to_owned());
                        persist(&ui, &mut data, &path, may_save);
                    }
                } else {
                    ui.set_status_text(i18n::tr(&data.stored.settings.language,"city-unknown").into());
                }
                data.last_world_refresh_second = i64::MIN;
                refresh_ui(&ui, &mut data);
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone(); let path = path.clone();
        ui.on_remove_world_city(move |zone| {
            if let Some(ui) = weak.upgrade() {
                let zone = zone.to_string();
                let mut data = state.borrow_mut();
                let original_count = data.stored.cities.len();
                data.stored.cities.retain(|existing| existing != &zone);
                if data.stored.cities.len() != original_count {
                    persist(&ui, &mut data, &path, may_save);
                } else {
                    ui.set_status_text(i18n::tr(&data.stored.settings.language,"city-absent").into());
                }
                data.last_world_refresh_second = i64::MIN;
                refresh_ui(&ui, &mut data);
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone(); let audio = audio.clone();
        ui.on_dismiss_alert(move || {
            if let Some(ui) = weak.upgrade() {
                let mut data = state.borrow_mut();
                data.alert = None;
                audio.stop();
                refresh_ui(&ui, &mut data);
            }
        });
    }

    // Transient display modes; no writes to the saved appearance.
    let display_session = Rc::new(RefCell::new(display::DisplaySession::default()));
    {
        let weak = ui.as_weak(); let session = display_session.clone();
        ui.on_enter_display_mode(move |mode| {
            if let Some(ui) = weak.upgrade() {
                if mode != 1 && mode != 2 { return; }
                let mut d = session.borrow_mut();
                if !d.active { d.previous_fullscreen = ui.get_fullscreen_mode(); }
                d.active = true;
                if !display::keep_display_awake(mode == 2) {
                    ui.set_home_message(i18n::tr(ui.get_language_code().as_str(),"awake-error").into());
                }
                ui.set_display_mode(mode);
                ui.set_home_panel(0);
                ui.set_language_menu(false);
                ui.set_fullscreen_mode(true);
                ui.invoke_focus_display_hotkeys();
                // Focus can change during Windows fullscreen transition; restore on next event loop turn.
                let refocus=ui.as_weak();
                Timer::single_shot(Duration::from_millis(150), move || {
                    if let Some(window)=refocus.upgrade() {
                        if window.get_display_mode()!=0 { window.invoke_focus_display_hotkeys(); }
                    }
                });
            }
        });
    }
    {
        let weak = ui.as_weak(); let session = display_session.clone();
        ui.on_exit_display_mode(move || {
            if let Some(ui) = weak.upgrade() {
                let mut d = session.borrow_mut();
                display::keep_display_awake(false);
                ui.set_display_mode(0);
                ui.set_saver_shift_x(0);
                ui.set_saver_shift_y(0);
                ui.set_fullscreen_mode(d.previous_fullscreen);
                d.active = false;
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone(); let path = path.clone();
        ui.on_select_language(move |lang| {
            if let Some(ui) = weak.upgrade() {
                if !i18n::supported(lang.as_str()) { return; }
                let mut data = state.borrow_mut();
                data.stored.settings.language = lang.to_string();
                sync_locale(&ui, lang.as_str());
                refresh_ui(&ui, &mut data);
                persist(&ui, &mut data, &path, may_save);
            }
        });
    }
    {
        let weak = ui.as_weak();
        ui.on_copy_donation_address(move |address| {
            if let Some(ui) = weak.upgrade() {
                let allowed = [
                    "bc1qwlrxrh64peukga0fp59m9yg7gpf0yj8q7fxnsc",
                    "0xA99A52085c6725854daa46bb302041569c8bA4E3",
                    "rP43SsrkhPkxTsFohMAm32sAQg7vqwmDpr",
                    "8xkdVTEaDGuWu4aE3HpEx8r9Aux98JZbdsMiDQvJWBWR",
                    "DGAT32ku8WmFaTDxCgVuRuVpUFmfdmD5Jb",
                    "GCYH4OD4I2GNRKFFOYROE3N3S2HCT5RXIML3TZV5DP3TLTLXPXQXIJZ3",
                    "LWtaFniqdYpv2xJtqo9WqDwCsQ2cW6PYWi",
                    "RAtXzKZyB3awfq2u2cK8YppC9kJamU5tPQ",
                ];
                let ok = allowed.contains(&address.as_str()) && copy_to_clipboard(address.as_str()).is_ok();
                ui.set_last_copy_address(address.clone());
                ui.set_last_copy_success(ok);
                ui.set_info_copy_feedback(if ok { ui.get_txt_copied() } else { ui.get_txt_copy_failed() });
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone(); let path = path.clone();
        ui.on_create_backup(move || {
            if let Some(ui) = weak.upgrade() {
                let data = state.borrow();
                if !may_save { ui.set_backup_status(i18n::tr(&data.stored.settings.language,"backup-blocked").into()); return; }
                let mut snapshot = data.stored.clone();
                snapshot.alarms = data.alarms.alarms.clone();
                match backup::export(&path, &snapshot) {
                    Ok(file) => {
                        sync_backups(&ui, &path);
                        ui.set_selected_backup(file.into());
                        ui.set_backup_status(i18n::tr(&data.stored.settings.language,"backup-saved").into());
                    },
                    Err(error) => ui.set_backup_status(format!("{}: {error}",i18n::tr(&data.stored.settings.language,"backup-export-error")).into()),
                }
            }
        });
    }
    // Native Windows Save As: keeps Data/Backups as the separate quick-backup option.
    {
        let weak = ui.as_weak(); let state = state.clone();
        ui.on_save_backup_as(move || {
            if let Some(ui) = weak.upgrade() {
                if !may_save {
                    ui.set_backup_status(i18n::tr(ui.get_language_code().as_str(),"backup-blocked").into());
                    return;
                }
                let mut snapshot = {
                    let data = state.borrow();
                    let mut copy = data.stored.clone();
                    copy.alarms = data.alarms.alarms.clone();
                    copy
                };
                // Normalize only the snapshot, not the running settings.
                snapshot.alarms.shrink_to_fit();
                match file_dialog::select_json_file(true) {
                    Ok(Some(filename)) => match backup::export_external(&filename, &snapshot) {
                        Ok(()) => ui.set_backup_status(
                            format!("{}: {}", i18n::tr(ui.get_language_code().as_str(),"backup-saved-external"), filename.display()).into()
                        ),
                        Err(error) => ui.set_backup_status(
                            format!("{}: {error}", i18n::tr(ui.get_language_code().as_str(),"backup-export-error")).into()
                        ),
                    },
                    Ok(None) => (), // Cancel: no data or status change.
                    Err(error) => ui.set_backup_status(format!("{}: {error}",
                        i18n::tr(ui.get_language_code().as_str(),"backup-export-error")).into()),
                }
            }
        });
    }
    // Native Windows Open: validate external JSON and pre-save the current state
    // before replacing it. Cancel and invalid JSON cannot modify saved state.
    {
        let weak = ui.as_weak(); let state = state.clone(); let path = path.clone(); let audio = audio.clone();
        ui.on_open_external_backup(move || {
            if let Some(ui) = weak.upgrade() {
                if !may_save {
                    ui.set_backup_status(i18n::tr(ui.get_language_code().as_str(),"backup-blocked").into());
                    return;
                }
                let filename = match file_dialog::select_json_file(false) {
                    Ok(Some(file)) => file,
                    Ok(None) => return,
                    Err(error) => {
                        ui.set_backup_status(format!("{}: {error}",
                            i18n::tr(ui.get_language_code().as_str(),"backup-import-error")).into());
                        return;
                    }
                };
                let imported = match backup::inspect_external(&filename) {
                    Ok(v) => v,
                    Err(error) => {
                        ui.set_backup_status(format!("{}: {error}",
                            i18n::tr(ui.get_language_code().as_str(),"backup-import-error")).into());
                        return;
                    }
                };
                let restored_runtime = match RuntimeState::new(imported.clone()) {
                    Ok(v) => v,
                    Err(error) => {
                        ui.set_backup_status(format!("{}: {error}",
                            i18n::tr(ui.get_language_code().as_str(),"backup-import-error")).into());
                        return;
                    }
                };
                let mut data = state.borrow_mut();
                let mut current = data.stored.clone();
                current.alarms = data.alarms.alarms.clone();
                if let Err(error) = backup::export(&path, &current) {
                    ui.set_backup_status(format!("{}: {error}",
                        i18n::tr(ui.get_language_code().as_str(),"backup-import-error")).into());
                    return;
                }
                match config::save(&path, &imported) {
                    Ok(()) => {
                        audio.stop();
                        *data = restored_runtime;
                        sync_appearance(&ui, &data.stored.settings);
                        sync_locale(&ui, &data.stored.settings.language);
                        sync_profiles(&ui, &data.stored);
                        ui.set_alarm_tone(data.stored.alarm_tone.as_label().into());
                        sync_alarm_ui(&ui, &data);
                        data.last_world_refresh_second = i64::MIN;
                        refresh_ui(&ui, &mut data);
                        sync_backups(&ui, &path);
                        ui.set_backup_status(format!("{}: {}",
                            i18n::tr(&data.stored.settings.language,"backup-restored-external"), filename.display()).into());
                    },
                    Err(error) => ui.set_backup_status(format!("{}: {error}",
                        i18n::tr(ui.get_language_code().as_str(),"backup-import-error")).into()),
                }
            }
        });
    }
    {
        let weak = ui.as_weak(); let state = state.clone(); let path = path.clone(); let audio = audio.clone();
        ui.on_restore_backup(move |name| {
            if let Some(ui) = weak.upgrade() {
                if !may_save { ui.set_backup_status(i18n::tr(&state.borrow().stored.settings.language,"backup-blocked").into()); return; }
                let imported = match backup::inspect(&path, name.as_str()) {
                    Ok(v) => v,
                    Err(error) => { ui.set_backup_status(format!("{}: {error}",i18n::tr(ui.get_language_code().as_str(),"backup-import-error")).into()); return; }
                };
                let restored_runtime = match RuntimeState::new(imported.clone()) {
                    Ok(v) => v,
                    Err(error) => { ui.set_backup_status(format!("{}: {error}",i18n::tr(ui.get_language_code().as_str(),"backup-import-error")).into()); return; }
                };
                let mut data = state.borrow_mut();
                let mut current = data.stored.clone();
                current.alarms = data.alarms.alarms.clone();
                if let Err(error) = backup::export(&path, &current) {
                    ui.set_backup_status(format!("{}: {error}",i18n::tr(ui.get_language_code().as_str(),"backup-import-error")).into()); return;
                }
                match config::save(&path, &imported) {
                    Ok(()) => {
                        audio.stop();
                        *data = restored_runtime;
                        sync_appearance(&ui, &data.stored.settings);
                        sync_locale(&ui, &data.stored.settings.language);
                        sync_profiles(&ui, &data.stored);
                        ui.set_alarm_tone(data.stored.alarm_tone.as_label().into());
                        sync_alarm_ui(&ui, &data);
                        data.last_world_refresh_second = i64::MIN;
                        refresh_ui(&ui, &mut data);
                        sync_backups(&ui, &path);
                        ui.set_backup_status(i18n::tr(&data.stored.settings.language,"backup-restored").into());
                    },
                    Err(error) => ui.set_backup_status(format!("{}: {error}",i18n::tr(ui.get_language_code().as_str(),"backup-import-error")).into()),
                }
            }
        });
    }

    let weak = ui.as_weak();
    let ticking_state = state.clone();
    let ticking_audio = audio.clone();
    #[cfg(all(windows, feature = "windows-installer"))]
    let ticking_files = audio_files.clone();
    let timer = Timer::default();
    let saver_started = Instant::now();
    let mut last_saver_position_second = u64::MAX;
    timer.start(TimerMode::Repeated, Duration::from_millis(100), move || {
        if let Some(ui) = weak.upgrade() {
            if ui.get_display_mode() == 1 {
                let elapsed = saver_started.elapsed().as_secs();
                if elapsed != last_saver_position_second {
                    let (dx, dy) = display::saver_drift(elapsed);
                    ui.set_saver_shift_x(dx);
                    ui.set_saver_shift_y(dy);
                    last_saver_position_second = elapsed;
                }
            }
            let mut state = ticking_state.borrow_mut();
            let event = poll_alert_events(&mut state);
            refresh_ui(&ui, &mut state);
            if let Some(is_timer) = event {
                #[cfg(all(windows, feature = "windows-installer"))]
                {
                    let files = ticking_files.borrow();
                    let filename = if is_timer { files.timer.clone() } else { files.alarm.clone() };
                    ticking_audio.play_with_file(state.stored.alarm_tone, filename);
                }
                #[cfg(not(all(windows, feature = "windows-installer")))]
                { let _ = is_timer; ticking_audio.play_until_stopped(state.stored.alarm_tone); }
            }
        }
    });
#[cfg(all(windows, feature = "windows-installer"))]
    if matches!(screensaver_mode, windows_screensaver::Mode::Configure) {
        ui.set_home_panel(2);
    }
    #[cfg(all(windows, feature = "windows-installer"))]
    if matches!(screensaver_mode, windows_screensaver::Mode::Screen) {
        ui.invoke_enter_display_mode(1);
    }
    // In .scr /s mode any real Windows input must close the whole process.
    // Two-second grace avoids spurious startup input; never saves settings.
    #[cfg(all(windows, feature = "windows-installer"))]
    let _saver_exit_timer = if matches!(screensaver_mode, windows_screensaver::Mode::Screen) {
        let weak_ui = ui.as_weak();
        let start = Instant::now();
        let baseline = Rc::new(RefCell::new(None::<u32>));
        let timer = Timer::default();
        timer.start(TimerMode::Repeated, Duration::from_millis(200), move || {
            if start.elapsed() < Duration::from_secs(2) { return; }
            if let Some(window) = weak_ui.upgrade() {
                if window.get_display_mode() != 1 {
                    let _ = slint::quit_event_loop();
                    return;
                }
                if let Some(now) = windows_screensaver::last_input_tick() {
                    let mut first = baseline.borrow_mut();
                    match *first {
                        Some(previous) if previous != now => {
                            let _ = slint::quit_event_loop();
                        }
                        None => { *first = Some(now); }
                        _ => {}
                    }
                }
            }
        });
        Some(timer)
    } else { None };
    #[cfg(all(windows, feature = "windows-installer"))]
    let result = if matches!(screensaver_mode, windows_screensaver::Mode::Normal) {
        match windows_tray::Tray::start() {
            Ok(_tray) => {
                // Only normal Installed mode hides to tray. Screensaver modes do not.
                ui.window().on_close_requested(|| slint::CloseRequestResponse::HideWindow);
                let weak = ui.as_weak();
                let tray_poll = Timer::default();
                tray_poll.start(TimerMode::Repeated, Duration::from_millis(150), move || {
                    match windows_tray::take_action() {
                        1 => { if let Some(window) = weak.upgrade() { let _ = window.show(); } }
                        2 => { let _ = slint::quit_event_loop(); }
                        _ => {}
                    }
                });
                ui.show()?;
                let status = slint::run_event_loop_until_quit();
                let _ = ui.hide();
                drop(tray_poll);
                status
            }
            Err(_) => ui.run(), // Safe fallback: never strand a hidden window.
        }
    } else { ui.run() };
    #[cfg(not(all(windows, feature = "windows-installer")))]
    let result = ui.run();
    display::keep_display_awake(false);

    result?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn clock_display_respects_user_settings() {
        let instant = Utc.with_ymd_and_hms(2026, 10, 13, 13, 5, 9).single().unwrap();
        let local = instant.with_timezone(&Local);
        let settings = ClockSettings::default();
        assert_eq!(format_time(local, &settings), local.format("%H:%M").to_string());
        assert_eq!(format_time(local, &ClockSettings { hour_24: false, show_seconds: false, ..settings }), local.format("%I:%M %p").to_string());
    }

    #[test]
    fn clock_seconds_do_not_change_main_fragment() {
        let instant = Utc.with_ymd_and_hms(2026, 10, 13, 13, 5, 9).single().unwrap();
        let now = instant.with_timezone(&Local);
        for hour_24 in [true, false] {
            let mut settings = ClockSettings { hour_24, show_seconds: false, ..ClockSettings::default() };
            let (before, suffix_before) = clock_face_parts(&now, &settings);
            settings.show_seconds = true;
            let (after, suffix_after) = clock_face_parts(&now, &settings);
            assert_eq!(before, after);
            assert!(suffix_after.contains(":09"));
            if hour_24 { assert!(suffix_before.is_empty()); }
            else { assert!(suffix_before.ends_with("PM") || suffix_before.ends_with("AM")); }
        }
    }

    #[test]
    fn theme_palette_independence() {
        let mut settings = ClockSettings::default();
        assert_eq!(active_colors(&settings), ("#2400D0", "#DAFF00", "#DAFF00"));
        set_active_color(&mut settings, "clock", "#123abc").unwrap();
        settings.dark_mode = false;
        assert_eq!(active_colors(&settings), ("#F2F4FF", "#2E18C4", "#2E18C4"));
        set_active_color(&mut settings, "clock", "#ff1020").unwrap();
        assert_eq!(settings.clock_color, "#123ABC");
        assert_eq!(settings.light_clock_color, "#FF1020");
        assert!(set_active_color(&mut settings,"unknown", "#ffffff").is_err());
    }

    #[test]
    fn appearance_color_defaults_match_original_look() {
        let s=ClockSettings::default();
        assert_eq!(config::rgb_from_hex(&s.background_color),Some((36,0,208)));
        assert_eq!(config::rgb_from_hex(&s.clock_color),Some((218,255,0)));
        let bright=adjusted_color("#DAFF00",100,100);
        assert_eq!((bright.red(),bright.green(),bright.blue()),(218,255,0));
    }

    #[test]
    fn each_legacy_date_format_is_supported() {
        let instant=Utc.with_ymd_and_hms(2026,10,13,12,0,0).single().unwrap();
        let now=instant.with_timezone(&Local);
        for fmt in config::DATE_FORMATS {
            let mut s=ClockSettings::default();
            s.date_format=(*fmt).into();
            assert!(!format_date(now.clone(),&s).is_empty());
        }
    }

    #[test]
    fn alarm_selectors_generate_exact_format() {
        assert_eq!(build_alarm_time("00", "00").unwrap(), "00:00");
        assert_eq!(build_alarm_time("07", "09").unwrap(), "07:09");
        assert_eq!(build_alarm_time("23", "59").unwrap(), "23:59");
    }

    #[test]
    fn alarm_selectors_reject_invalid_entries() {
        for (hour, minute) in [("24", "00"), ("23", "60"), ("", "15"),
                                ("7", "05"), ("07", "5"), ("-1", "10"),
                                ("aa", "12"), ("12", "1:")] {
            assert!(build_alarm_time(hour, minute).is_err());
        }
    }

    #[test]
    fn delete_middle_alarm_by_id_preserves_others() {
        let mut book = AlarmBook::default();
        book.add("06:30").unwrap();
        book.add("07:30").unwrap();
        book.add("08:30").unwrap();
        let to_remove = book.alarms[1].id;
        assert!(book.remove(to_remove));
        assert_eq!(book.alarms.len(), 2);
        assert_eq!(book.alarms[0].time, "06:30");
        assert_eq!(book.alarms[1].time, "08:30");
        assert!(!book.remove(to_remove));
    }

}
