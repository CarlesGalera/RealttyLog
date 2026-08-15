# Quickstart: Tailing bàsic

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

## Escenari A — Seguiment en directe (SC-001)

1. Crea un fitxer buit: `touch /tmp/demo.log`.
2. En un terminal, afegeix-hi una línia cada 200 ms:
   ```sh
   while true; do echo "$(date +%T) línia de prova" >> /tmp/demo.log; sleep 0.2; done
   ```
3. Obre `/tmp/demo.log` amb RealttyLog.
4. **Resultat esperat**: cada línia nova apareix a la finestra en menys d'1 segon des que
   s'escriu, i la vista es manté al final (autoscroll actiu).

## Escenari B — Pausa i represa de l'autoscroll (User Story 2)

1. Amb l'escenari A encara en marxa, desplaça la vista de RealttyLog cap amunt.
2. **Resultat esperat**: la vista es queda quieta encara que arribin línies noves; l'estat
   visible passa a "pausat".
3. Activa "tornar al directe" (botó o desplaçament fins al final).
4. **Resultat esperat**: la vista salta a l'última línia i l'estat torna a "en directe".

## Escenari C — Rotació de log (SC-005)

1. Amb RealttyLog seguint `/tmp/demo.log` (escenari A en marxa), en un altre terminal:
   ```sh
   : > /tmp/demo.log   # truncament a mida zero
   ```
2. **Resultat esperat**: RealttyLog continua mostrant les línies noves que s'hi afegeixen
   després del truncament, sense error ni necessitat de tornar a obrir el fitxer.
3. Repeteix substituint el truncament per un reemplaçament (`mv demo.log.old && touch
   demo.log` amb el mateix camí) i comprova el mateix resultat.

## Escenari D — Fitxer gran (SC-002, SC-003)

1. Genera un fitxer d'uns 5 GB amb moltes línies curtes, per exemple:
   ```sh
   yes "línia de prova per omplir espai" | head -c 5G > /tmp/gran.log
   ```
2. Obre `/tmp/gran.log` amb RealttyLog i mesura el temps fins que es veu el final del
   fitxer.
   **Resultat esperat**: menys de 2 segons.
3. Deixa'l seguint en directe mentre hi afegeixes línies noves de tant en tant (com a
   l'escenari A) durant uns 30 minuts.
4. Mesura el consum de memòria del procés (Gestor de tasques a Windows, `top`/`htop` a
   Linux).
   **Resultat esperat**: es manté per sota d'uns 20 MB per sobre del consum inicial en obrir
   el fitxer, independentment de com de gran sigui `/tmp/gran.log`.

## Neteja

```sh
rm -f /tmp/demo.log /tmp/gran.log
```
