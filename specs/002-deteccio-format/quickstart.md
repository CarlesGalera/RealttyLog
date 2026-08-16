# Quickstart: Detecció i formatatge de payloads

Guia de validació manual. Continua directament sobre el quickstart de la Fase 1: aquests
escenaris assumeixen que ja saps obrir un fitxer o cercar un directori.

## Posada en marxa

```sh
cargo build --release
./target/release/realttylog
```

## Escenari G — Detectar JSON, XML i HTML sense trencar el mur (SC-001)

1. Crea un fitxer amb línies mixtes:
   ```sh
   cat > /tmp/payloads.log <<'EOF'
   2026-08-16 10:00:01 INFO petició rebuda
   2026-08-16 10:00:02 DEBUG payload={"usuari":"pep","actiu":true,"intents":3}
   2026-08-16 10:00:03 INFO resposta enviada
   2026-08-16 10:00:04 DEBUG cos=<usuari><nom>Pep</nom><actiu>true</actiu></usuari>
   2026-08-16 10:00:05 WARN pàgina d'error <html><body><h1>404</h1></body></html>
   EOF
   ```
2. Obre `/tmp/payloads.log` amb RealttyLog.
3. **Resultat esperat**: les línies amb JSON, XML i HTML mostren un indicador; les altres
   tres no en mostren cap, i totes cinc es continuen veient en una sola línia (mur
   condensat intacte).

## Escenari H — Desplegar amb indentació i ressaltat (SC-002, SC-004)

1. Amb l'escenari G obert, clica l'indicador de la línia amb el JSON.
2. **Resultat esperat**: el payload es mostra indentat, amb claus, cadenes, números i
   booleans distingits per color, sense perdre cap dada de l'original.
3. Clica l'indicador de la línia amb l'XML i, per separat, el de l'HTML.
4. **Resultat esperat**: totes dues es despleguen indentades per nivell d'imbricació,
   independentment l'una de l'altra i de la línia JSON ja desplegada.
5. Torna a clicar l'indicador del JSON.
6. **Resultat esperat**: aquella línia torna a l'estat condensat; les altres dues
   continuen desplegades.

## Escenari I — Descodificar un JWT (SC-005)

1. Afegeix una línia amb un JWT conegut (capçalera `{"alg":"HS256","typ":"JWT"}`, payload
   `{"sub":"1234567890","name":"Pep"}`):
   ```sh
   echo '2026-08-16 10:00:06 INFO auth token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IlBlcCJ9.dGVzdC1zaWduYXR1cmU' >> /tmp/payloads.log
   ```
2. Torna a obrir (o recarrega) `/tmp/payloads.log` i clica l'indicador de la línia del
   token.
3. **Resultat esperat**: es mostren dos blocs JSON indentats —capçalera i payload— amb els
   valors exactes de dalt (`alg: HS256`, `sub: 1234567890`, `name: Pep`), i la signatura es
   mostra tal com és, marcada com a no desxifrable.

## Escenari J — Payload gran i fitxer en directe (SC-002, SC-003)

1. Genera un JSON d'uns 100 KB en una sola línia:
   ```sh
   python3 -c "import json; print('2026-08-16 10:00:07 DEBUG gran=' + json.dumps({'items': [{'id': i, 'valor': 'x'*20} for i in range(2000)]}))" >> /tmp/payloads.log
   ```
2. Obre el fitxer, clica l'indicador d'aquesta línia i mesura el temps fins que es mostra
   indentat.
   **Resultat esperat**: menys de 200 ms (SC-002).
3. Amb el JSON gran desplegat, obre el fitxer en directe (com a l'Escenari C de la Fase 1)
   i afegeix-hi una línia nova des d'un altre terminal.
   **Resultat esperat**: la línia nova apareix amb normalitat; el desplegament no interromp
   ni alenteix el seguiment (SC-003).

## Neteja

```sh
rm -f /tmp/payloads.log
```
