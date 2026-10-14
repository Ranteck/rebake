# Spec: rebake se actualiza solo (desde auto-actualizar/intent.md 2026-10-14)
Estado: aceptado

## Requisitos
- REQ-1 MUST: al arrancar, rebake consulta la última versión publicada en GitHub Releases y la
  compara con la suya por número (0.5.10 es más nueva que 0.5.9). Origen: Resultado 1.
- REQ-2 MUST: si está al día, o la suya es más nueva, no muestra nada. Origen: Resultado 4.
- REQ-3 MUST: la pantalla abre sin esperar la revisión, que corre en segundo plano. Con una
  versión atrasada, la cabecera muestra "rebake X disponible (tenés Y): U actualiza" hasta que se
  cierre la pantalla. Origen: Resultados 1 y 2; Restricciones — "sin demorar el arranque".
- REQ-4 MUST: con `U`, baja la versión publicada, verifica su checksum, reemplaza el binario que
  se está corriendo y vuelve a abrir la pantalla con la versión nueva. Origen: Resultado 3;
  Restricciones — "se verifica con su checksum".
- REQ-5 MUST: si la descarga falla, el checksum no coincide o no se puede escribir el binario,
  el aviso dice por qué, el binario queda como estaba y se sigue en la pantalla actual. Origen:
  Restricciones — "si no coincide, no se reemplaza nada"; "Nunca deja un binario a medio
  reemplazar".
- REQ-6 MUST: `U` sin versión nueva conocida, o con una instalación en curso, no hace nada más
  que avisarlo. Origen: Resultado 3 — "si no, sigue con la que tiene".
- REQ-7 MUST: los subcomandos no esperan la revisión: si respondió antes de que terminen y hay
  versión nueva, escriben por stderr "hay rebake X (tenés Y): abrí `rebake` para actualizar".
  Origen: Resultado 5.
- REQ-8 MUST: sin red, si GitHub tarda más de 2 segundos o responde algo que no es una versión,
  no se muestra nada y nada se demora. Origen: Restricciones — "la revisión nunca frena el uso".
- REQ-9 SHOULD: `--version` y `--help` no consultan nada. Origen: Restricciones — "no debe
  demorar el arranque".

## Capacidades y escenarios
### Revisión (REQ-1, REQ-2, REQ-8, REQ-9)
- GIVEN rebake 0.5.9 y publicada 0.5.10 — WHEN se revisa — THEN hay versión nueva
- GIVEN rebake 0.6.0 compilado a mano y publicada 0.5.3 — WHEN se revisa — THEN no hay
- GIVEN un servidor que no responde — WHEN `rebake escanear` — THEN termina sin esperarlo
- GIVEN cualquier estado — WHEN `rebake --version` — THEN no hay consulta a la red

### En la pantalla (REQ-3 a REQ-6)
- GIVEN publicada 0.6.1 y tengo 0.6.0 — WHEN abro `rebake` — THEN la pantalla abre enseguida y
  al rato la cabecera dice "rebake 0.6.1 disponible (tenés 0.6.0): U actualiza"
- GIVEN ese aviso — WHEN `U` — THEN se reemplaza el binario y la pantalla vuelve con 0.6.1
- GIVEN un checksum que no coincide — WHEN `U` — THEN el aviso dice "no se actualizó" y por qué,
  y el binario no cambió (rechazo)
- GIVEN sin versión nueva — WHEN `U` — THEN avisa "rebake está al día" y nada cambia

### Subcomandos (REQ-7)
- GIVEN publicada 0.6.1 y tengo 0.6.0 — WHEN `rebake instalar --si` y la revisión respondió —
  THEN stderr trae la línea del aviso y la instalación corrió igual

## Concerns
- Agrega una consulta a GitHub por arranque, sin demorarlo. Usa `curl`, que ya pide el `install.sh`; sin
  `curl`, cuenta como "sin red".
- Si rebake se instaló con `cargo install`, reemplaza ese binario (`~/.cargo/bin/rebake`) por el
  publicado. No lo distingue.
- La descarga confía en GitHub Releases y su checksum, igual que el `install.sh`: protege de
  descargas cortadas, no de un repo comprometido.
- Los tests usan un servidor falso con una URL configurable, como `REBAKE_BASE_URL` del
  `install.sh`.

## Cambios
- 2026-10-14: revisión en segundo plano y `U` para actualizar, en vez de preguntar antes de
  abrir (enmienda del intent). Aprobado.
- 2026-10-14 (v0.6.4): la tecla también va en minúscula (`u`) y la cabecera dice "u
  actualiza": con `U` sola, la minúscula no hacía nada y no avisaba. Aprobado.
