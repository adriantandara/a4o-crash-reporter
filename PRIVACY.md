# Politica de confidențialitate – ALL4ONE Crash Reporter

Ultima actualizare: 13 noiembrie 2026 · versiunea acordului: 1

## Cine prelucrează datele

ALL4ONE Romania, comunitatea care administrează serverul FiveM ALL4ONE. Contact: Discord-ul ALL4ONE, secțiunea de tichete.

## Ce date colectăm și de ce

Doar după ce accepți în aplicație, și doar pentru crash-urile de pe serverul ALL4ONE:

| Ce | Ce conține | De ce |
|---|---|---|
| Fișierul `.dmp` (minidump) creat de FiveM | O parte din memoria jocului din momentul crash-ului: module încărcate, fire de execuție, stive. Poate include numele tău din joc, mesaje recente din joc, identificatori de sesiune FiveM și calea folderului tău de Windows în numele modulelor. | Să aflăm ce a provocat crash-ul. |
| `crashometry.json` | Semnătura crash-ului, placa video, ultimul server, starea OneSync. **Rockstar ID-ul este eliminat înainte de trimitere.** | Gruparea crash-urilor identice. |
| Jurnalul CitizenFX al sesiunii | Mesajele FiveM din sesiune: resursele încărcate, erorile. **Numele și calea contului tău de Windows sunt eliminate înainte de trimitere.** | Găsirea resursei sau a asset-ului implicat. |
| Adresa IP și cheia publică a aplicației | Adresa de la care trimiți și o cheie generată pe PC-ul tău | Legarea raportului de deconectarea ta de pe server și oprirea abuzurilor. |

Raportul se leagă de contul tău de pe serverul ALL4ONE, ca să putem vedea dacă un crash se repetă la același jucător și să te putem contacta pentru detalii.

**Nu colectăm:** fișiere personale, alte fișiere din Descărcări, `launcher.log`, `socialclub.log`, parole, crash-uri de pe alte servere.

## Verificarea actualizărilor

Dacă nu oprești opțiunea din meniu, aplicația verifică o dată la 12 ore dacă există o versiune nouă: o cerere la `ucp.all4one.ro`, fără date despre tine. Serverul vede doar adresa IP și versiunea aplicației și nu le salvează legat de contul tău. Installer-ul se descarcă de pe GitHub. Detalii: [UPDATES.md](UPDATES.md).

## Temeiul legal

Consimțământul tău (art. 6 alin. (1) lit. a) din GDPR), pe care îl dai în aplicație și îl poți retrage oricând.

## Cât timp păstrăm datele

Rapoartele se șterg automat după **90 de zile**. Statisticile agregate (de exemplu „semnătura X a apărut de 12 ori”) nu conțin date personale și pot fi păstrate mai mult.

## Cine are acces

Doar echipa de dezvoltare ALL4ONE, din panoul intern. Datele nu se vând și nu se transmit către terți. Serverele sunt găzduite în Uniunea Europeană.

FiveM trimite, separat și independent de această aplicație, propriul raport de crash către Cfx.re, conform politicii lor.

## Drepturile tale

- **Retragerea acordului:** din meniul aplicației, „Retrage acordul și șterge datele”. Aplicația se oprește, nu mai pornește cu Windows, își șterge datele locale și cere serverului să uite cheia acestui PC.
- **Acces, ștergere, rectificare:** cere pe Discord-ul ALL4ONE rapoartele trimise de tine sau ștergerea lor. Răspundem în cel mult 30 de zile.
- **Plângere:** te poți adresa Autorității Naționale de Supraveghere a Prelucrării Datelor cu Caracter Personal (ANSPDCP), dataprotection.ro.

## Date păstrate pe PC-ul tău

În `%LOCALAPPDATA%\ALL4ONE\CrashReporter`:
- `settings.json`: acordul și opțiunile;
- `device.key`: cheia aplicației, criptată cu Windows DPAPI pentru contul tău;
- `rapoarte.txt`: istoricul rapoartelor;
- `reporter.log`: jurnalul aplicației, de cel mult 256 KB.

Folderul se șterge la retragerea acordului și la dezinstalare.

## Vârstă

Dacă ai sub 16 ani, folosește aplicația doar cu acordul unui părinte.

## Modificări

Dacă se schimbă ce colectăm, aplicația îți cere din nou acordul.
