# Publicarea unei versiuni

Cerințe: `gh` autentificat cu drept de scriere pe repo, Rust și cheia privată de release. Echipa ALL4ONE păstrează cheia criptată (ansible-vault) în repo-ul intern. Scriptul o primește prin `A4O_RELEASE_KEY`; implicit o caută în `~/.config/a4o-crash-reporter/release-primary.key`.

1. Crește versiunea în `Cargo.toml` și rulează `cargo build` ca să se actualizeze `Cargo.lock`. Fă commit și push pe `main`.
2. Creează și urcă tag-ul: `git tag v<versiune> && git push origin v<versiune>`. Workflow-ul `Release` construiește `.exe` și `.msi`, publică atestarea, scanează pe VirusTotal și creează un **draft** de release.
3. Testează installer-ul din draft pe un PC curat: instalare, prompt, tray, dezinstalare.
4. Rulează `tools/publish-release.sh <versiune>`. Scriptul:
   - descarcă draft-ul și verifică `SHA256SUMS.txt` și atestarea;
   - semnează `release.json` și îl urcă împreună cu `release.json.sig`;
   - te întreabă dacă publici release-ul.

   Când semnarea SignPath e activă (variabila `SIGNPATH_ORGANIZATION_ID` din repo), manifestul cere automat și semnătura Authenticode.
5. Anunță versiunea pe Discord, cu schimbările și data de la care va fi disponibilă.
6. În panou, la Crash-uri → Versiuni aplicație, aprobă versiunea cu perioada de anunț. Abia după ea apare în aplicație și în `https://ucp.all4one.ro/crash-reporter/latest`.

## Secrete și variabile în GitHub

| Nume | Tip | Rol |
| --- | --- | --- |
| `VT_API_KEY` | secret | scanarea VirusTotal; fără el pasul e sărit |
| `SIGNPATH_API_TOKEN` | secret | token-ul CI din SignPath |
| `SIGNPATH_ORGANIZATION_ID` | variabilă | activează semnarea; fără ea release-ul iese nesemnat Authenticode |

În SignPath: proiectul `a4o-crash-reporter`, politica `release-signing`, configurațiile de artifact `exe` și `msi` din `.signpath/artifact-configurations/`.

## Retragerea unei versiuni

Panou → Crash-uri → Versiuni aplicație → Retrage. Versiunea nu se mai oferă, iar endpoint-ul public revine la versiunea aprobată anterior. Cine a instalat-o deja o păstrează. Repari problema și publici o versiune nouă.

## Cheile de semnare

- Există două chei Ed25519: principală și de rezervă. Cheile publice sunt în `src/config.rs` (`RELEASE_KEYS`) și în Guard.
- Cheile private nu ajung niciodată în acest repo, în GitHub Actions sau pe server. Echipa le păstrează criptat, cu acces limitat.
- Cu cheia de rezervă semnezi doar dacă se pierde cea principală. În acel caz publici o versiune semnată cu cheia de rezervă, în care înlocuiești cheia pierdută în `RELEASE_KEYS` și în Guard.
