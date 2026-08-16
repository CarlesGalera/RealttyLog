# Feature Specification: Detecció i formatatge de payloads

**Feature Branch**: `002-deteccio-format`

**Created**: 2026-08-16

**Status**: Draft

**Input**: User description: "Fase 2 de RealttyLog (Detection Engine): moltes línies de log
porten un payload JSON, XML, HTML o un token JWT enganxat en una sola línia, il·legible.
Cal detectar-ho sense trencar el mur condensat del seguiment, i permetre desplegar-lo amb
indentació i ressaltat en un clic. Abast: JSON, XML, HTML i JWT — cap altre format (Modbus,
MQTT, EtherNet/IP i OPC UA són protocols binaris; el que en surt als logs ja és JSON o XML,
que ja hi són)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Reconèixer d'un cop d'ull quines línies porten un payload (Priority: P1)

Una persona repassa el mur d'un fitxer de log ple de línies curtes i, de tant en tant, una
línia amb un JSON o un XML sencer enganxat sense cap espai. Sense haver de fer res, veu un
indicador a les línies que en porten, i la resta del mur es continua llegint igual de
condensat que sempre.

**Why this priority**: és la base de tota la fase — sense saber quines línies val la pena
obrir, l'usuari hauria d'anar clicant a cegues. A més, no ha de trencar mai el mur
condensat que ja funciona (Fase 1): és un afegit, no una substitució.

**Independent Test**: amb un fitxer que barreja línies de text pla i línies amb un JSON o
un XML complet, obrir-lo i comprovar que només les segones porten l'indicador, i que totes
dues es continuen mostrant en una sola línia al mur.

**Acceptance Scenarios**:

1. **Given** una línia que conté un JSON vàlid, **When** es mostra al mur, **Then** hi
   apareix un indicador de payload detectat, sense alterar el text condensat de la línia.
2. **Given** una línia que conté un XML o un HTML vàlid, **When** es mostra, **Then** hi
   apareix el mateix tipus d'indicador.
3. **Given** una línia que conté un token JWT (`xxx.yyy.zzz` en base64url), **When** es
   mostra, **Then** hi apareix l'indicador.
4. **Given** una línia de text pla sense cap d'aquests formats, **When** es mostra,
   **Then** no hi apareix cap indicador.
5. **Given** una línia amb un fragment que sembla estructurat però no ho és del tot (per
   exemple, claus soltes sense ser JSON vàlid), **When** es mostra, **Then** no s'hi marca
   com a payload: millor cap indicador que un que porti a un desplegament trencat.

---

### User Story 2 - Desplegar el payload amb indentació i ressaltat (Priority: P1)

L'usuari clica l'indicador d'una línia i veu el payload —JSON o XML/HTML— formatat amb
indentació i colors per tipus de dada, en lloc de la ratlla il·legible original.

**Why this priority**: és on hi ha el valor real: llegir l'estructura d'un JSON de veritat
en lloc d'una línia de text corredissa.

**Independent Test**: amb una línia que conté un JSON conegut, clicar-ne l'indicador i
comprovar que el payload es mostra indentat, amb la mateixa informació que l'original però
llegible.

**Acceptance Scenarios**:

1. **Given** una línia marcada amb un JSON, **When** l'usuari en clica l'indicador,
   **Then** veu el JSON indentat i amb ressaltat per tipus (claus, cadenes, números,
   booleans) sense perdre cap dada respecte a l'original.
2. **Given** una línia marcada amb un XML o HTML, **When** l'usuari en clica l'indicador,
   **Then** veu el marcatge indentat per nivell d'imbricació, amb les etiquetes
   ressaltades.
3. **Given** un payload desplegat, **When** l'usuari torna a clicar l'indicador (o una
   acció equivalent de tancar), **Then** la línia torna a l'estat condensat, sense afectar
   la resta del mur.
4. **Given** dues línies marcades desplegades alhora, **When** es mostren, **Then**
   cadascuna es pot tancar independentment de l'altra.

---

### User Story 3 - Descodificar un JWT (Priority: P2)

L'usuari clica l'indicador d'una línia amb un token JWT i veu la capçalera i el contingut
(payload) descodificats i formatats com el JSON que porten a dins, en lloc de la cadena
base64url original.

**Why this priority**: els JWT són habituals en logs d'autenticació i d'API, però són
il·legibles a cop d'ull; descodificar-los reutilitza el mateix formatador de JSON de la
User Story 2, així que aporta molt valor amb poca feina afegida un cop aquella existeix.

**Independent Test**: amb una línia que conté un JWT vàlid conegut (capçalera i payload
coneguts), clicar-ne l'indicador i comprovar que es mostren tots dos blocs com a JSON
indentat, amb els valors correctes.

**Acceptance Scenarios**:

1. **Given** una línia marcada amb un JWT, **When** l'usuari en clica l'indicador,
   **Then** veu la capçalera i el payload descodificats i indentats com a JSON, per
   separat.
2. **Given** un JWT descodificat, **When** es mostra, **Then** la signatura es mostra tal
   com és (no es pot desxifrar sense la clau), identificada com a tal i no com a error.
3. **Given** un token amb la forma d'un JWT però amb la capçalera o el payload corromput
   (no decodifica a JSON vàlid), **When** l'usuari en clica l'indicador, **Then** veu un
   avís clar en lloc d'un desplegament trencat o buit.

---

### Edge Cases

- **Payload molt gran** (un JSON de desenes de KB enganxat en una línia): es formata i es
  mostra igualment, sense penjar la interfície ni trencar el pressupost de memòria acotada
  de la constitució.
- **JSON o XML imbricat muntanyes de nivells**: la indentació es manté correcta a
  qualsevol profunditat raonable.
- **Diversos payloads a la mateixa línia** (per exemple, dos JSON separats per text): es
  detecta i es desplega el primer que apareix; la resta de la línia es mostra tal qual.
- **Payload que ocupa tota una línia extremadament llarga** (ja identificat com a cas
  límit a la Fase 1): la detecció no n'agreuja el cost de lectura ni de renderitzat.
- **JSON o XML amb caràcters no vàlids com a UTF-8 al mig** (FR de la Fase 1): el
  formatador els mostra amb el mateix caràcter de reemplaçament que la resta del visor,
  sense fallar.
- **HTML sense DOCTYPE ni arrel `<html>`** (un fragment solt, per exemple una taula):
  s'identifica igualment com a marcatge i es formata, encara que no sigui un document HTML
  complet.
- **Text que comença amb `{` o `<` però no és JSON/XML/HTML vàlid**: no es marca com a
  payload (User Story 1, escenari 5).

## Requirements *(mandatory)*

### Functional Requirements

#### Detecció

- **FR-001**: El sistema MUST detectar, per a cada línia mostrada, si conté un payload
  JSON, XML, HTML o JWT vàlid.
- **FR-002**: El sistema MUST distingir HTML d'XML quan tots dos són sintàcticament
  vàlids, per triar quin ressaltat aplicar-hi.
- **FR-003**: El sistema MUST NOT alterar el text condensat de la línia al mur pel simple
  fet de detectar-hi un payload: la detecció només afegeix un indicador.
- **FR-004**: El sistema MUST NOT marcar com a payload un fragment que no sigui vàlid en
  cap dels quatre formats, encara que hi comenci amb un caràcter típic (`{`, `<`).
- **FR-005**: Quan una línia conté més d'un payload, el sistema MUST detectar i oferir el
  primer que hi apareix.

#### Desplegament i formatatge

- **FR-006**: El sistema MUST permetre desplegar el payload d'una línia marcada amb una
  sola acció, mostrant-lo indentat.
- **FR-007**: El sistema MUST ressaltar per tipus de dada (claus, cadenes, números,
  booleans/null per JSON; etiquetes i atributs per XML/HTML) en el payload desplegat.
- **FR-008**: El sistema MUST permetre tornar a l'estat condensat amb una sola acció,
  independentment d'altres línies desplegades.
- **FR-009**: El sistema MUST permetre tenir més d'una línia desplegada alhora, cadascuna
  gestionada de manera independent.
- **FR-010**: El sistema MUST conservar totes les dades de l'original en desplegar-lo: cap
  camp ni valor es pot perdre ni truncar en formatar.

#### JWT

- **FR-011**: El sistema MUST detectar un JWT per la seva forma (tres segments en
  base64url separats per punts) i descodificar-ne la capçalera i el payload com a JSON.
- **FR-012**: El sistema MUST mostrar la signatura d'un JWT tal com és, identificada com a
  no desxifrable, en lloc d'intentar-la interpretar.
- **FR-013**: El sistema MUST mostrar un avís clar, en lloc d'un desplegament buit o
  trencat, quan la capçalera o el payload d'un JWT no decodifiquin a JSON vàlid.

#### Rendiment i fiabilitat

- **FR-014**: La detecció MUST limitar-se a les línies que es mostren (research.md de la
  Fase 1: la finestra activa, no tot el fitxer), d'acord amb el principi de memòria
  acotada de la constitució.
- **FR-015**: Formatar un payload gran MUST NOT bloquejar la interfície ni impedir que
  arribin línies noves mentre se segueix en directe.

### Key Entities

- **Payload detectat**: el resultat de la detecció sobre una línia — quin format s'hi ha
  trobat (JSON, XML, HTML o JWT) i on comença i acaba dins la línia. No és un concepte nou
  de negoci, és una anotació sobre la "Línia" que ja existeix (Fase 1, data-model.md).
- **Estat de desplegament**: si una línia concreta es mostra condensada o desplegada en un
  moment donat. És independent per a cada línia.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: En un fitxer amb línies de text pla i línies amb payloads coneguts, el 100%
  de les línies amb JSON, XML, HTML o JWT vàlids es marquen, i cap línia de text pla es
  marca per error.
- **SC-002**: Un JSON de fins a 100 KB enganxat en una línia es desplega formatat en menys
  de 200 ms.
- **SC-003**: Desplegar o tancar un payload no interromp mai el seguiment en directe de la
  resta del fitxer.
- **SC-004**: Una persona que veu per primera vegada l'indicador entén que pot clicar-lo
  per veure el payload sencer, sense consultar cap documentació.
- **SC-005**: Un JWT amb capçalera i payload coneguts es descodifica als valors exactes
  que porta, verificat contra el contingut original.

## Assumptions

- **Quatre formats, res més en aquesta fase**: JSON, XML, HTML i JWT. Modbus, MQTT,
  EtherNet/IP i OPC UA són protocols binaris sense format de text propi — el que en surt
  als logs ja és JSON o XML, ja coberts. Dumps hexadecimals, stack traces i SQL llarg
  queden fora perquè no tenen un patró d'obertura prou inequívoc per detectar-los sense
  falsos positius.
- **Un payload per línia**: si n'hi ha diversos, es treballa amb el primer (FR-005); no
  cal una interfície per triar entre diversos payloads d'una mateixa línia en aquesta fase.
- **Desplegament dins la mateixa vista**: "panell inferior o emergent" (document de
  concepte original) es concreta com a expansió inline de la pròpia línia dins la llista,
  no una finestra o panell separat — més senzill d'implementar sobre la vista ja existent
  de la Fase 1 i prou per llegir un payload.
- **Sense edició**: el payload desplegat és de només lectura, igual que la resta del
  visor.
- **JWT sense verificació de signatura**: es descodifica capçalera i payload (base64url
  MUST decodificar-se sempre que la forma sigui correcta), però no es valida
  criptogràficament la signatura — RealttyLog és un visor, no una eina de seguretat.
