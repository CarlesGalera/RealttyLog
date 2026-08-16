# Feature Specification: Ressaltat per paraula clau i filtratge instantani

**Feature Branch**: `003-ressaltat-i-filtratge`

**Created**: 2026-08-16

**Status**: Draft

**Input**: User description: "Fase 3 de RealttyLog: amb l'expansió inline de la Fase 2 ja
feta, el que queda del document de concepte original és el punt 4 de Característiques
Clau — regles de colors per paraula clau (ERROR, WARN, HTTP 500...) i filtratge instantani
per canal o nivell de log. RealttyLog no coneix l'esquema del log que llegeix (no en fa
parsing estructurat), així que 'canal' i 'nivell' no són camps garantits: es defineixen com
a paraules clau que l'usuari indica, no com a columnes fixes."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ressaltar línies per paraula clau (Priority: P1)

Una persona repassa un fitxer de log ple d'INFO rutinaris i vol que els `ERROR` i els
`WARN` li saltin a la vista sense haver-los de buscar un per un. Defineix una regla per
paraula (o fragment) amb un color, i totes les línies que la contenen es ressalten a
l'instant, tant a les que ja hi eren com a les que arriben en directe.

**Why this priority**: és el valor mínim de la fase — sense ressaltat visual, l'usuari
segueix depenent de llegir-s'ho tot línia per línia, que és exactament el problema que el
projecte vol resoldre.

**Independent Test**: amb un fitxer que barreja línies `INFO`, `WARN` i `ERROR`, definir una
regla `ERROR` → vermell i comprovar que només aquelles línies es mostren ressaltades,
sense afectar les altres.

**Acceptance Scenarios**:

1. **Given** cap regla de ressaltat definida, **When** s'obre un fitxer, **Then** cap línia
   es ressalta i el mur es veu igual que a les fases anteriors.
2. **Given** una regla de paraula clau amb un color assignat, **When** una línia visible la
   conté, **Then** la línia es mostra ressaltada amb aquell color.
3. **Given** una regla activa, **When** arriba una línia nova en directe que la conté,
   **Then** es ressalta igual que si ja hi fos en obrir el fitxer.
4. **Given** diverses regles amb colors diferents, **When** una línia només compleix una
   d'elles, **Then** s'hi aplica el color d'aquella regla concreta.
5. **Given** una línia que compleix més d'una regla alhora, **When** es mostra, **Then**
   s'hi aplica la regla amb prioritat més alta (Key Entities) de manera consistent, no una
   barreja ambigua de colors.
6. **Given** una regla definida sense distingir majúscules i minúscules (per defecte),
   **When** una línia conté la paraula en una capitalització diferent, **Then** també es
   ressalta.

---

### User Story 2 - Filtrar el mur per nivell o paraula clau (Priority: P1)

L'usuari vol veure només els `ERROR` d'un fitxer sorollós, amagant la resta de línies sense
perdre-les (continuen al fitxer, no es descarten). Activa el filtre per la paraula `ERROR` i
el mur mostra només les línies que la contenen; el desactiva i torna a veure-ho tot.

**Why this priority**: el ressaltat sol ja ajuda, però en un fitxer de milers de línies
trobar les que importen encara costa cop d'ull rere cop d'ull — filtrar-les és el que
estalvia temps de debò. Es prioritza igual que la User Story 1 perquè reutilitza les
mateixes regles definides allà.

**Independent Test**: amb un fitxer de línies mixtes i una regla `ERROR` ja definida,
activar-ne el filtre i comprovar que només les línies que la compleixen queden visibles;
desactivar-lo i comprovar que hi tornen totes.

**Acceptance Scenarios**:

1. **Given** una regla definida, **When** l'usuari n'activa el filtre, **Then** el mur
   mostra només les línies que la compleixen; la resta desapareix de la vista sense
   esborrar-se del fitxer.
2. **Given** un filtre actiu, **When** l'usuari el desactiva, **Then** totes les línies hi
   tornen a ser, en el mateix ordre que abans.
3. **Given** més d'una regla amb el filtre actiu alhora, **When** es mostra el mur,
   **Then** hi apareix qualsevol línia que en compleixi almenys una (OR, no AND: reduir
   soroll no ha de exigir que una línia compleixi totes les paraules alhora).
4. **Given** un filtre actiu, **When** el fitxer segueix en directe i arriba una línia nova,
   **Then** apareix al mur només si compleix el filtre actiu, sense interrompre el
   seguiment.
5. **Given** un filtre actiu que amaga la línia seleccionada o desplegada (Fase 2),
   **When** s'aplica, **Then** el desplegament es tanca de manera net, sense deixar un forat
   ni un error a la vista.
6. **Given** un filtre actiu que no compleix cap línia del fitxer, **When** es mostra,
   **Then** el mur queda buit amb un avís clar que hi ha un filtre actiu, no una pantalla en
   blanc sense explicació.

---

### User Story 3 - Gestionar les regles durant la sessió (Priority: P2)

L'usuari obre el panell de regles, n'afegeix, n'edita el color o la paraula, en desactiva
una temporalment sense esborrar-la, i n'esborra les que ja no calen — tot mentre el fitxer
es continua seguint en directe.

**Why this priority**: sense poder gestionar les regles, les User Stories 1 i 2 només
servirien per a una única paraula fixada al codi. No és el valor central de la fase (que és
veure el resultat ressaltat o filtrat), però cal per fer-la útil més enllà d'una demo.

**Independent Test**: afegir una regla nova, comprovar que s'aplica; editar-ne el color,
comprovar que canvia; desactivar-la sense esborrar-la, comprovar que deixa de ressaltar
però es manté a la llista; esborrar-la, comprovar que desapareix de la llista i deixa
d'afectar el mur.

**Acceptance Scenarios**:

1. **Given** el panell de regles obert, **When** l'usuari n'afegeix una amb paraula i
   color, **Then** s'aplica de seguida a les línies visibles, sense recarregar el fitxer.
2. **Given** una regla existent, **When** l'usuari en canvia el color o la paraula,
   **Then** el ressaltat (i el filtre, si estava actiu) es recalcula amb el valor nou.
3. **Given** una regla activa, **When** l'usuari la desactiva sense esborrar-la, **Then**
   deixa de ressaltar i de filtrar, però es manté a la llista per reactivar-la després.
4. **Given** una regla a la llista, **When** l'usuari l'esborra, **Then** desapareix de la
   llista i cap línia hi queda ressaltada ni filtrada per aquella paraula.
5. **Given** que es tanca l'aplicació, **When** es torna a obrir, **Then** les regles NO
   es recorden (Assumptions): la persistència queda per a la Fase 4 (Configuració
   Portable).

---

### Edge Cases

- **Paraula clau buida o només espais**: la regla no s'accepta; no té sentit ressaltar o
  filtrar per una cadena buida que "compliria" totes les línies.
- **Paraula clau molt freqüent** (per exemple, una lletra sola): es permet igualment —
  l'usuari és qui decideix si la regla és útil, l'eina no jutja el contingut de la regla.
- **Milers de línies visibles amb diverses regles actives alhora**: avaluar el ressaltat i
  el filtre no pot alentir perceptiblement el desplaçament ni el seguiment en directe
  (memòria i rendiment acotats, principi III de la constitució).
- **Una línia ja expandida (Fase 2) que compleix una regla de ressaltat**: el text
  desplegat es ressalta igual que la línia condensada, de manera coherent.
- **Canviar les regles mentre el fitxer segueix creixent en directe**: el recàlcul s'aplica
  també a les línies que ja eren a la finestra activa, no només a les noves.
- **Dues regles amb el mateix color però paraules diferents**: es permet; l'eina no exigeix
  colors únics, és una elecció de l'usuari.

## Requirements *(mandatory)*

### Functional Requirements

#### Regles

- **FR-001**: El sistema MUST permetre definir una regla composta per una paraula o
  fragment de text i un color.
- **FR-002**: La cerca de la paraula clau d'una regla MUST ser insensible a majúscules i
  minúscules per defecte.
- **FR-003**: El sistema MUST permetre afegir, editar, desactivar (sense esborrar) i
  esborrar regles en qualsevol moment, amb efecte immediat sobre el mur visible.
- **FR-004**: El sistema MUST NOT acceptar una regla amb la paraula clau buida.
- **FR-005**: Quan diverses regles actives compleixen la mateixa línia, el sistema MUST
  aplicar-hi el color d'una única regla determinada per una prioritat consistent (Key
  Entities), mai una barreja ambigua.

#### Ressaltat

- **FR-006**: El sistema MUST ressaltar visualment (color de fons o de text) qualsevol
  línia visible que compleixi almenys una regla activa.
- **FR-007**: El ressaltat MUST aplicar-se tant a les línies ja carregades com a les que
  arriben mentre el fitxer segueix en directe, sense necessitat de recarregar-lo.
- **FR-008**: El ressaltat MUST reflectir els canvis de regla (afegir, editar, desactivar,
  esborrar) de manera immediata sobre les línies ja visibles.

#### Filtratge

- **FR-009**: El sistema MUST permetre activar un filtre per una o més regles, mostrant
  només les línies que en compleixen almenys una (OR entre regles filtrades).
- **FR-010**: Amagar una línia per filtre MUST NOT esborrar-la ni modificar-la al fitxer
  d'origen: és només un canvi de visibilitat a la vista.
- **FR-011**: El sistema MUST permetre desactivar el filtre i recuperar totes les línies a
  l'instant, en el mateix ordre que abans d'aplicar-lo.
- **FR-012**: Quan un filtre actiu no compleix cap línia visible, el sistema MUST mostrar
  un avís explícit que el mur és buit per un filtre actiu, no una vista en blanc sense
  context.
- **FR-013**: El filtratge MUST aplicar-se també a les línies noves mentre el fitxer
  segueix en directe, sense interrompre el seguiment (Fase 1).

### Key Entities

- **Regla de ressaltat**: paraula o fragment de text, color assignat, estat (activa o
  desactivada), estat de filtre (si a més de ressaltar, amaga les línies que no la
  compleixen), i una prioritat implícita per l'ordre de creació (la més recent guanya en
  cas de conflicte de color entre diverses regles que compleixen la mateixa línia).
- **Estat de filtratge del mur**: quines regles tenen el filtre actiu en un moment donat;
  determina quines línies es mostren, no és un concepte nou de dades sobre la línia
  mateixa.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Amb una regla `ERROR` definida sobre un fitxer amb línies mixtes, el 100% de
  les línies que contenen `error` (en qualsevol capitalització) es ressalten.
- **SC-002**: Activar o desactivar un filtre sobre un fitxer de desenes de milers de línies
  ja carregades actualitza la vista en menys de 200 ms.
- **SC-003**: Ressaltar o filtrar no interromp mai el seguiment en directe de la resta del
  fitxer (0 línies noves perdudes mentre hi ha regles actives).
- **SC-004**: Una persona que mai ha vist el panell de regles pot definir-ne una i veure'n
  l'efecte sense consultar cap documentació.
- **SC-005**: Amb dos filtres actius simultanis (per exemple `ERROR` i `WARN`), el mur
  mostra el 100% de les línies que compleixen almenys un dels dos i cap de les que no en
  compleixen cap.

## Assumptions

- **"Canal" i "nivell" són paraules clau, no camps estructurats**: el document de concepte
  original parlava de "filtratge per canal o nivell de log", però RealttyLog no fa parsing
  d'un esquema de log fix (cada eina que el genera té el seu propi format). Es resol amb un
  mecanisme genèric de paraula clau que serveix igual de bé per a un nivell (`ERROR`), un
  canal amb nom (`auth-service`) o qualsevol altre fragment que l'usuari vulgui distingir.
- **Sense persistència en aquesta fase**: les regles viuen només durant la sessió oberta.
  Desar-les en un fitxer al costat de l'executable és exactament l'abast de la Fase 4
  (Configuració Portable) del document de concepte original — introduir-ho aquí duplicaria
  feina.
- **Coincidència de text pla, no expressions regulars**: el document de concepte parla de
  "paraules clau"; s'interpreta com a coincidència de subcadena literal (cas insensible),
  no com un motor de regex, per mantenir la interfície simple. Si calen regles més
  expressives, queda per a un CR futur.
- **El filtre actua sobre la finestra visible, no re-escaneja tot el fitxer**: coherent amb
  el principi de memòria acotada (Fase 1, decisió de `ViewportCache`) — amagar línies és un
  canvi de visualització, no una nova cerca completa del fitxer en cada canvi de filtre.
