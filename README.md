# ALL4ONE Crash Reporter

Aplicație Windows, opțională și open source, care trimite echipei ALL4ONE crash-urile FiveM pe care le iei pe serverul ALL4ONE. Cu ele găsim mai repede asset-urile, scripturile sau resursele care provoacă crash-uri.

## Cum funcționează

1. La prima pornire îți arată ce face și îți cere acordul. Fără acord nu face nimic și nu scrie nimic pe disc.
2. Rulează în fundal, cu o iconiță lângă ceas. Din meniul ei o pui pe pauză, o închizi sau îți retragi acordul.
3. Când apare fereastra de eroare FiveM, verifică întâi ultimul server pe care ai jucat. Dacă nu e ALL4ONE, nu face nimic.
4. Dacă e ALL4ONE, apasă „Save information”, așteaptă arhiva salvată de FiveM în Descărcări și închide fereastra de eroare.
5. Din arhivă trimite doar:
   - fișierul `.dmp`;
   - `crashometry.json`, fără Rockstar ID;
   - jurnalul CitizenFX, din care elimină numele și calea contului tău de Windows.

   Restul arhivei (`launcher.log`, `socialclub.log` etc.) rămâne doar la tine.
6. Serverul ALL4ONE primește raportul doar dacă tocmai te-ai deconectat de pe server cu un crash, de pe aceeași adresă IP. Detaliile sunt în [docs/protocol.md](docs/protocol.md).

Fiecare raport trimis apare în **Istoricul rapoartelor**, din meniul aplicației.

## Ce nu face

- Nu se injectează în FiveM sau GTA V și nu citește memoria jocului. Dump-ul îl creează FiveM, nu aplicația.
- Nu modifică fișierele jocului.
- Nu cere drepturi de administrator. Se instalează doar pentru utilizatorul curent.
- Nu citește alte fișiere decât `FiveM.app\data\cache\crashometry` și arhivele `CfxCrashDump_*.zip` create după crash.
- Nu face polling: urmărește evenimentele Windows (pornirea FiveM, apariția ferestrei de eroare) și altfel stă inactivă. Consumul de procesor este practic 0, iar memoria folosită e de câțiva MB.

## Încredere și verificare

- **Cod sursă:** tot codul e aici, scris în Rust, cu puține dependențe: `windows` (Microsoft), `ed25519-dalek`, `sha2`, `miniz_oxide`, `serde_json`. Conexiunea HTTPS folosește WinHTTP din Windows.
- **Build public:** fiecare versiune e compilată de GitHub Actions din tag-ul publicat. Pentru fiecare fișier publicăm o atestare de proveniență (`gh attestation verify <fisier> --repo adriantandara/a4o-crash-reporter`).
- **VirusTotal:** fiecare release are linkul de scanare pentru `.exe` și `.msi`.
- **Manifest semnat:** fiecare versiune are un `release.json` semnat cu o cheie offline. Aplicația și serverul îl verifică înainte să ofere versiunea.
- **Semnătură digitală:** release-urile vor fi semnate prin SignPath Foundation, după aprobarea proiectului.
- **Sume de control:** `SHA256SUMS.txt` în fiecare release.

## Politica de semnare a codului

Semnarea gratuită a codului: [SignPath.io](https://about.signpath.io), certificat oferit de [SignPath Foundation](https://signpath.org).

- Autori și aprobatori: [adriantandara](https://github.com/adriantandara) (owner ALL4ONE). Fiecare release e aprobat manual.
- Se semnează doar fișierele construite de GitHub Actions din codul acestui repository, la un tag `v*`.
- Aplicația trimite date doar echipei ALL4ONE și doar cu acordul utilizatorului; detalii în [PRIVACY.md](PRIVACY.md).

## Instalare și dezinstalare

Descarci `ALL4ONE-Crash-Reporter-<versiune>.msi` din [Releases](https://github.com/adriantandara/a4o-crash-reporter/releases) și îl rulezi. Se instalează în `%LOCALAPPDATA%\Programs\ALL4ONE Crash Reporter`.

Dezinstalarea din *Setări → Aplicații* oprește aplicația și șterge:
- programul;
- pornirea automată;
- datele locale din `%LOCALAPPDATA%\ALL4ONE\CrashReporter` (setările, cheia PC-ului, istoricul și jurnalul).

## Actualizări

Actualizăm aplicația doar când e nevoie și anunțăm fiecare versiune înainte să apară. Aplicația te anunță când apare o versiune nouă, dar nu instalează nimic fără acordul tău; verificarea o poți opri din meniu. Înainte de instalare, fiecare versiune e verificată cu semnătura noastră și cu SHA-256. Detalii: [UPDATES.md](UPDATES.md). Pentru echipă: [docs/releasing.md](docs/releasing.md).

## Datele tale

Vezi [PRIVACY.md](PRIVACY.md).

## Compilare

```
cargo build --release
```

Pe Linux sau WSL: `cargo xwin build --release --target x86_64-pc-windows-msvc`. Installer-ul se construiește cu WiX 5 din `installer/` (vezi `.github/workflows/release.yml`).

## Licență

[MIT](LICENSE)
