# Quickstart: Cerca i tailing

Guia de validació manual d'extrem a extrem. No conté codi d'implementació: només com
construir, executar i comprovar que el comportament coincideix amb els criteris d'èxit de
`spec.md`.

## Prerequisits

- Rust estable instal·lat (`rustup`, `cargo`).
- Un terminal per generar/modificar fitxers de prova mentre RealttyLog està obert.

## Posada en marxa

```sh
cargo build --release
./target/release/realttylog
```

## Escenari A — Cercar a un directori de logs partits (SC-001, SC-008)

1. Crea un directori amb diversos fitxers, simulant logs partits per dia, i posa un text
   concret en un de sol:
   ```sh
   mkdir -p /tmp/logs
   for d in 13 14 15; do echo "línia normal del dia $d" > /tmp/logs/app-2026-08-$d.log; done
   echo "ERROR: connexió refusada amb la base de dades" >> /tmp/logs/app-2026-08-14.log
   ```
2. Obre el directori `/tmp/logs` amb RealttyLog.
3. Cerca `connexió refusada`.
4. **Resultat esperat**: apareix un únic resultat, assenyalant `app-2026-08-14.log`, en menys
   de 30 segons des que has obert el directori (SC-001), sense haver-hi hagut d'endevinar
   quin fitxer era.

## Escenari B — Saltar del resultat al fitxer i la línia (User Story 2)

1. Amb l'escenari A fet, clica el resultat de la cerca.
2. **Resultat esperat**: s'obre `app-2026-08-14.log` amb la línia de l'error visible, i les
   línies del voltant com a context, sense haver-hi de desplaçar-se manualment.
3. Torna a la llista de resultats.
4. **Resultat esperat**: hi tornes sense haver de repetir la cerca.

## Escenari C — Seguiment en directe (SC-003)

1. Crea un fitxer buit: `touch /tmp/demo.log`.
2. En un terminal, afegeix-hi una línia cada 200 ms:
   ```sh
   while true; do echo "$(date +%T) línia de prova" >> /tmp/demo.log; sleep 0.2; done
   ```
3. Obre `/tmp/demo.log` directament amb RealttyLog (sense passar per cerca).
4. **Resultat esperat**: cada línia nova apareix a la finestra en menys d'1 segon des que
   s'escriu, i la vista es manté al final (autoscroll actiu).

## Escenari D — Pausa i represa de l'autoscroll (User Story 4)

1. Amb l'escenari C encara en marxa, desplaça la vista de RealttyLog cap amunt.
2. **Resultat esperat**: la vista es queda quieta encara que arribin línies noves; l'estat
   visible passa a "pausat".
3. Activa "tornar al directe" (botó o desplaçament fins al final).
4. **Resultat esperat**: la vista salta a l'última línia i l'estat torna a "en directe".

## Escenari E — Rotació de log (SC-007)

1. Amb RealttyLog seguint `/tmp/demo.log` (escenari C en marxa), en un altre terminal:
   ```sh
   : > /tmp/demo.log   # truncament a mida zero
   ```
2. **Resultat esperat**: RealttyLog continua mostrant les línies noves que s'hi afegeixen
   després del truncament, sense error ni necessitat de tornar a obrir el fitxer.
3. Repeteix substituint el truncament per un reemplaçament (`mv demo.log.old && touch
   demo.log` amb el mateix camí) i comprova el mateix resultat.

## Escenari F — Fitxers grans, en cerca i en directe (SC-002, SC-004, SC-005, SC-009)

1. Genera uns quants fitxers d'uns 5 GB cadascun dins un directori, amb un text concret
   només en un d'ells:
   ```sh
   mkdir -p /tmp/logs-grans
   yes "línia de prova per omplir espai" | head -c 5G > /tmp/logs-grans/a.log
   yes "línia de prova per omplir espai" | head -c 5G > /tmp/logs-grans/b.log
   { yes "línia de prova per omplir espai" | head -c 5G; echo "ERROR agulla al paller"; } > /tmp/logs-grans/c.log
   ```
2. Obre `/tmp/logs-grans` amb RealttyLog i cerca `agulla al paller`.
   **Resultat esperat**: el primer resultat apareix en menys de 3 segons (SC-002).
3. Obre `/tmp/logs-grans/a.log` directament (sense cercar) i mesura el temps fins que es veu
   el final del fitxer.
   **Resultat esperat**: menys de 2 segons (SC-004).
4. Deixa'l seguint en directe mentre hi afegeixes línies noves de tant en tant (com a
   l'escenari C) durant uns 30 minuts, i mesura el consum de memòria del procés (Gestor de
   tasques a Windows, `top`/`htop` a Linux).
   **Resultat esperat**: es manté per sota d'uns 20 MB per sobre del consum inicial (SC-005).
5. Desplaça't fins al principi del fitxer.
   **Resultat esperat**: hi veus el contingut real, no un buit ni un avís de límit (SC-009).

## Neteja

```sh
rm -rf /tmp/demo.log /tmp/logs /tmp/logs-grans
```
