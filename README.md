# Digital Clock Rust Portable

Native **Windows x64 portable** desktop digital clock built with **Rust** and **Slint** (no .NET or WebView2 UI dependency).

**Current source:** v0.3.8 · RC3.2 release candidate. The standalone Rust edition is distinct from the existing Digital Clock .NET and Android editions.

## Features

- Clock with adjustable display settings and font/color options
- Alarms, countdown timer, stopwatch, world clock
- In-app screensaver and bedside mode (not a Windows `.scr` screensaver)
- EN / HR / DE / IT / ES user interface languages
- Portable local settings and backup/restore

## Download and usage

The official Windows ZIP will be attached to a GitHub Release **only after final release approval**. This repository currently contains **source code**, not an installer or a published binary.

When a release becomes available, extract the full ZIP to a folder and start `digital-clock-rust-portable.exe`. Keep bundled runtime DLLs beside the EXE. User settings and backups are stored alongside the portable app under `Data/`; do not run it directly from the ZIP.

## Building from source

On Windows x64, install an appropriate Rust MSVC toolchain and Microsoft C++ Build Tools, then run:

```powershell
cargo test --locked
cargo build --release --locked
```

This source snapshot retains the exact reviewed **Cargo.toml, Cargo.lock, Rust sources, icons, and fonts** from the RC3.2 review archive. Its Rust package version is `0.3.8`. It was built and functionally checked on Windows prior to this repository snapshot, but byte-identical reproducible builds are **not claimed**.

## Licensing and attribution

- Application source: MIT license (see [LICENSE](LICENSE) and `Cargo.toml`).
- Bundled font files: individual **SIL OFL 1.1** texts and author notices in [`LEGAL/Fonts/`](LEGAL/Fonts/).
- Rust dependencies, including Slint, have their **own licenses**. `Cargo.lock` pins dependency versions; the binary release package carries the corresponding license notices.
- Built with [Slint](https://slint.dev/). Public *Made with Slint* attribution is provided on the [Apps & Games installation page](https://appsandgames.org/downloads/digital-clock/Installation.html).

See [SOURCE-PROVENANCE.md](SOURCE-PROVENANCE.md) for checksums and the scope of this source snapshot.

## Project

[Apps & Games](https://appsandgames.org/) · Digital Clock Rust Portable · [Source repository](https://github.com/Soldatix/Digital-Clock-Rust)
