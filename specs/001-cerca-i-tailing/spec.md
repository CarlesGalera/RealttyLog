# Feature Specification: Cerca i tailing

**Feature Branch**: `001-cerca-i-tailing`

**Created**: 2026-08-15

**Status**: Draft

**Input**: User description: "Fase 1 ampliada de RealttyLog: en entorns de producció reals els
logs sovint es parteixen per dia o per mida, i qui busca un problema no sempre sap a quin
fitxer és. Cal poder obrir un directori sencer de logs, cercar-hi text a tots els fitxers,
saltar del resultat al fitxer i la línia exactes, i des d'allà seguir-lo en directe amb
autoscroll i pausa automàtica en repassar l'historial — tot sense penalització en fitxers
grans."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Cercar text a tot un directori de logs (Priority: P1)

Una persona que administra un servidor de producció sap que hi ha hagut un error, però els
logs es parteixen per dia (`app-2026-08-14.log`, `app-2026-08-15.log`...) o per mida
(`app.log`, `app.log.1`, `app.log.2`...) i no sap en quin fitxer és. Obre el directori on
viuen tots aquests fitxers amb RealttyLog i hi escriu el text que busca (per exemple, un
missatge d'error o un identificador de petició). Veu una llista de totes les coincidències,
amb el fitxer i un fragment de context per a cadascuna.

**Why this priority**: sense això, l'usuari ha d'endevinar o obrir fitxer per fitxer fins a
trobar l'error, que és exactament el problema que aquesta ampliació de la Fase 1 vol
resoldre. És l'entrada real a l'eina en un cas d'ús de producció.

**Independent Test**: es pot provar sol, amb un directori de diversos fitxers de text on
només un conté una paraula concreta: cercar-la i comprovar que la llista de resultats
assenyala el fitxer correcte, sense necessitat de cap altra funcionalitat.

**Acceptance Scenarios**:

1. **Given** un directori amb diversos fitxers de log, **When** l'usuari l'obre amb
   RealttyLog, **Then** veu llistats els fitxers de log que conté.
2. **Given** un directori obert, **When** l'usuari hi escriu un text de cerca, **Then**
   RealttyLog el cerca a tots els fitxers del directori i en mostra els resultats.
3. **Given** una cerca amb coincidències en més d'un fitxer, **When** es mostren els
   resultats, **Then** cada resultat indica de quin fitxer prové i un fragment de la línia
   on s'ha trobat.
4. **Given** una cerca sense cap coincidència, **When** acaba, **Then** l'usuari ho veu
   clarament indicat, no una llista buida sense explicació.
5. **Given** una cerca en curs sobre un directori gran, **When** l'usuari la cancel·la,
   **Then** RealttyLog atura la cerca i mostra els resultats trobats fins aquell moment.

---

### User Story 2 - Saltar del resultat de cerca al fitxer i la línia exactes (Priority: P1)

Un cop l'usuari veu la llista de coincidències, en clica una i RealttyLog l'obre
directament posicionat a la línia on s'ha trobat, sense que hagi d'obrir el fitxer a mà i
buscar-hi la línia visualment.

**Why this priority**: una llista de resultats sense poder-hi saltar és només mig útil;
localitzar l'error i poder-lo mirar en context és el que tanca el cas d'ús.

**Independent Test**: amb una cerca ja feta i almenys un resultat, clicar-lo i comprovar que
s'obre el fitxer corresponent amb la línia trobada visible, sense haver de desplaçar-se
manualment.

**Acceptance Scenarios**:

1. **Given** una llista de resultats de cerca, **When** l'usuari en clica un, **Then**
   RealttyLog obre el fitxer corresponent posicionat a la línia de la coincidència.
2. **Given** un resultat obert d'aquesta manera, **When** l'usuari mira al seu voltant,
   **Then** veu també les línies anteriors i posteriors per entendre'n el context.
3. **Given** l'usuari ha obert un resultat, **When** vol tornar a la llista de resultats,
   **Then** ho pot fer sense haver de repetir la cerca.

---

### User Story 3 - Seguir en directe el fitxer un cop localitzat (Priority: P1)

Un cop l'usuari és al fitxer correcte —hagi arribat des d'una cerca o l'hagi obert
directament—, si aquell fitxer encara rep escriptures, RealttyLog en mostra les línies
noves a mesura que es generen, com faria `tail -f`.

**Why this priority**: després de localitzar el problema, sovint cal veure si continua
passant en directe; és el nucli original de "tailing" que dona nom al projecte.

**Independent Test**: es pot provar sol, amb un script que vagi afegint línies a un fitxer
mentre RealttyLog el té obert, i comprovant que cada línia nova hi apareix sense recarregar.

**Acceptance Scenarios**:

1. **Given** un fitxer obert (directament o des d'un resultat de cerca), **When** el procés
   que el genera hi afegeix una línia nova, **Then** apareix a la finestra en menys d'1
   segon, sense cap acció de l'usuari.
2. **Given** RealttyLog té un fitxer obert, **When** l'usuari el tanca des de l'aplicació,
   **Then** deixa de llegir-lo i allibera el fitxer perquè cap altre procés no en quedi
   bloquejat.

---

### User Story 4 - Repassar l'historial sense perdre el fil (Priority: P1)

Mentre les línies van entrant en directe, l'usuari puja amunt per repassar el context d'un
error. La finestra deixa de saltar avall sola; quan acaba, torna al final amb una sola
acció.

**Why this priority**: sense això, un flux d'alt volum fa impossible llegir res mentre se
segueix en directe.

**Independent Test**: amb un fitxer que rep línies noves contínuament, desplaçar-se cap
amunt i comprovar que la vista es queda quieta; tornar avall amb una acció i comprovar que
el seguiment en directe es reprèn.

**Acceptance Scenarios**:

1. **Given** RealttyLog està seguint un fitxer en directe, **When** l'usuari desplaça la
   vista cap amunt, **Then** l'autoscroll es pausa i la posició de lectura no es mou encara
   que arribin línies noves.
2. **Given** l'autoscroll està pausat, **When** l'usuari activa "tornar al directe",
   **Then** la vista salta a l'última línia i l'autoscroll es reprèn.
3. **Given** l'autoscroll està pausat, **When** arriben línies noves, **Then** l'usuari en
   veu un indicador sense que se li interrompi la lectura.

---

### User Story 5 - Fitxers grans sense penalització (Priority: P2)

Tant la cerca a un directori com el seguiment d'un fitxer concret funcionen igual de bé
quan els fitxers implicats pesen diversos gigabytes, sense que l'ordinador es quedi sense
memòria ni l'aplicació es bloquegi.

**Why this priority**: en producció els logs no sempre estan nets i rotats; si l'eina només
funciona amb fitxers petits, no serveix per al cas d'ús que la justifica.

**Independent Test**: generar un directori amb fitxers de diversos GB, cercar-hi text i
obrir-ne un en directe, mesurant temps i consum de memòria.

**Acceptance Scenarios**:

1. **Given** un directori amb fitxers de diversos gigabytes, **When** s'hi cerca un text,
   **Then** RealttyLog el llegeix per streaming, sense carregar cap fitxer sencer a memòria.
2. **Given** un fitxer de diversos gigabytes obert en directe durant una estona llarga,
   **When** es mesura el consum de memòria, **Then** es manté acotat i no creix
   proporcionalment a la mida del fitxer.

---

### Edge Cases

- **Directori amb subdirectoris** (logs organitzats per data en carpetes): RealttyLog hi
  entra i també els cerca.
- **Fitxer no reconeixible com a text** (binari, comprimit): es descarta silenciosament de
  la cerca, sense error visible.
- **Fitxer sense permisos de lectura**: es descarta de la cerca amb un avís discret, sense
  aturar la resta.
- **Cerca amb resultats massius** (terme molt comú): es mostra un nombre acotat de resultats
  amb manera de refinar la cerca, en lloc d'aclaparar la interfície o penjar-se.
- **Fitxer nou que apareix al directori mentre una cerca està en curs**: no s'hi inclou;
  l'usuari pot tornar a cercar per veure'l.
- **Rotació de log** (el fitxer que se segueix es trunca a mida zero o es reemplaça per un
  de nou amb el mateix nom): RealttyLog ho detecta i continua seguint-lo sense que l'usuari
  hagi de tornar a obrir-lo.
- **Fitxer esborrat mentre s'està seguint**: RealttyLog ho mostra clarament en lloc de fallar
  en silenci.
- **Fitxer en una unitat de xarxa que es desconnecta temporalment**: RealttyLog ho indica i
  reprèn el seguiment sol quan torna a estar disponible.
- **Contingut no vàlid com a UTF-8**: es mostra de manera visible (caràcter de
  reemplaçament) en lloc de trencar la lectura o la cerca.
- **Línia extremadament llarga**: es mostra sense penjar la interfície ni bloquejar
  l'arribada de línies posteriors.
- **Fitxer buit**: RealttyLog l'obre igualment i mostra un estat clar de "sense contingut
  encara", seguint-lo per si hi comença a arribar res.

## Requirements *(mandatory)*

### Functional Requirements

#### Cerca en un directori

- **FR-001**: El sistema MUST permetre obrir un directori i llistar-ne els fitxers de log
  que conté, incloent-hi els de subdirectoris.
- **FR-002**: El sistema MUST permetre cercar un text lliure a tots els fitxers d'un
  directori obert alhora.
- **FR-003**: El sistema MUST llegir els fitxers per streaming durant la cerca, sense
  carregar-ne cap de sencer a memòria.
- **FR-004**: El sistema MUST mostrar, per cada coincidència, el fitxer on s'ha trobat i un
  fragment de context suficient per reconèixer-la sense obrir el fitxer.
- **FR-005**: El sistema MUST permetre cancel·lar una cerca en curs i conservar els
  resultats trobats fins aquell moment.
- **FR-006**: El sistema MUST indicar clarament quan una cerca no ha trobat cap
  coincidència.
- **FR-007**: El sistema MUST descartar de la cerca, sense aturar-se, els fitxers que no
  siguin de text o que no es puguin llegir per manca de permisos.
- **FR-008**: El sistema MUST acotar el nombre de resultats mostrats d'una cerca amb moltes
  coincidències, oferint una manera de refinar-la.

#### Navegació

- **FR-009**: El sistema MUST permetre obrir, des d'un resultat de cerca, el fitxer
  corresponent posicionat a la línia exacta de la coincidència.
- **FR-010**: El sistema MUST mostrar el context (línies anteriors i posteriors) al voltant
  d'una línia a la qual s'ha saltat des d'un resultat de cerca.
- **FR-011**: El sistema MUST permetre tornar a la llista de resultats d'una cerca sense
  haver de repetir-la.
- **FR-012**: El sistema MUST permetre obrir un fitxer de log directament, sense passar per
  una cerca prèvia.

#### Seguiment en directe

- **FR-013**: En obrir un fitxer sense saltar-hi des d'un resultat de cerca, el sistema
  MUST posicionar la vista al final i mostrar-ne les últimes línies, sense llegir ni
  carregar tot el fitxer a memòria.
- **FR-014**: El sistema MUST detectar quan s'afegeix contingut nou al fitxer obert i
  mostrar-lo sense que l'usuari hagi de fer cap acció.
- **FR-015**: El sistema MUST mostrar cada línia com a text pla, preservant l'ordre
  original i sense alterar-ne el contingut.
- **FR-016**: El sistema MUST desplaçar automàticament la vista cap a l'última línia mentre
  segueix un fitxer en directe (autoscroll).
- **FR-017**: El sistema MUST pausar l'autoscroll automàticament quan l'usuari desplaça la
  vista cap amunt per llegir línies anteriors.
- **FR-018**: El sistema MUST permetre reprendre l'autoscroll i tornar a l'última línia amb
  una sola acció de l'usuari.
- **FR-019**: El sistema MUST distingir visualment si està en mode "en directe" o "pausat".
- **FR-020**: El sistema MUST detectar la rotació o el truncament extern del fitxer que
  segueix i continuar-ne el seguiment sense intervenció de l'usuari ni missatge d'error.
- **FR-021**: El sistema MUST informar de manera visible si el fitxer que seguia deixa
  d'estar disponible, i MUST reprendre el seguiment sol quan hi torni a haver accés.
- **FR-022**: El sistema MUST gestionar contingut que no sigui UTF-8 vàlid sense aturar-se
  ni descartar la resta del fitxer.
- **FR-023**: El sistema MUST permetre tancar el fitxer que s'està seguint i obrir-ne un
  altre (o tornar a la cerca) sense haver de reiniciar l'aplicació.

#### Rendiment i fiabilitat

- **FR-024**: El sistema MUST mantenir el consum de memòria acotat i independent de la mida
  total dels fitxers implicats, tant en cercar com en seguir, d'acord amb la constitució
  del projecte.
- **FR-025**: El sistema MUST permetre repassar tot l'historial disponible d'un fitxer, per
  gran que sigui, sense descartar ni ocultar contingut que encara hi existeixi: acotar la
  memòria és responsabilitat de RealttyLog, no un límit que l'usuari hagi de patir sobre el
  seu propi fitxer.

### Key Entities

- **Directori de logs**: la carpeta que l'usuari obre per cercar-hi. Conté una llista de
  fitxers de log (inclosos els de subdirectoris) vigent en el moment d'obrir-lo o de llançar
  una cerca.
- **Cerca**: un text lliure llançat contra un directori de logs obert, amb un estat (en
  curs, acabada, cancel·lada) i una llista de coincidències.
- **Coincidència**: un resultat d'una cerca. Assenyala un fitxer, una posició dins seu i un
  fragment de context; és el pont entre la cerca i el fitxer seguit que se n'obre.
- **Fitxer seguit**: el log que RealttyLog té obert en un moment donat, arribi d'una cerca o
  s'obri directament. Té un camí, una posició de lectura actual i un estat (en directe,
  pausat, no disponible).
- **Línia**: una unitat de contingut aparegut al fitxer seguit, amb el seu text i l'ordre en
  què ha arribat. És la unitat mínima que es mostra i es desplaça per la finestra.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Des d'un directori de logs sense identificar prèviament el fitxer correcte,
  l'usuari arriba a la línia exacta d'un error conegut en menys de 30 segons.
- **SC-002**: Cercar un text en un conjunt de fitxers de fins a 20 GB en total mostra el
  primer resultat en menys de 3 segons.
- **SC-003**: Una línia nova escrita al fitxer obert apareix a la finestra en menys d'1
  segon des que s'ha escrit.
- **SC-004**: Obrir directament un fitxer de 5 GB i mostrar-ne el final triga menys de 2
  segons.
- **SC-005**: El consum de memòria es manté per sota d'uns 20 MB per sobre del punt de
  partida després de 30 minuts seguint en directe un fitxer de diversos gigabytes, sense que
  això impliqui perdre l'accés a cap part de l'historial ja escrit al fitxer.
- **SC-006**: L'usuari torna al mode "en directe" des de qualsevol punt de l'historial amb
  una sola acció, sempre.
- **SC-007**: Una rotació del fitxer de log no interromp mai el seguiment ni fa que
  l'aplicació es tanqui o deixi de respondre, verificat amb almenys 10 rotacions seguides.
- **SC-008**: Una persona que no ha fet servir RealttyLog abans obre un directori, hi cerca
  un text i entén com saltar a un resultat sense haver de consultar cap documentació.
- **SC-009**: Des de qualsevol fitxer, per gran que sigui, l'usuari pot desplaçar-se fins al
  principi i veure-hi el contingut real que hi ha, no un buit ni un missatge de límit
  superat.

## Assumptions

- **Cerca de text literal**: en aquesta fase la cerca és per coincidència de text, sense
  distingir majúscules ni minúscules per defecte, i sense expressions regulars. Cercar amb
  patrons més avançats queda per a una fase futura si cal.
- **Foto fixa del directori**: la cerca opera sobre els fitxers que hi ha en el moment de
  llançar-la. Un fitxer nou que aparegui mentre la cerca està en curs no s'hi inclou; cal
  tornar a cercar per veure'l.
- **Sense índex persistent**: cada cerca torna a llegir els fitxers pertinents; no es
  manté cap índex entre sessions ni entre cerques. És l'enfocament més senzill i portable
  per a aquesta fase; es pot revisar si el volum real de logs ho exigeix.
- **Comportament "tail -f" en obrir directament**: en obrir un fitxer sense passar per una
  cerca, es mostra el tram final que ja hi havia, no una pantalla buida esperant contingut
  nou.
- **Sense interpretació del contingut**: en aquesta fase les línies es tracten com a text
  pla. Detectar i formatar JSON o HTML incrustat és la Fase 2 (Detection Engine) i queda
  fora d'aquest abast.
- **Codificació per defecte UTF-8**: és l'estàndard de la immensa majoria de logs moderns;
  els bytes que no ho compleixin es mostren substituïts en lloc de tractar-se com un error
  fatal.
- **Rotació de log**: es cobreixen els dos patrons habituals (truncament a mida zero i
  reemplaçament per un fitxer nou amb el mateix nom).
- **Un sol fitxer seguit en directe alhora**: es pot cercar a tot un directori, però només
  se'n segueix un en directe a la vegada en aquesta fase. Seguir-ne diversos simultàniament
  queda per a una fase futura si cal.
