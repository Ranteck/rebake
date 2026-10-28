# Spec: aprobar todas y destildar (desde intent.md 2026-10-27)
Estado: borrador

Definiciones:
- Por revisar: receta armada pendiente de aprobación humana.
- Posible origen: hallazgo de búsqueda web, sin receta aún.
- Tanda: lote de hasta 5 corridas (investigaciones o búsquedas).

## Requisitos
- REQ-1 MUST: En Recetas, una tecla libre sin Shift (`v`) pide confirmación en el pie ("¿Aprobar N recetas? s confirma; otra tecla cancela") para aprobar todas las por revisar. Origen: Resultado 1.
- REQ-2 MUST: `s` aprueba todas las por revisar y guarda el cookbook; cualquier otra tecla cancela sin cambios y el pie lo avisa. Origen: Resultado 1.
- REQ-3 MUST: Sin recetas por revisar, `v` no pregunta y el pie avisa que no hay nada para aprobar. Origen: Restricciones.
- REQ-4 MUST: En Recetas, espacio o `a` sigue funcionando igual para alternar aprobación: sobre una aprobada la vuelve a por revisar (destildar) y guarda el cambio. Origen: Resultado 2 y Restricciones.
- REQ-5 MUST: Al terminar una tanda mixta, el pie informa cuántas quedaron por revisar, cuántas con posible origen y cuántas con error; sugiere `v` solo si hay por revisar, y `t` si quedan pendientes. Origen: Resultado 3.
- REQ-6 MUST: "Por revisar" usa la marca `□` en lista, cabecera y ayuda, en lugar de `○`. Origen: Resultado 4.
- REQ-7 MUST: En 80×24, si conviven pendientes (`t`) y por revisar (`v`), el pie muestra ambas con `? ayuda` compactando etiquetas (pestañas o acciones) para no superar 80 columnas, sin Shift ni modales; la confirmación usa la segunda línea. Origen: Restricciones.
- REQ-8 MUST: La aprobación con `v` solo afecta recetas por revisar; no aprueba ítems sin receta ni excluidos, y no instala. Origen: Resultado 1 ("Una tecla aprueba todas las por revisar") y Restricciones ("las que no tienen receta no se aprueban").

## Capacidades y escenarios
### Aprobar todas con confirmación (REQ-1, REQ-2, REQ-8)
- GIVEN 60 por revisar y 6 sin receta
- WHEN Denis presiona `v` y confirma con `s`
- THEN las 60 quedan aprobadas, las 6 sin receta siguen igual y el pie confirma la aprobación.

- GIVEN la pregunta "¿Aprobar 60 recetas?" en el pie
- WHEN Denis presiona otra tecla (como Esc)
- THEN ninguna cambia y el pie avisa la cancelación.

### Sin recetas por revisar (REQ-3)
- GIVEN 0 por revisar
- WHEN Denis presiona `v`
- THEN no pregunta y el pie avisa que no hay nada para aprobar.

### Destildar sigue igual (REQ-4)
- GIVEN una receta aprobada seleccionada
- WHEN Denis presiona espacio o `a`
- THEN vuelve a por revisar con marca `□` y queda guardada.

### Fin de tanda mixta (REQ-5)
- GIVEN una tanda de 5 en curso (3 investigaciones y 2 búsquedas)
- WHEN terminan (2 investigaciones listas, 1 error, 1 posible origen y 1 búsqueda fallida)
- THEN el pie avisa el balance (2 por revisar, 1 posible origen, 2 errores) y sugiere `v` para aprobar o `t` para seguir.

### Convivencia en 80×24 y marca nueva (REQ-6, REQ-7)
- GIVEN terminal en 80×24 con 107 gratis, 141 con Claude y 60 por revisar
- WHEN se dibuja Recetas
- THEN los ítems muestran `□`, la cabecera muestra `□ 60 por revisar` y el pie muestra completos pestañas, `? ayuda`, `t` y `v` compactando etiquetas para no desbordar.

## Concerns
- **Tecla `v`:** Libre en la TUI, sin Shift, operable con mano izquierda y nemotécnica (visto bueno / validar). No colisiona con `a` (individual), `t` (tandas) ni `s` (escanear/confirmar).
- **Marca `□`:** U+25A1 representa un casillero esperando tilde (`✓`), mide 1 columna y disipa la confusión de `○` con procesos activos.
- **Ajuste en 80 columnas:** Con cifras de tres dígitos, compactar pestañas o acciones garantiza mostrar `t`, `v` y `?` en el pie sin desbordar.
- **Estrategia inversa:** Aprobar todo y destildar con espacio agiliza tandas grandes sin restar control.
