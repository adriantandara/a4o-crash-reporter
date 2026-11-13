# Protocolul de raportare

Baza: `https://guard.all4one.ro/api/crash/v1`. Doar HTTPS, TLS 1.2 sau 1.3.

## Biletul de crash (pe serverul de joc)

La `playerDropped`, dacă motivul începe cu `Game crashed:`, serverul de joc salvează un bilet cu:
- `user_id` și license;
- adresa IP a jucătorului;
- serverul (`ip:port`);
- motivul complet (care conține semnătura crash-ului);
- ora.

Biletul expiră după 30 de minute și poate fi folosit o singură dată.

## Semnarea cererilor

Fiecare cerere are aceste antete:

| Antet | Valoare |
|---|---|
| `X-A4O-Key` | cheia publică Ed25519 (32 de octeți, base64url fără padding) |
| `X-A4O-Timestamp` | secunde Unix |
| `X-A4O-Nonce` | 16 octeți aleatori, hex |
| `X-A4O-Signature` | semnătura Ed25519 (base64url) a textului de mai jos |
| `X-A4O-Version` | versiunea aplicației |

Textul semnat, cu linii separate prin `\n`:

```
A4OCR1
<METODA>
<cale completă, ex. /api/crash/v1/claim>
<timestamp>
<nonce>
<sha256 hex al corpului cererii>
```

Serverul:
- respinge timestamp-urile mai vechi sau mai noi cu peste 5 minute față de ora lui;
- respinge un nonce deja văzut pentru aceeași cheie în ultimele 10 minute;
- respinge semnăturile invalide (`401`);
- respinge cheile blocate (`403`).

## 1. `POST /claim`

Corp JSON:

```json
{
  "app_version": "0.1.0",
  "server": "82.22.174.206:30120",
  "crash_hash": "fivem.exe+947655",
  "crash_key": "west-july-three",
  "crashed_at": 1794510651,
  "dump_size": 10125126,
  "dump_sha256": "b599…",
  "has_log": true,
  "metadata": { "crashometry_gpu_name": "…", "…": "…" }
}
```

Serverul:
1. Verifică semnătura și limitele de rata: pe IP, pe cheie și global.
2. Caută cel mai recent bilet nefolosit cu aceeași adresă IP și același `server`, din ultimele 30 de minute.
   - Dacă nu există, răspunde `404`.
   - Dacă motivul biletului conține `crash_hash` sau `crash_key`, raportul primește `match = exact`, altfel `match = ip_time`.
3. Respinge `dump_size` peste 96 MB (`413`).
4. Dacă `dump_sha256` a mai fost primit de la același cont, răspunde `409`.
5. Marchează biletul ca folosit.
6. Creează raportul cu statusul `pending`, legat de `user_id`, cheie, IP și metadate. Răspunde `201`:

```json
{ "report_id": "c8f1…", "upload_token": "…" }
```

Tokenul este valabil 15 minute și doar pentru acest raport.

## 2. `PUT /reports/{report_id}/dump`

- Antete: `Authorization: Bearer <upload_token>` și `Content-Type: application/zlib`. Cererea este semnată, ca toate celelalte.
- Corp: minidump-ul comprimat zlib.

Serverul:
- dezarhivează în flux și se oprește la `dump_size`;
- verifică dimensiunea exactă, prefixul `MDMP` și SHA-256 egal cu `dump_sha256`;
- salvează dump-ul într-un director fără drept de execuție, sub un nume generat.

Răspunde `204`.

## 3. `PUT /reports/{report_id}/log` (opțional)

La fel, pentru jurnalul CitizenFX, cu limita de 8 MB după dezarhivare. Conținutul trebuie să fie text UTF-8.

## 4. `POST /reports/{report_id}/complete`

Trece raportul în statusul `received` dacă dump-ul a fost primit. Răspunde `200`.

## `POST /device/forget`

Este semnat. Serverul marchează cheia ca retrasă și dezleagă cheia de cont. Rapoartele rămân până la ștergerea automată.

## Coduri de răspuns

| Cod | Sens | Aplicația |
|---|---|---|
| 401 | semnătură sau timestamp invalid | nu reîncearcă |
| 403 | cheie blocată | nu reîncearcă |
| 404 | niciun bilet potrivit | nu reîncearcă |
| 409 | dump deja primit | marchează ca „deja trimis” |
| 410 | token expirat | nu reîncearcă |
| 413 | prea mare | nu reîncearcă |
| 429 | limită depășită | nu reîncearcă |
| 5xx / rețea | eroare temporară | reîncearcă după 15 s și apoi după 60 s |

## Păstrare

Un job zilnic șterge rapoartele (fișiere și rânduri) mai vechi de 90 de zile și rapoartele `pending` mai vechi de o oră.
