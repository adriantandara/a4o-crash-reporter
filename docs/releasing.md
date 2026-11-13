# Publicarea unei versiuni

Cerințe: `gh` autentificat cu drept de scriere pe repo, Rust, cheia `~/.config/a4o-crash-reporter/release-primary.key`.

1. Crește versiunea în `Cargo.toml` și rulează `cargo build` ca să se actualizeze `Cargo.lock`. Fă commit și push pe `main`.
2. Creează și urcă tag-ul: `git tag v<versiune> && git push origin v<versiune>`. Workflow-ul `Release` construiește `.exe` și `.msi`, publică atestarea, scanează pe VirusTotal și creează un **draft** de release.
3. Testează installer-ul din draft pe un PC curat: instalare, prompt, tray, dezinstalare.
4. Rulează `tools/publish-release.sh <versiune>`. Scriptul:
   - descarcă draft-ul și verifică `SHA256SUMS.txt` și atestarea;
   - semnează `release.json` și îl urcă împreună cu `release.json.sig`;
   - te întreabă dacă publici release-ul.

   Când release-ul e semnat Authenticode, adaugă `--authenticode`.
5. Anunță versiunea pe Discord, cu schimbările și data de la care va fi disponibilă.
6. În panou, la Crash-uri → Versiuni aplicație, aprobă versiunea cu perioada de anunț. Abia după ea apare în aplicație și în `https://ucp.all4one.ro/crash-reporter/latest`.

## Retragerea unei versiuni

Panou → Crash-uri → Versiuni aplicație → Retrage. Versiunea nu se mai oferă, iar endpoint-ul public revine la versiunea aprobată anterior. Cine a instalat-o deja o păstrează. Repari problema și publici o versiune nouă.

## Cheile de semnare

- `release-primary.key`: cheia folosită la fiecare release. Păstrează o copie criptată în managerul de parole.
- `release-backup.key`: mut-o offline (stick USB, manager de parole) și șterge-o de pe PC. Cu ea semnezi doar dacă se pierde cheia principală.
- Dacă se pierde una dintre chei, publică o versiune semnată cu cealaltă, în care înlocuiești cheia pierdută în `src/config.rs` (`RELEASE_KEYS`) și în Guard (`src/guard/api/src/crash/crash-releases.service.ts`).
- Cheile nu ajung niciodată în repo, în GitHub Actions sau pe server.
