# Politica de actualizări

## Pe scurt

- **Actualizăm doar când e nevoie:** o problemă de securitate, un bug care împiedică trimiterea rapoartelor sau o schimbare la FiveM care strică aplicația. Nu adăugăm funcții noi doar ca să existe o versiune nouă.
- **Fiecare versiune e anunțată înainte** pe Discord-ul ALL4ONE, cu lista schimbărilor. În aplicație și pe all4one.ro apare abia după perioada de anunț, de obicei 2 zile. Singura excepție este o problemă de securitate urgentă: atunci anunțul și versiunea apar în aceeași zi.
- **Nu ești obligat să actualizezi.** Aplicația doar te anunță că există o versiune nouă. Tu alegi „Instalează” sau „Ignoră”, iar verificarea automată o poți opri din meniu. Aplicația nu instalează nimic singură.
- **O versiune veche nu se blochează.** Funcționează cât timp rămâne compatibilă cu serverul. Dacă la un moment dat nu mai poate trimite rapoarte, îți spune asta; nu se actualizează singură.

## Ce date trimite verificarea

La 2 minute după pornire și apoi o dată la 12 ore, aplicația cere `https://ucp.all4one.ro/crash-reporter/latest`. Cererea nu conține date despre tine. Serverul vede doar adresa IP și versiunea aplicației (din User-Agent). Installer-ul se descarcă direct de pe GitHub.

Verificarea se oprește din meniu: „Caută actualizări automat”.

## Cum ne asigurăm că o actualizare e a noastră

O versiune trece prin patru verificări înainte să ajungă la tine:

1. **Build public.** GitHub Actions compilează versiunea din codul publicat la tag-ul respectiv și publică o atestare de proveniență pentru `.exe` și `.msi`. Release-ul pornește ca draft și nu e vizibil public.
2. **Verificarea echipei.** Înainte de publicare, verificăm draft-ul: sumele `SHA256SUMS.txt`, atestarea de build și scanarea VirusTotal.
3. **Manifest semnat.** Fișierul `release.json` conține pentru fiecare installer numele, mărimea și SHA-256. Îl semnăm Ed25519 cu o cheie care nu stă nici pe GitHub, nici pe server. Cheile publice sunt incluse în aplicație:
   - principală: `XovA_mOeVHqYFtCC7zEbm6gzMKfaRnH_t5EAXw_c3_g`
   - de rezervă: `X3iryo3lFBVLuH7V_e5L6ZD3Ngd_06_Oy2P0JCB9mhc`
4. **Aprobarea în panoul ALL4ONE.** Serverul verifică din nou semnătura și oferă versiunea abia după aprobare și după perioada de anunț.

Înainte să pornească installer-ul, aplicația verifică și ea semnătura manifestului, numele fișierului, mărimea exactă și SHA-256. Dacă versiunea e semnată digital (Authenticode), verifică și semnătura Windows. Dacă oricare verificare eșuează, fișierul se șterge și nu se instalează nimic.

Așa, un cont GitHub compromis nu ajunge să publice o actualizare, și nici un server compromis singur. Pentru asta ar fi nevoie de cheia de semnare, care stă offline.

## Instalare manuală

Poți descărca oricând versiunea curentă de pe [Releases](https://github.com/adriantandara/a4o-crash-reporter/releases) sau de pe pagina de pe all4one.ro. Ca să verifici un fișier descărcat:

```
gh attestation verify ALL4ONE-Crash-Reporter-<versiune>.msi --repo adriantandara/a4o-crash-reporter
sha256sum -c SHA256SUMS.txt
```
