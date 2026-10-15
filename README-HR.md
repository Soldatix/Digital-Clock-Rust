# Digital Clock Rust Portable

Izvorni kod za **Windows x64 Portable** digitalni sat izrađen u jeziku **Rust** s grafičkim sučeljem **Slint**. Za prikaz sučelja ne koristi .NET ni WebView2.

**Trenutačno stanje:** v0.3.8 · RC3.2 kandidat za izdanje. Ovo je zasebna Rust verzija i ne zamjenjuje postojeću .NET ili Android aplikaciju.

## Funkcije

- Sat s prilagodbom prikaza, fontova i boja
- Alarmi, odbrojavanje, štoperica i svjetski sat
- Način čuvara zaslona unutar aplikacije i Bedside način (nije Windows `.scr`)
- Jezici EN / HR / DE / IT / ES
- Prijenosne lokalne postavke te izvoz/uvoz sigurnosnih kopija

## Preuzimanje i pokretanje

Službeni Windows ZIP bit će dodan u GitHub Release **tek nakon završnog odobrenja objave**. Trenutačno repozitorij sadrži **izvorni kod**, a ne objavljeni instalacijski paket.

Kad izdanje bude dostupno, raspakiraj cijeli ZIP u mapu i pokreni `digital-clock-rust-portable.exe`. Microsoftove DLL datoteke priložene u paketu moraju ostati pokraj EXE datoteke. Postavke i backup spremaju se lokalno u `Data/`.

## Kompilacija

Na Windows x64 potrebni su Rust MSVC toolchain i Microsoft C++ Build Tools:

```powershell
cargo test --locked
cargo build --release --locked
```

Ovaj izvorni paket čuva **neizmijenjene Cargo.toml, Cargo.lock, Rust datoteke, ikone i fontove** iz provjerenog RC3.2 arhiva. Cargo verzija aplikacije je `0.3.8`. Funkcionalni testovi na Windowsu prethodno su provedeni; **ne tvrdimo** da je kompilacija bajtno reproducibilna.

## Licence

- Izvorni kod aplikacije: [MIT](LICENSE), kao što je navedeno u `Cargo.toml`.
- Priloženi fontovi: zasebne dozvole **SIL OFL 1.1** i obavijesti u [`LEGAL/Fonts/`](LEGAL/Fonts/).
- Rust ovisnosti, uključujući Slint, imaju svoje licence. Verzije su zapisane u `Cargo.lock`; licence za binarne ovisnosti prilažu se distribucijskom ZIP-u.
- Slint atribucija (*Made with Slint*) javno je dostupna na [Apps & Games stranici za instalaciju](https://appsandgames.org/downloads/digital-clock/Installation.html).

Dodatni podaci o provjeri: [SOURCE-PROVENANCE.md](SOURCE-PROVENANCE.md).
