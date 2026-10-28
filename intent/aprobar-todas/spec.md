# Spec: aprobar todas y destildar (desde intent.md 2026-10-27)
Estado: borrador

Definiciones:
- Por revisar: receta armada pendiente del OK de Denis.
- Tanda: lote de hasta 5 corridas de Claude.

## Requisitos
- REQ-1 MUST: En Recetas, una tecla libre sin Shift (`v`) pide confirmación en el pie ("¿Aprobar N recetas? s confirma; otra tecla cancela") para aprobar todas las por revisar. Origen: Resultado 1 ("Una tecla aprueba todas las por revisar").
- REQ-2 MUST: `s` aprueba todas las por revisar y guarda el cookbook; cualquier otra tecla cancela sin cambiar nada y el pie lo avisa. Origen: Resultado 1 ("s confirma, como las tandas").
- REQ-3 MUST: Sin recetas por revisar, `v` no pregunta y el pie avisa que no hay nada para aprobar. Origen: Restricciones ("las que no tienen receta no se aprueban").
- REQ-4 MUST: En Recetas, espacio o `a` sobre una receta aprobada la vuelve a por revisar (destildar) y guarda el cambio. Origen: Resultado 2 ("espacio sobre una aprobada la vuelve a por revisar").
- REQ-5 MUST: Al terminar una tanda, el pie informa qué dejó (por revisar, errores) y el siguiente paso: `v` para aprobar o `t` para otra tanda. Origen: Resultado 3 ("el pie avisa qué dejó... y qué sigue: aprobar o lanzar otra con t").
- REQ-6 MUST: "Por revisar" usa la marca `□` en la lista, cabecera y ayuda, en lugar de `○`. Origen: Resultado 4 ("cambiá el ○").
- REQ-7 MUST: Todo entra en 80×24 sin Shift ni ventanas emergentes, usando la segunda línea del pie para confirmar. Origen: Restricciones ("Pantalla simple: entra en 80×24, sin teclas con Shift y sin ventanas nuevas").
- REQ-8 MUST: Aprobar todas no toca ítems sin receta ni excluidos, y no inicia la instalación. Origen: Restricciones ("Aprobar de a una sigue igual") y Fuera de alcance ("Instalar al aprobar").

## Capacidades y escenarios
### Aprobar todas con confirmación (REQ-1, REQ-2, REQ-7, REQ-8)
- GIVEN 60 por revisar y 6 sin receta
- WHEN Denis presiona `v` y confirma con `s`
- THEN las 60 quedan aprobadas, las 6 sin receta siguen igual y el pie confirma la aprobación.

- GIVEN la pregunta "¿Aprobar 60 recetas?" en el pie
- WHEN Denis presiona otra tecla (como Esc o n)
- THEN ninguna cambia y el pie avisa que se canceló.

### Sin recetas por revisar (REQ-3)
- GIVEN 0 por revisar
- WHEN Denis presiona `v`
- THEN no pregunta y el pie avisa que no hay nada para aprobar.

### Destildar con espacio (REQ-4)
- GIVEN una receta aprobada seleccionada
- WHEN Denis presiona espacio o `a`
- THEN vuelve a por revisar con marca `□` y queda guardada en el cookbook.

### Fin de tanda y siguiente paso (REQ-5)
- GIVEN una tanda de 5 en curso
- WHEN terminan todas (4 bien y 1 con error)
- THEN el pie avisa el resultado (4 por revisar, 1 error) y sugiere qué sigue: `v` para aprobar o `t` para otra tanda.

### Marca y pantalla en 80×24 (REQ-6, REQ-7)
- GIVEN terminal en 80×24 con recetas por revisar
- WHEN se dibuja Recetas
- THEN los ítems muestran `□`, la cabecera muestra el contador con `□` y el pie muestra `v aprobar 60` sin desbordar.

## Concerns
- **Tecla `v`:** Libre en toda la TUI, sin Shift, operable con la mano izquierda y nemotécnica ("visto bueno" / validar). No colisiona con `a` (individual), `t` (tandas) ni `s` (escanear/confirmar).
- **Marca `□`:** U+25A1 (cuadrado vacío) representa un casillero esperando tilde (`✓`), mide 1 columna y elimina la confusión de `○` con procesos activos o spinners.
- **Confirmación con `s`:** Mantiene la convención de `ConfirmarTanda` en la segunda línea del pie, sin modales ni cambios de foco.
- **Estrategia inversa:** Aprobar todo en lote y destildar excepciones con espacio agiliza tandas grandes sin restar control a Denis.
