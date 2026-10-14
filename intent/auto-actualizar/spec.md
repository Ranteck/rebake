# Spec: rebake se actualiza solo (desde auto-actualizar/intent.md 2026-10-14)
Estado: aceptado

## Requisitos
- REQ-1 MUST: al arrancar, rebake consulta la última versión publicada en GitHub Releases y la
  compara con la suya por número (0.5.10 es más nueva que 0.5.9). Origen: Resultado 1.
- REQ-2 MUST: si está al día, o la suya es más nueva, no muestra nada. Origen: Resultado 4.
- REQ-3 MUST: al abrir la pantalla con una versión atrasada, antes de mostrarla pregunta
  "Hay rebake X (tenés Y). ¿Actualizar? [s/N]". Enter vacío o cualquier otra respuesta es no.
  Origen: Resultados 2 y 5.
- REQ-4 MUST: con "s", baja la versión publicada, verifica su checksum, reemplaza el binario que
  se está corriendo y abre la pantalla con la versión nueva. Origen: Resultado 3; Restricciones
  — "se verifica con su checksum".
- REQ-5 MUST: si la descarga falla, el checksum no coincide o no se puede escribir el binario,
  dice por qué, deja el binario como estaba y abre la pantalla con la versión actual. Origen:
  Restricciones — "si no coincide, no se reemplaza nada"; "Nunca deja un binario a medio
  reemplazar".
- REQ-6 MUST: con "no", abre la pantalla con la versión actual. Origen: Resultado 3.
- REQ-7 MUST: los subcomandos no preguntan: con una versión atrasada escriben por stderr una
  línea "hay rebake X (tenés Y): abrí `rebake` para actualizar" y siguen. Origen: Resultado 5.
- REQ-8 MUST: sin red, si GitHub tarda más de 2 segundos o responde algo que no es una versión,
  rebake arranca igual y no muestra nada. Origen: Restricciones — "la revisión nunca frena el
  uso"; "no debe demorar el arranque".
- REQ-9 SHOULD: `--version` y `--help` no consultan nada. Origen: Restricciones — "no debe
  demorar el arranque".

## Capacidades y escenarios
### Revisión (REQ-1, REQ-2, REQ-8, REQ-9)
- GIVEN rebake 0.5.3 y publicada 0.5.3 — WHEN abro `rebake` — THEN arranca sin preguntar
- GIVEN rebake 0.5.9 y publicada 0.5.10 — WHEN abro `rebake` — THEN pregunta
- GIVEN rebake 0.6.0 compilado a mano y publicada 0.5.3 — WHEN abro `rebake` — THEN no pregunta
- GIVEN sin red — WHEN abro `rebake` — THEN arranca en menos de 3 segundos, sin aviso
- GIVEN cualquier estado — WHEN `rebake --version` — THEN no hay consulta a la red

### Actualizar desde la pantalla (REQ-3 a REQ-6)
- GIVEN publicada 0.5.4 y tengo 0.5.3 — WHEN abro `rebake` y respondo "s" — THEN se reemplaza el
  binario y la pantalla abre con 0.5.4
- GIVEN la misma situación — WHEN respondo Enter — THEN la pantalla abre con 0.5.3 y el binario
  no cambió
- GIVEN un checksum que no coincide — WHEN respondo "s" — THEN dice "el checksum no coincide; no
  se actualizó", el binario no cambió y abre con 0.5.3 (rechazo)
- GIVEN el binario en una carpeta sin permiso de escritura — WHEN respondo "s" — THEN dice que no
  pudo escribir y en qué ruta, y abre con 0.5.3 (rechazo)

### Subcomandos (REQ-7)
- GIVEN publicada 0.5.4 y tengo 0.5.3 — WHEN `rebake instalar --si` — THEN stderr trae la línea
  del aviso, no espera respuesta y la instalación corre igual

## Concerns
- Agrega una consulta a GitHub por arranque. Usa `curl`, que ya pide el `install.sh`; sin
  `curl`, cuenta como "sin red".
- Si rebake se instaló con `cargo install`, reemplaza ese binario (`~/.cargo/bin/rebake`) por el
  publicado. No lo distingue.
- La descarga confía en GitHub Releases y su checksum, igual que el `install.sh`: protege de
  descargas cortadas, no de un repo comprometido.
- Los tests usan un servidor falso con una URL configurable, como `REBAKE_BASE_URL` del
  `install.sh`.
