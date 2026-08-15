# Feature Specification: Tailing bàsic

**Feature Branch**: `001-tailing-basic`

**Created**: 2026-08-15

**Status**: Draft

**Input**: User description: "Fase 1 del full de ruta de RealttyLog: lectura asíncrona de
fitxers de log en creixement constant, amb rendiment estabilitzat, autoscroll a l'última
línia i pausa automàtica quan l'usuari repassa l'historial."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Seguir un log en temps real (Priority: P1)

Una persona que administra un servidor de producció obre amb RealttyLog el fitxer de log
d'una aplicació que està escrivint-hi contínuament. Sense fer res més, veu com apareixen les
línies noves a mesura que es generen, com faria amb `tail -f` a un terminal, però en una
finestra pròpia.

**Why this priority**: és la raó de ser de l'eina. Sense seguiment en temps real no hi ha
producte.

**Independent Test**: es pot provar sol, amb un script que vagi afegint línies a un fitxer
de text mentre RealttyLog el té obert, i comprovant que cada línia nova apareix a la
finestra sense recarregar ni tornar a obrir el fitxer.

**Acceptance Scenarios**:

1. **Given** un fitxer de log existent, **When** l'usuari l'obre amb RealttyLog, **Then**
   veu les últimes línies que ja hi havia escrites, amb la vista situada al final del
   fitxer.
2. **Given** RealttyLog té un fitxer obert, **When** el procés que genera el log hi afegeix
   una línia nova, **Then** la línia apareix a la finestra en menys d'un segon, sense cap
   acció de l'usuari.
3. **Given** RealttyLog té un fitxer obert, **When** l'usuari el tanca des de l'aplicació,
   **Then** deixa de llegir-lo i allibera el fitxer perquè cap altre procés no en quedi
   bloquejat.

---

### User Story 2 - Repassar l'historial sense perdre el fil (Priority: P1)

Mentre les línies van entrant, la persona que mira el log vol pujar amunt per repassar un
error que acaba de veure passar. En fer-ho, la finestra deixa de saltar avall sola; quan ja
ha acabat de mirar-s'ho, torna al final amb una sola acció i reprèn el seguiment en directe.

**Why this priority**: sense això, un flux d'alt volum fa impossible llegir res: la vista
saltaria contínuament mentre l'usuari intenta llegir una línia antiga.

**Independent Test**: amb un fitxer que rep línies noves contínuament, desplaçar-se cap
amunt i comprovar que la vista es queda quieta encara que arribin línies noves; després,
tornar avall amb una acció i comprovar que el seguiment en directe es reprèn.

**Acceptance Scenarios**:

1. **Given** RealttyLog està seguint un fitxer en directe, **When** l'usuari desplaça la
   vista cap amunt, **Then** l'autoscroll es pausa i la posició de lectura no es mou encara
   que arribin línies noves.
2. **Given** l'autoscroll està pausat, **When** l'usuari activa "tornar al directe" (amb un
   sol clic o desplaçant-se fins al final), **Then** la vista salta a l'última línia i
   l'autoscroll es reprèn.
3. **Given** l'autoscroll està pausat, **When** arriben línies noves, **Then** l'usuari en
   veu un indicador (per exemple, un comptador o un avís visual) sense que se li interrompi
   la lectura.

---

### User Story 3 - Obrir fitxers grans sense penalització (Priority: P2)

La persona que administra el servidor obre un fitxer de log que ja pesa diversos gigabytes
perquè fa dies que no es rota. RealttyLog l'obre igual de ràpid que un de petit i no fa que
l'ordinador es quedi sense memòria ni es bloquegi.

**Why this priority**: en entorns de producció reals els logs no sempre estan nets i
rotats; si l'eina només funciona amb fitxers petits, no serveix per al cas d'ús que la
justifica.

**Independent Test**: generar un fitxer de diversos GB, obrir-lo amb RealttyLog i mesurar
temps d'obertura i consum de memòria, comparant-los amb els d'un fitxer petit.

**Acceptance Scenarios**:

1. **Given** un fitxer de log de diversos gigabytes, **When** l'usuari l'obre, **Then**
   RealttyLog es posiciona al final i mostra les últimes línies sense llegir ni carregar tot
   el fitxer a memòria.
2. **Given** RealttyLog segueix un fitxer gran en directe durant una estona llarga, **When**
   es mesura el consum de memòria, **Then** es manté acotat i no creix proporcionalment a la
   mida del fitxer.

---

### Edge Cases

- **Rotació de log** (el fitxer es trunca a mida zero o es reemplaça per un de nou amb el
  mateix nom, com fan `logrotate` o els loggers amb rotació diària): RealttyLog ho detecta i
  continua seguint el fitxer nou o truncat sense que l'usuari hagi de tornar a obrir-lo ni
  vegi cap error.
- **Fitxer esborrat mentre s'està seguint**: RealttyLog ho mostra clarament a la interfície
  en lloc de fallar en silenci o tancar-se.
- **Fitxer en una unitat de xarxa que es desconnecta temporalment**: RealttyLog ho indica i
  reprèn el seguiment sol quan torna a estar disponible, sense reobrir manualment.
- **Contingut no vàlid com a UTF-8** (bytes solts d'una altra codificació): es mostren de
  manera visible (per exemple, com a caràcter de reemplaçament) en lloc de trencar la
  lectura o saltar-se contingut.
- **Línia extremadament llarga** (una traça sencera en una sola línia, de diversos MB): es
  mostra sense penjar la interfície ni bloquejar l'arribada de línies posteriors.
- **Fitxer buit en obrir-lo**: RealttyLog l'obre igualment i mostra un estat clar de "sense
  contingut encara", seguint-lo per si hi comença a arribar res.
- **Doble obertura del mateix fitxer**: no hi ha res que ho impedeixi, però tampoc cal que
  RealttyLog ho gestioni de manera especial en aquesta fase.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: El sistema MUST permetre obrir un fitxer de log existent des de l'aplicació.
- **FR-002**: En obrir un fitxer, el sistema MUST posicionar la vista al final i mostrar-ne
  les últimes línies, sense llegir ni carregar tot el fitxer a memòria.
- **FR-003**: El sistema MUST detectar quan s'afegeix contingut nou al fitxer obert i
  mostrar-lo a la finestra sense que l'usuari hagi de fer cap acció.
- **FR-004**: El sistema MUST mostrar cada línia com a text pla, preservant l'ordre original
  i sense alterar-ne el contingut.
- **FR-005**: El sistema MUST desplaçar automàticament la vista cap a l'última línia mentre
  segueix un fitxer en directe (autoscroll).
- **FR-006**: El sistema MUST pausar l'autoscroll automàticament quan l'usuari desplaça la
  vista cap amunt per llegir línies anteriors.
- **FR-007**: El sistema MUST permetre reprendre l'autoscroll i tornar a l'última línia amb
  una sola acció de l'usuari.
- **FR-008**: El sistema MUST distingir visualment si està en mode "en directe" (seguint,
  autoscroll actiu) o "pausat" (l'usuari repassa l'historial).
- **FR-009**: El sistema MUST detectar la rotació o el truncament extern del fitxer que
  segueix i continuar-ne el seguiment sense intervenció de l'usuari ni missatge d'error.
- **FR-010**: El sistema MUST informar de manera visible si el fitxer que seguia deixa
  d'estar disponible (esborrat, desmuntat), i MUST reprendre el seguiment sol quan hi torni
  a haver accés.
- **FR-011**: El sistema MUST gestionar contingut que no sigui UTF-8 vàlid sense aturar-se
  ni descartar la resta del fitxer.
- **FR-012**: El sistema MUST permetre tancar el fitxer que s'està seguint i obrir-ne un
  altre sense haver de reiniciar l'aplicació.
- **FR-013**: El sistema MUST mantenir el consum de memòria acotat i independent de la mida
  total del fitxer mentre el segueix, d'acord amb la constitució del projecte.

### Key Entities

- **Fitxer seguit**: el log que RealttyLog té obert en un moment donat. Té un camí, una
  posició de lectura actual dins el fitxer i un estat (en directe, pausat, no disponible).
- **Línia**: una unitat de contingut aparegut al fitxer seguit, amb el seu text i l'ordre en
  què ha arribat. És la unitat mínima que es mostra i es desplaça per la finestra.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Una línia nova escrita al fitxer apareix a la finestra en menys d'1 segon des
  que s'ha escrit.
- **SC-002**: Obrir un fitxer de 5 GB i mostrar-ne el final triga menys de 2 segons.
- **SC-003**: El consum de memòria es manté per sota d'uns 20 MB per sobre del punt de
  partida després de 30 minuts seguint en directe un fitxer de diversos gigabytes.
- **SC-004**: L'usuari torna al mode "en directe" des de qualsevol punt de l'historial amb
  una sola acció, sempre.
- **SC-005**: Una rotació del fitxer de log no interromp mai el seguiment ni fa que
  l'aplicació es tanqui o deixi de respondre, verificat amb almenys 10 rotacions seguides.
- **SC-006**: Una persona que no ha fet servir RealttyLog abans obre un fitxer i entén si
  està "en directe" o "pausat" sense haver de consultar cap documentació.

## Assumptions

- **Un sol fitxer alhora**: aquesta fase segueix un únic fitxer per finestra. Múltiples
  fitxers o pestanyes simultànies no hi entren; és una decisió de disseny per a una fase
  posterior si cal.
- **Comportament "tail -f"**: en obrir un fitxer ja existent, es mostra el tram final que ja
  hi havia (com faria `tail -f`), no una pantalla buida esperant contingut nou. És l'ús
  esperat de qualsevol eina de "tailing" i el que fan totes les eines de referència citades
  al document de concepte.
- **Sense interpretació del contingut**: en aquesta fase les línies es tracten com a text
  pla. Detectar i formatar JSON o HTML incrustat és la Fase 2 (Detection Engine) i queda
  fora d'aquest abast.
- **Sense filtres ni cerca**: seleccionar per paraula clau, nivell de log o expressió
  regular no és objectiu d'aquesta fase.
- **Codificació per defecte UTF-8**: és l'estàndard de la immensa majoria de logs moderns;
  els bytes que no ho compleixin es mostren substituïts en lloc de tractar-se com un error
  fatal.
- **Rotació de log**: es cobreixen els dos patrons habituals (truncament a mida zero i
  reemplaçament per un fitxer nou amb el mateix nom), que són els que fan servir
  `logrotate` i la majoria de loggers amb rotació integrada.
