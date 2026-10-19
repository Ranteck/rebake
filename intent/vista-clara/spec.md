# Spec: una pantalla que se entienda (desde intent.md 2026-10-15)
Estado: aceptado

## Requisitos
- REQ-1 MUST: la pestaña Recetas muestra siempre una leyenda de las marcas de la lista
  (`✓ aprobada  ○ por revisar  · sin receta  ? sin origen  ✗ excluida  … armando`); `?` solo
  marca lo que rebake no sabe de qué repo viene (cambio pedido por Denis, 2026-10-16). Origen: Resultado 1,
  "qué significa cada marca y cada estado".
- REQ-2 MUST: la cabecera muestra el camino `escanear › armar recetas › aprobar › instalar`
  con el paso actual resaltado (en negrita y con un marcador que se ve sin color). El paso es
  el primero que todavía tiene trabajo: sin ítems → escanear; con pendientes con origen →
  armar; con por revisar → aprobar; si no → instalar. Origen: Resultado 2.
- REQ-3 MUST: la línea "Siguiente:" del pie nunca queda vacía si hay algo para hacer y nombra
  la tecla y la cantidad (p. ej. "62 recetas por revisar: elegí una y `a` la aprueba").
  Origen: Resultado 2, "cuál es la próxima tecla útil"; Problema, "no aparece Siguiente:".
- REQ-4 MUST: un nombre que no entra en la lista se corta con `…`, contando ancho en pantalla
  y no bytes. Los plugins del marketplace oficial se muestran sin
  `@claude-plugins-official`, y el nombre completo sigue en el título del detalle.
  Origen: Problema, "nombres cortados".
- REQ-5 MUST: el detalle usa palabras simples: "Viene de" (con "lo dice Claude Code",
  "lo pasaste vos", "enlace simbólico" o "buscado en la web" en lugar de la vía interna),
  "Qué va a ejecutar" para los pasos, "en cada perfil"/"una sola vez" en lugar de
  `por perfil`/`una vez`, "Cómo se arma: comando oficial, sin Claude" en lugar de
  `Doc: receta directa…`, y "Para saber si ya está" en lugar de `Verificar`.
  Origen: Resultado 3; Problema, "jerga interna".
- REQ-6 MUST: en 80 columnas la cabecera no pierde los contadores: si no entran, se abrevian
  (`✓0 ○62 ·1 ?19 ✗0`) antes de cortarse. Origen: Resultado 4; Problema, "la cabecera se corta".
- REQ-7 MUST: el pie muestra solo teclas que hacen algo en esa pestaña y ese estado. En
  Instalación sin recetas aprobadas: `1` volver a Recetas y `q` salir. Origen: Problema,
  "teclas que no hacen nada ahí".
- REQ-8 SHOULD: los estados usan color semántico (aprobada verde, por revisar amarillo,
  error rojo, ausente atenuado), siempre junto a la marca o al texto. Con `NO_COLOR` no
  vacío no se usa color, pero sí negrita y video inverso. Origen: Resultado 4, "con y sin color";
  Restricciones.
- REQ-9 SHOULD: `?` abre una ayuda que explica el camino, las marcas y todas las teclas;
  `?` o `Esc` la cierran y vuelven a la misma selección. Origen: Resultado, "sin leer el README".

- REQ-10 MUST: cada pestaña muestra arriba, en una línea, para qué es y cuándo se usa:
  Recetas "En esta PC: decidí cómo se instala cada cosa"; Instalación "En una PC nueva:
  instalá las recetas aprobadas"; Pendientes "Después de instalar: lo que rebake no hace por
  vos (logins y ajustes)". La pestaña Checklist pasa a llamarse Pendientes, y las teclas
  `1` `2` `3` siguen abriendo las mismas pestañas. Origen: Problema, "las pestañas no
  dicen para qué son".

## Capacidades y escenarios
### Pestañas que se explican (REQ-10)
- GIVEN cualquier pestaña abierta
- WHEN se dibuja
- THEN la primera línea del cuerpo dice para qué es y cuándo se usa, y `2` sigue abriendo
  la lista de logins y ajustes, ahora llamada Pendientes

### Leyenda y camino (REQ-1, REQ-2)
- GIVEN el cookbook real con 62 por revisar y 0 aprobadas
- WHEN se abre `rebake`
- THEN la cabecera resalta "aprobar" y la lista muestra la leyenda de marcas

### Siguiente paso (REQ-3)
- GIVEN 62 por revisar y 20 sin receta, 19 de ellas sin origen
- WHEN se abre `rebake`
- THEN el pie dice cuántas faltan revisar y con qué tecla se aprueba

### Nombres largos (REQ-4)
- GIVEN `claude-md-management@claude-plugins-official` en una lista de 40 columnas
- WHEN se dibuja
- THEN se lee `claude-md-management` entero y nada termina cortado sin `…`

### Detalle legible (REQ-5)
- GIVEN un marketplace con receta directa
- WHEN se selecciona
- THEN el detalle dice "Viene de … (lo dice Claude Code)" y "Qué va a ejecutar", sin
  `metadatos` ni `Doc:`

### Terminal chica (REQ-6, REQ-7)
- GIVEN 80x24 y la pestaña Instalación sin aprobadas
- WHEN se dibuja
- THEN se ven los cuatro contadores y el pie solo ofrece `1` y `q`

### Sin color (REQ-8)
- GIVEN `NO_COLOR=1`
- WHEN se dibuja
- THEN no hay colores y cada estado se sigue distinguiendo por su marca

### Ayuda (REQ-9)
- GIVEN un ítem seleccionado
- WHEN se aprieta `?` y después `Esc`
- THEN se ve la ayuda y al cerrarla vuelve la misma selección
- GIVEN el campo del filtro o del link abierto
- WHEN se escribe `?`
- THEN se agrega al texto y no abre la ayuda

## Concerns
- Los tests que buscan textos de la pantalla actual (`por perfil`, `Fuente:`, etc.) van a
  cambiar: no se debilitan, se actualizan al texto nuevo porque el cambio se pidió.
- unix-cli: los subcomandos ya mandan los datos a stdout y los errores a stderr, y devuelven
  código ≠ 0 al fallar. No encontré violaciones que justifiquen cambiarlos acá.
- El color depende de la paleta de la terminal (Noctalia cambia con el fondo). Por eso el
  color nunca es la única señal.

## Cambios
- 2026-10-18 (v0.8.5): la pantalla se simplifica porque se sentía "súper cargada" (Denis). La
  cabecera es una línea y muestra solo los contadores que no son cero; con sus marcas y nombres
  hacen de leyenda (REQ-1). El camino (REQ-2), la leyenda completa y la línea de para qué es
  cada pestaña (REQ-10) pasan a la ayuda `?`, que muestra el paso actual entre corchetes. El pie
  tiene dos líneas: pestañas, `?` y las teclas de lo elegido (REQ-7), y "Siguiente" (REQ-3), que
  nombra la tecla útil de cada paso. El detalle saca "Estado" (lo dice su primera línea), la
  fecha de la receta y los perfiles cuando son todos. Aprobado.
