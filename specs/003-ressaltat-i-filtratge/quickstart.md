# Quickstart: Ressaltat per paraula clau i filtratge instantani

Guia de validació manual. Continua sobre els quickstarts de les Fases 1 i 2: aquests
escenaris assumeixen que ja saps obrir un fitxer i reconèixer els indicadors de payload.

## Posada en marxa

```sh
cargo build --release
./target/release/realttylog
```

## Escenari K — Ressaltar per paraula clau (SC-001)

1. Crea un fitxer amb línies mixtes:
   ```sh
   cat > /tmp/rules.log <<'EOF'
   2026-08-16 10:00:01 INFO petició rebuda
   2026-08-16 10:00:02 WARN latència alta al backend
   2026-08-16 10:00:03 ERROR connexió refusada
   2026-08-16 10:00:04 INFO resposta enviada
   2026-08-16 10:00:05 ERROR timeout esperant la base de dades
   EOF
   ```
2. Obre `/tmp/rules.log`, obre el panell de regles i defineix `ERROR` → vermell.
3. **Resultat esperat**: les dues línies amb `ERROR` es ressalten en vermell; les altres
   tres es veuen igual que abans.
4. Afegeix una segona regla, `WARN` → groc.
5. **Resultat esperat**: la línia amb `WARN` es ressalta en groc, sense afectar les línies
   `ERROR` ni les `INFO`.

## Escenari L — Filtrar el mur (SC-002, SC-005)

1. Amb l'escenari K obert i les regles `ERROR` (vermell) i `WARN` (groc) definides, activa
   el filtre de la regla `ERROR`.
2. **Resultat esperat**: només es veuen les dues línies `ERROR`; les `INFO` i la `WARN`
   desapareixen de la vista (però el fitxer no s'ha tocat).
3. Activa també el filtre de la regla `WARN`.
4. **Resultat esperat**: ara es veuen les línies `ERROR` i la `WARN` (unió, no
   intersecció) — tres línies en total, cap `INFO`.
5. Desactiva els dos filtres.
6. **Resultat esperat**: totes cinc línies hi tornen a ser, en el mateix ordre.
7. Amb el fitxer obert en directe (com a l'Escenari C de la Fase 1), reactiva el filtre
   `ERROR` i afegeix des d'un altre terminal una línia nova amb `ERROR` i una amb `INFO`.
   **Resultat esperat**: només la línia `ERROR` nova apareix al mur filtrat; la `INFO` nova
   no hi apareix però tampoc interromp el seguiment.

## Escenari M — Gestionar regles durant la sessió (SC-004)

1. Amb el panell de regles obert, edita el color de la regla `WARN` a un altre color.
2. **Resultat esperat**: la línia `WARN` ja visible canvia de color a l'instant, sense
   recarregar el fitxer.
3. Desactiva (sense esborrar) la regla `ERROR`.
4. **Resultat esperat**: les línies `ERROR` deixen de ressaltar-se i, si el seu filtre
   estava actiu, tornen a ser visibles igualment (desactivar atura també el filtre).
5. Esborra la regla `WARN`.
6. **Resultat esperat**: desapareix de la llista del panell i la línia corresponent deixa
   de ressaltar-se.
7. Torna als resultats de cerca (`< Resultats`) i obre un altre fitxer.
   **Resultat esperat**: les regles que queden (per exemple, `ERROR` reactivada) es
   continuen aplicant al nou fitxer — no calen tornar-les a definir.

## Escenari N — Casos límit

1. Intenta crear una regla amb la paraula clau buida.
   **Resultat esperat**: no s'accepta (FR-004); un missatge o l'estat del formulari ho deixa
   clar, no es crea cap regla buida.
2. Amb un filtre actiu que no compleix cap línia del fitxer obert (per exemple, una regla
   `NOEXISTEIX`), activa'n el filtre.
   **Resultat esperat**: el mur queda buit amb un avís explícit que hi ha un filtre actiu,
   no una pantalla en blanc sense context (FR-012).
3. Amb una línia que compleix dues regles alhora (per exemple, un `ERROR` que també conté
   `latència`), comprova quin color s'hi aplica.
   **Resultat esperat**: el color de la regla creada més recentment de les dues
   (research.md, decisió 2), de manera consistent cada vegada que es torna a dibuixar.
4. Amb una línia ja desplegada (Fase 2, per exemple un JSON) que compleix una regla activa,
   comprova el text desplegat.
   **Resultat esperat**: el text desplegat també es ressalta, coherent amb la línia
   condensada.

## Neteja

```sh
rm -f /tmp/rules.log
```
