# Securitate

## Raportarea unei probleme

Nu deschide un issue public pentru vulnerabilități. Scrie-ne pe Discord-ul ALL4ONE, la tichete, sau folosește [GitHub Security Advisories](https://github.com/adriantandara/a4o-crash-reporter/security/advisories/new). Răspundem în 72 de ore.

## Modelul de securitate

Aplicația rulează pe PC-ul jucătorului, deci poate fi decompilată sau modificată. Nu are niciun secret care să merite extras. Protecția stă pe server:

- **Bilet de crash:** serverul de joc creează un bilet doar când un jucător se deconectează cu motivul `Game crashed: ...`. Un raport este acceptat doar dacă se potrivesc:
  - adresa IP;
  - fereastra de timp (30 de minute);
  - serverul.

  Fiecare bilet poate fi folosit o singură dată.
- **Cheie per PC:** aplicația generează o cheie Ed25519, păstrată criptat cu DPAPI. Fiecare cerere e semnată și are timestamp și nonce, ca să nu poată fi refolosită. Serverul poate bloca o cheie.
- **Validare strictă:**
  - dump-ul trebuie să înceapă cu `MDMP`;
  - mărimea și hash-ul SHA-256 trebuie să fie cele declarate;
  - fiecare câmp are o limită de mărime.

  Fișierele se salvează sub nume generate de server și nu sunt niciodată executate sau parsate de API.
- **Limite:** rapoarte pe IP, pe cheie și pe cont.

## Actualizări

O actualizare ajunge la jucători numai dacă trece de toate aceste verificări:
- build-ul public, cu atestare de proveniență;
- verificarea manuală a draft-ului;
- manifestul semnat Ed25519 cu o cheie offline (cheile publice sunt în aplicație);
- aprobarea în panoul ALL4ONE, cu perioadă de anunț.

Înainte să pornească installer-ul, aplicația verifică semnătura manifestului, mărimea și SHA-256. Nu instalează nimic fără acordul jucătorului. Detalii: [UPDATES.md](UPDATES.md).

Ce poate face cineva cu aplicația modificată: să trimită pentru propriul crash, de pe propriul IP, un fișier care trece validarea. Atât.

Protocolul complet: [docs/protocol.md](docs/protocol.md).
