# Digital Clock Windows Portable

Izvorni kod za **Windows x64 Portable** digitalni sat izrađen u jeziku **Rust** s grafičkim sučeljem **Slint**. Za prikaz sučelja ne koristi .NET ni WebView2.

**Stabilno izdanje:** v0.3.8. Windows Portable aplikacija izrađena je u Rustu i Slintu; Android i Microsoft Store verzije zasebna su izdanja.

## Funkcije

- Sat s prilagodbom prikaza, fontova i boja
- Alarmi, odbrojavanje, štoperica i svjetski sat
- Način čuvara zaslona unutar aplikacije i Bedside način (nije Windows `.scr`)
- Jezici EN / HR / DE / IT / ES
- Prijenosne lokalne postavke te izvoz/uvoz sigurnosnih kopija

## Preuzimanje i pokretanje

Gotov Windows Portable ZIP dostupan je u [GitHub izdanju v0.3.8](https://github.com/Soldatix/Digital-Clock-Rust/releases/tag/v0.3.8). Ovaj repozitorij sadrži izvorni kod; GitHub izdanje sadrži datoteke za pokretanje.

Kad izdanje bude dostupno, raspakiraj cijeli ZIP u mapu i pokreni `digital-clock-rust-portable.exe`. Microsoftove DLL datoteke priložene u paketu moraju ostati pokraj EXE datoteke. Postavke i backup spremaju se lokalno u `Data/`.

## Kompilacija

Na Windows x64 potrebni su Rust MSVC toolchain i Microsoft C++ Build Tools:

```powershell
cargo test --locked
cargo build --release --locked
```

Stabilno izdanje v0.3.8 temelji se na testiranom RC3.2 kodu. Rust izvorne datoteke, `Cargo.toml`, `Cargo.lock`, ikone i fontovi ostali su **neizmijenjeni**. Verzija u Cargo paketu je `0.3.8`. **Ne tvrdimo** da je kompilacija bajtno reproducibilna.

## Licence

- Izvorni kod aplikacije: [MIT](LICENSE), kao što je navedeno u `Cargo.toml`.
- Priloženi fontovi: zasebne dozvole **SIL OFL 1.1** i obavijesti u [`LEGAL/Fonts/`](LEGAL/Fonts/).
- Rust ovisnosti, uključujući Slint, imaju svoje licence. Verzije su zapisane u `Cargo.lock`; licence za binarne ovisnosti prilažu se distribucijskom ZIP-u.
- Slint atribucija (*Made with Slint*) javno je dostupna na [Apps & Games stranici za instalaciju](https://appsandgames.org/downloads/digital-clock/Installation.html).

Dodatni podaci o provjeri: [SOURCE-PROVENANCE.md](SOURCE-PROVENANCE.md).
