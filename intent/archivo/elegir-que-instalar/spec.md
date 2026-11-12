# Spec: elegir qué instalar desde la URL del cookbook (desde elegir-que-instalar/intent.md 2026-10-13)
Estado: aceptado

## Requisitos
- REQ-1 MUST: `rebake <url>` sin clon previo clona el cookbook en su carpeta y abre la pantalla
  en la pestaña Instalación. Origen: Resultado 1 — "`rebake <url>` clona la primera vez y abre
  la pantalla".
- REQ-2 MUST: `rebake <url>` con un clon de esa misma URL trae lo último y abre en Instalación;
  `rebake` a secas sigue abriendo en Recetas. Origen: Resultado 1 — "después alcanza con
  `rebake`".
- REQ-3 MUST: si la carpeta ya tiene un clon de otra URL, o contenido que no es un clon, falla
  nombrando lo que hay (sin credenciales) y no toca nada. Origen: Restricciones — "Con un clon
  de otra URL, avisa y no pisa nada".
- REQ-4 MUST: la pestaña Instalación lista las recetas aprobadas en el orden en que se
  instalan, todas marcadas; las flechas mueven, espacio marca o desmarca y `t` marca todas o
  ninguna. Origen: Resultado 3 — "instalar todo o desmarcar las recetas que no se quieren";
  Resultado — "medio similar al flujo de instalar skills o plugins de claude code".
- REQ-5 MUST: `P` pide confirmación diciendo cuántas de cuántas y solo instala las marcadas.
  Origen: Resultado 3.
- REQ-6 MUST: marcar y desmarcar no cambia el cookbook: no se guarda ni se publica. Origen:
  Restricciones — "Desmarcar vale solo para esa instalación en esa PC".
- REQ-7 MUST: desmarcar una receta desmarca las que la requieren, salvo que esté comprobada
  como instalada; marcar una receta marca las que requiere. Origen: Resultado 3; spec raíz
  REQ-10 — "en el orden de sus requisitos".
- REQ-8 MUST: abrir la pantalla no ejecuta ningún comando del cookbook; `c` corre la
  comprobación de cada receta aprobada en cada perfil, sin instalar. Origen: Restricciones —
  "Abrir un cookbook no ejecuta nada suyo: la comprobación corre cuando Denis la pide".
- REQ-9 MUST: tras comprobar, cada fila dice "instalada", "falta", "falta en <perfiles>" o "no
  se puede saber" (receta sin comprobación); las instaladas quedan sin casilla. Origen:
  Resultado 2 — "qué ya está instalado en esta PC y qué falta"; Restricciones — "Se elige por
  receta, no por perfil".
- REQ-10 MUST: una comprobación que no se puede ejecutar o supera 30 segundos deja la fila en
  "no se pudo comprobar" y las demás siguen. Origen: Resultado 2.
- REQ-11 SHOULD: un panel muestra, de la receta resaltada, los comandos que va a correr, su
  comprobación, qué requiere y dónde falta. Origen: Resultado — "medio similar al flujo de instalar skills o
  plugins de claude code".
- REQ-12 MUST: la lista marca "nueva" la receta aprobada que llegó en la traída de esta
  corrida y "cambió" la que cambió sus comandos o su comprobación; las marcas no se guardan y
  el primer clonado no marca nada. Origen: Resultado 4.
- REQ-13 MUST: `rebake instalar` lista las recetas que va a instalar, con esas marcas, antes de
  pedir confirmación; con `--si` las lista y sigue. Origen: Resultado 5.

## Capacidades y escenarios
### Entrar por la URL (REQ-1, REQ-2, REQ-3)
- GIVEN `~/Documentos/rebake` inexistente — WHEN `rebake <url>` — THEN queda el clon marcado y
  la pantalla abre en Instalación
- GIVEN un clon de `<url>` — WHEN `rebake <url>` — THEN trae lo último y abre en Instalación
- GIVEN un clon de `<url>` — WHEN `rebake` — THEN abre en Recetas
- GIVEN un clon de otra URL — WHEN `rebake <url>` — THEN falla nombrando la URL del clon y la
  carpeta queda igual (rechazo)

### Elegir (REQ-4 a REQ-7, REQ-11)
- GIVEN 4 recetas aprobadas — WHEN abro Instalación — THEN las 4 están marcadas
- GIVEN un plugin que requiere un marketplace sin comprobar — WHEN desmarco el marketplace —
  THEN el plugin también se desmarca
- GIVEN 1 de 4 desmarcada — WHEN `P` y confirmo — THEN se instalan 3, la desmarcada no corre
  ningún comando y el cookbook no cambia
- GIVEN ninguna receta aprobada — WHEN abro Instalación — THEN dice que no hay nada para
  instalar y cómo aprobar

### Comprobar (REQ-8, REQ-9, REQ-10)
- GIVEN una receta con comprobación — WHEN abro la pantalla — THEN su comprobación no corrió
- GIVEN una receta instalada en laburo y no en personal — WHEN `c` — THEN dice "falta en
  personal" y sigue marcada
- GIVEN una receta instalada en todos sus perfiles — WHEN `c` — THEN dice "instalada" y no
  tiene casilla
- GIVEN una comprobación que no termina — WHEN `c` — THEN a los 30 segundos esa fila dice "no
  se pudo comprobar" y las demás tienen su estado

### Novedades del repo (REQ-12, REQ-13)
- GIVEN un clon y otro equipo que aprobó una receta nueva — WHEN `rebake` — THEN esa fila dice
  "nueva"
- GIVEN un clon y otro equipo que cambió un comando de una receta — WHEN `rebake instalar` —
  THEN la confirmación lista las recetas y esa dice "cambió"
- GIVEN un primer `rebake <url>` — WHEN abre — THEN ninguna fila tiene marca

## Diseño
- **Entrada:** la URL es un argumento opcional del comando raíz, sin subcomando.
  `rebake clonar <url>` no cambia. La URL del clon se compara con `origin` sin credenciales ni
  `.git` final.
- **Selección:** vive en el estado de la pantalla, junto a la lista de instalación que ya
  existe; el instalador recibe solo las recetas marcadas.
- **Comprobación:** se separa del instalador la parte que hoy corre `verificar` por perfil,
  para poder llamarla sin instalar, en segundo plano y con su propio tope.
- **Novedades:** el comando lee el cookbook antes y después de traer y compara las recetas
  aprobadas; el resultado viaja a la pantalla y a `instalar`.

## Concerns
- Las marcas son de la corrida: si un comando trajo un cambio y recién en la corrida siguiente
  abrís Instalación, ya no aparece marcado. Recordarlo entre corridas pide guardar estado en la
  PC; se deja afuera salvo que Denis lo pida.
- `c` es la primera vez que se ejecuta algo del cookbook: corre todas las comprobaciones
  juntas, sin confirmación. El panel deja leerlas antes, de a una.
- Con el cookbook actual de Denis (73 recetas pendientes) la pestaña queda vacía hasta
  investigar y aprobar.
- El dibujo de la pantalla no se prueba; sí su lógica (teclas y estado) y los comandos.
