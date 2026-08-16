# Feature Specification: Configuració portable de les regles

**Feature Branch**: `004-configuracio-portable`

**Created**: 2026-08-16

**Status**: Draft

**Input**: User description: "Fase 4 de RealttyLog (Configuració Portable), l'última del
document de concepte original: desar les regles de ressaltat i filtratge (Fase 3) en un
fitxer al costat de l'executable, perquè sobrevisquin a tancar l'aplicació i viatgin amb
l'executable si es copia a un altre lloc — coherent amb 'portable, sense instal·lació' de
la constitució. La Fase 3 ho va deixar explícitament fora d'abast."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Les regles hi són en tornar a obrir l'aplicació (Priority: P1)

Una persona defineix unes quantes regles de ressaltat i filtratge un dia, tanca
l'aplicació, i l'endemà la torna a obrir: les regles hi són, tal com les va deixar, sense
haver-les de tornar a escriure.

**Why this priority**: és exactament el que la Fase 3 va deixar pendent i el que dona sentit
a la fase — sense això, "Configuració Portable" no aporta res de nou.

**Independent Test**: definir dues o tres regles, tancar l'aplicació del tot, tornar-la a
obrir, i comprovar que les regles (paraula, color, activa/desactivada, filtre) hi són amb
els mateixos valors.

**Acceptance Scenarios**:

1. **Given** una o més regles definides, **When** es tanca l'aplicació i es torna a obrir,
   **Then** totes hi són amb la mateixa paraula clau, color, estat d'activació i estat de
   filtre que tenien.
2. **Given** cap regla definida (primer ús), **When** es tanca i es torna a obrir
   l'aplicació, **Then** arrenca sense cap regla i sense cap avís d'error.
3. **Given** regles ja desades, **When** se n'edita una (color, paraula), se'n desactiva
   una altra, o se n'esborra una tercera, **Then** el canvi es desa igualment: no cal cap
   acció explícita de "guardar".

---

### User Story 2 - La configuració viatja amb l'executable (Priority: P1)

Una persona copia l'executable de RealttyLog (i el fitxer de configuració que hi ha al
costat) a un altre ordinador o a un altre directori del mateix ordinador: les regles hi
segueixen sent, sense haver-les de tornar a definir.

**Why this priority**: és la raó de ser de "portable" (constitució, principi I) aplicada a
la configuració — si les regles es desessin en una carpeta de configuració del sistema
(el perfil de l'usuari, per exemple), deixarien de viatjar amb l'executable, que és
exactament el que el projecte vol evitar.

**Independent Test**: definir una regla, tancar l'aplicació, copiar l'executable i el
fitxer de configuració a un directori diferent, executar-lo des d'allà, i comprovar que la
regla hi és.

**Acceptance Scenarios**:

1. **Given** regles desades al costat de l'executable original, **When** es copien tant
   l'executable com el fitxer de configuració a un directori (o ordinador) diferent i
   s'executa des d'allà, **Then** les regles hi són igual.
2. **Given** només es copia l'executable (sense el fitxer de configuració), **When**
   s'executa des d'un directori nou, **Then** arrenca sense regles, com un primer ús —mai
   amb un error que impedeixi obrir l'aplicació.

---

### User Story 3 - Un fitxer de configuració trencat no impedeix treballar (Priority: P2)

Algú edita a mà el fitxer de configuració (o es fa malbé de qualsevol altra manera) i
l'aplicació el troba il·legible en arrencar: RealttyLog arrenca igualment, sense regles
prèvies, en lloc de fallar o quedar-se sense obrir.

**Why this priority**: la persistència és una comoditat, no ha de convertir-se mai en un
motiu pel qual el visor de logs —la funció principal de l'eina— deixi de funcionar.

**Independent Test**: escriure contingut invàlid al fitxer de configuració i comprovar que
l'aplicació arrenca amb normalitat, sense regles i sense cap diàleg bloquejant.

**Acceptance Scenarios**:

1. **Given** un fitxer de configuració amb contingut que no és JSON vàlid, **When**
   s'arrenca l'aplicació, **Then** arrenca amb normalitat, sense regles prèvies i sense
   cap diàleg ni pantalla d'error que calgui tancar.
2. **Given** un fitxer de configuració vàlid però amb un camp d'una regla fora de rang o
   absent (per exemple, sense color), **When** s'arrenca, **Then** aquella regla concreta
   es descarta silenciosament sense impedir carregar la resta que sí siguin vàlides.
3. **Given** un directori on l'executable no té permisos d'escriptura, **When** l'usuari
   defineix o edita una regla, **Then** el canvi s'aplica igualment durant la sessió
   (ressaltat i filtre funcionen), encara que no es pugui desar per a la propera vegada.

---

### Edge Cases

- **Desar falla a mitja escriptura** (per exemple, es queda sense espai al disc): la
  sessió en curs no es veu afectada; en el pitjor cas, el fitxer de configuració queda
  igual que abans de l'intent (mai a mig escriure, il·legible).
- **Dues instàncies de RealttyLog obertes alhora sobre el mateix directori**: cadascuna
  desa el seu propi estat en sortir; la que es tanca després sobreescriu el fitxer —no hi
  ha fusió de regles entre instàncies, és un cas fora d'abast per a un visor d'un sol
  usuari.
- **Una regla amb la paraula clau buida al fitxer de configuració** (manipulat a mà): es
  descarta en carregar, igual que l'aplicació ja la rebutjaria en crear-la (Fase 3,
  FR-004).

## Requirements *(mandatory)*

### Functional Requirements

#### Desar

- **FR-001**: El sistema MUST desar el conjunt de regles (paraula clau, color, estat
  d'activació, estat de filtre) en un fitxer situat al mateix directori que l'executable.
- **FR-002**: El sistema MUST desar els canvis (afegir, editar, (des)activar, esborrar una
  regla) sense requerir cap acció explícita de "guardar" per part de l'usuari.
- **FR-003**: Un error en desar (per exemple, directori de només lectura) MUST NOT
  interrompre la sessió en curs ni les regles ja aplicades en memòria.

#### Carregar

- **FR-004**: El sistema MUST carregar les regles desades en arrencar, abans que l'usuari
  obri cap fitxer de log.
- **FR-005**: Si no existeix cap fitxer de configuració (primer ús), el sistema MUST
  arrencar amb el conjunt de regles buit, sense cap avís d'error.
- **FR-006**: Si el fitxer de configuració existeix però no es pot interpretar (contingut
  invàlid), el sistema MUST arrencar igualment amb el conjunt de regles buit, sense cap
  diàleg bloquejant.
- **FR-007**: Si una regla concreta dins d'un fitxer de configuració per la resta vàlid té
  un camp absent o fora de rang, el sistema MUST descartar només aquella regla i carregar
  la resta amb normalitat.

#### Portabilitat

- **FR-008**: El fitxer de configuració MUST viure sempre relatiu a la ubicació de
  l'executable, mai en una carpeta de configuració pròpia del sistema operatiu ni del
  perfil de l'usuari.
- **FR-009**: Copiar l'executable i el seu fitxer de configuració junts a un altre
  directori o ordinador MUST preservar les regles sense cap pas addicional.

### Key Entities

- **Fitxer de configuració**: un únic fitxer, al costat de l'executable, que representa el
  conjunt de regles de la Fase 3 (`RuleSet`) en un format llegible i editable a mà. No
  introdueix cap concepte de negoci nou: és la forma persistida de "Regla de ressaltat" i
  "Estat de filtratge del mur" (spec de la Fase 3).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: El 100% de les regles definides en una sessió hi són, amb els mateixos
  valors, en tornar a obrir l'aplicació.
- **SC-002**: Copiar l'executable i el fitxer de configuració a un directori nou preserva
  el 100% de les regles, sense cap pas manual addicional.
- **SC-003**: Un fitxer de configuració il·legible o absent mai impedeix que l'aplicació
  arrenqui (0 arrencades fallides per aquesta causa).
- **SC-004**: Desar un canvi a una regla no introdueix cap alentiment perceptible a la
  interfície (per sota del llindar dels 200 ms ja establert a les Fases 2 i 3 per a
  accions interactives).

## Assumptions

- **Només les regles de la Fase 3**: el document de concepte original només esmenta
  "regles de ressaltat (colors, regex)" per a aquesta fase. Altres preferències possibles
  (mida de finestra, últim fitxer obert) queden fora d'abast: no hi ha cap requisit que les
  esmenti, i ampliar-ho seria afegir abast no demanat.
- **Un sol fitxer, format llegible**: es desa com un únic fitxer de text al costat de
  l'executable (decisió de format concreta a research.md), no una carpeta ni una base de
  dades encastada.
- **Desar és silenciós**: ni desar amb èxit ni fallar-hi produeix cap confirmació visible
  a l'usuari (Edge Cases, FR-003) — és una comoditat de fons, coherent amb com ja
  funcionava el ressaltat de la Fase 3 (efecte immediat, sense passos explícits).
- **Sense fusió entre instàncies**: si dues execucions de RealttyLog conviuen sobre el
  mateix directori, la que es tanca en darrer lloc guanya (Edge Cases) — no hi ha bloqueig
  de fitxer ni resolució de conflictes, fora d'abast per a un visor d'un sol usuari.
