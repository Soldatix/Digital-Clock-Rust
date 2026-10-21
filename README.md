# Digital Clock Windows Portable

Native **Windows x64 portable** desktop digital clock built with **Rust** and **Slint** (no .NET or WebView2 UI dependency).

**Stable release:** v0.3.8. This Windows Portable edition uses Rust and Slint; the Android and Microsoft Store editions are separate.

## Features

- Clock with adjustable display settings and font/color options
- Alarms, countdown timer, stopwatch, world clock
- In-app screensaver and bedside mode (not a Windows `.scr` screensaver)
- EN / HR / DE / IT / ES user interface languages
- Portable local settings and backup/restore

## Download and usage

Download the current Windows Portable ZIP from the [v0.3.8 release](https://github.com/Soldatix/Digital-Clock-Rust/releases/tag/v0.3.8). This repository contains the source code; the GitHub release contains the ready-to-run Windows ZIP.

When a release becomes available, extract the full ZIP to a folder and start `digital-clock-rust-portable.exe`. Keep bundled runtime DLLs beside the EXE. User settings and backups are stored alongside the portable app under `Data/`; do not run it directly from the ZIP.

## Building from source

On Windows x64, install an appropriate Rust MSVC toolchain and Microsoft C++ Build Tools, then run:

```powershell
cargo test --locked
cargo build --release --locked
```

The stable v0.3.8 release promotes the functionally tested RC3.2 code without modifying the Rust source, `Cargo.toml`, `Cargo.lock`, icons or fonts. The package version is `0.3.8`. Byte-identical reproducible builds are **not claimed**.

## Licensing and attribution

- Application source: MIT license (see [LICENSE](LICENSE) and `Cargo.toml`).
- Bundled font files: individual **SIL OFL 1.1** texts and author notices in [`LEGAL/Fonts/`](LEGAL/Fonts/).
- Rust dependencies, including Slint, have their **own licenses**. `Cargo.lock` pins dependency versions; the binary release package carries the corresponding license notices.
- Built with [Slint](https://slint.dev/). Public *Made with Slint* attribution is provided on the [Apps & Games installation page](https://appsandgames.org/downloads/digital-clock/Installation.html).

See [SOURCE-PROVENANCE.md](SOURCE-PROVENANCE.md) for checksums and the scope of this source snapshot.

### Optional Apps & Games credit

If you use our source code to create your own version, we'd appreciate a mention in its About / Info section:

**Based on Digital Clock Windows Portable by Apps & Games — [appsandgames.org](https://appsandgames.org/)**

## Project

[Apps & Games](https://appsandgames.org/) · Digital Clock Windows Portable · [Source repository](https://github.com/Soldatix/Digital-Clock-Rust)
