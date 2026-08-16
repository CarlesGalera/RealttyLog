# Quickstart: Configuració portable de les regles

Guia de validació manual. Continua sobre els quickstarts de les Fases 1-3: aquests
escenaris assumeixen que ja saps definir i gestionar regles de ressaltat/filtratge.

## Posada en marxa

```sh
cargo build --release
cd target/release
./realttylog
```

(Executar-lo des del mateix directori que el binari perquè els passos de còpia de
l'Escenari P tinguin sentit.)

## Escenari O — Les regles sobreviuen a tancar l'aplicació (SC-001)

1. Amb `realttylog` obert, defineix dues regles: `ERROR` → vermell (amb el filtre actiu) i
   `WARN` → groc (sense filtre).
2. Tanca l'aplicació del tot (no només el fitxer: surt del programa).
3. Comprova que `realttylog-rules.json` existeix al mateix directori que l'executable.
4. Torna a obrir `realttylog`.
   **Resultat esperat**: les dues regles hi són, amb el mateix color, paraula clau i estat
   de filtre que tenien abans de tancar.
5. Esborra la regla `WARN` i desactiva `ERROR` (sense esborrar-la). Tanca i torna a obrir.
   **Resultat esperat**: només hi ha `ERROR`, desactivada.

## Escenari P — La configuració viatja amb l'executable (SC-002)

1. Amb almenys una regla definida i l'aplicació tancada, copia tant `realttylog` (o
   `realttylog.exe`) com `realttylog-rules.json` a un directori nou:
   ```sh
   mkdir -p /tmp/realttylog-portable
   cp realttylog realttylog-rules.json /tmp/realttylog-portable/
   cd /tmp/realttylog-portable
   ./realttylog
   ```
2. **Resultat esperat**: la regla definida abans hi és, sense haver-la tornat a escriure.
3. Ara copia només l'executable (sense el fitxer de configuració) a un altre directori nou
   i executa'l des d'allà.
   **Resultat esperat**: arrenca sense cap regla i sense cap error, com un primer ús.

## Escenari Q — Un fitxer de configuració trencat no bloqueja l'aplicació (SC-003)

1. Amb l'aplicació tancada, escriu contingut invàlid al fitxer de configuració:
   ```sh
   echo "això no és JSON" > realttylog-rules.json
   ./realttylog
   ```
   **Resultat esperat**: l'aplicació arrenca amb normalitat, sense regles, sense cap
   diàleg ni pantalla d'error.
2. Tanca l'aplicació. Escriu un array JSON vàlid amb una regla ben formada i una altra amb
   el camp `color` absent:
   ```sh
   cat > realttylog-rules.json <<'EOF'
   [
     {"keyword": "ERROR", "color": {"r": 220, "g": 90, "b": 90}, "enabled": true, "filter": false},
     {"keyword": "TRENCADA", "enabled": true, "filter": false}
   ]
   EOF
   ./realttylog
   ```
   **Resultat esperat**: la regla `ERROR` es carrega i s'aplica amb normalitat; la regla
   `TRENCADA` (sense color) no hi apareix, sense que això impedeixi carregar la primera.

## Neteja

```sh
rm -f realttylog-rules.json
rm -rf /tmp/realttylog-portable
```
