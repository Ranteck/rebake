# Plan: cuidar la cuota al armar todas (desde spec.md 2026-10-22 e intent.md 2026-10-22)
Estado: aceptado

## Archivos que cambian
- `src/investigador.rs`: compartir internamente `receta_directa`, conservando su uso en `Trabajo` y sus validaciones.
- `src/tui/app.rs`: clasificación común, confirmación fijada por IDs y seguimiento de corridas.
- `src/tui/vista.rs`: `armable`, `siguiente_paso`, `lineas_pie`, posible origen y ayuda.
- `src/tui/mod.rs`: despacho y limpieza ante errores de preparación; reutilizar efectos, canales y guardado.
- `tests/tui.rs`, `tests/investigador.rs`: ampliar fixtures y pruebas existentes.
- `README.md`: recorrido, teclas y búsqueda de origen.
- `intent/cuidar-cuota/plan.md` (nuevo): este plan. Los demás archivos existen.

## Tests a escribir
Pruebas nuevas deterministas: teclas, efectos, resultados inyectados y `TestBackend`; sin hilos, esperas, red ni Claude.

| Test (comportamiento observable) | REQ que cubre |
| --- | --- |
| 25 directas y 41 pagas: `t` despacha solamente las 25 gratis; sus resultados quedan por revisar. URLs/identificadores inseguros nunca cuentan como gratis. | REQ-1, REQ-8 |
| Con 0, 1, 4, 5 y 6 pagas, pregunta solo cuando corresponde y por 1, 4, 5 y 5; completar directas no oculta la pregunta. | REQ-2 |
| `s` confirma 3 investigaciones y 2 búsquedas, ordenadas como la lista; omite IDs resueltos, excluidos, eliminados o ya ocupados, sin reemplazarlos ni incorporar requisitos nuevos; todos omitidos no despachan nada. | REQ-3 |
| Enter, Esc, `i`, `t`, `q`, `s` mayúscula y cambio de pestaña cancelan sin su acción habitual; aviso de gasto cero y directas conservadas. | REQ-3 |
| Claude de tanda o individual bloquea otra pregunta y avisa que espere; `t` sigue despachando directas. Hilos exclusivamente directos permiten preguntar. | REQ-4 |
| Terminar cinco resultados mezclando éxitos/errores no encadena; un `i` adicional sigue independiente. Fallidas cuentan otra vez; error al preparar trabajos libera seguimiento. | REQ-4, REQ-7, REQ-8 |
| En 80×24, 107/141 y 999/999 muestran pestañas, ayuda y ambas cuentas completas; variantes cero omiten su parte; filtro y mensajes no alteran cuentas globales. | REQ-5 |
| Posibles de plugin/marketplace seguros anuncian gratis; MCP, skill con vía buscada y datos inseguros anuncian una corrida; dibujar no acepta el origen. | REQ-6 |
| `i` durante tanda no consume sus cinco lugares; aprobación individual, Enter, descarte, excluidas y pestaña Instalación conservan comportamiento; `t` saltea posibles pendientes, descartados y no encontrados. | REQ-8 |

## Orden de trabajo
1. Escribir los tests de clasificación y comprobar su fallo; exponer `receta_directa` dentro del crate, sin duplicar criterios. Clasificar pendientes sin trabajo; investigaciones antes que búsquedas, orden `(tipo, id)` de pantalla, conservando alcance global de `t` pese al filtro. Ejecutar tests focalizados (REQ-1, REQ-3, REQ-5).
2. Probar e implementar `ConfirmarTanda` siguiendo `ConfirmarInstalacion`: guardar hasta cinco IDs y su operación; `t` devuelve únicamente las directas. Renderizar pregunta desde el modo, con prioridad sobre mensajes. `s` revalida los IDs fijados; otra tecla consume y cancela. Reutilizar `Investigar`, `Buscar`, `Lote` (REQ-1–REQ-3).
3. Probar y registrar separadamente IDs de Claude en curso, incluidos los de `i`, y miembros de tanda; mantener `investigando` para spinners de directas también. Retirar IDs al registrar cualquier resultado o fallo de preparación. Bloquear preguntas solamente por Claude; nunca lanzar otra tanda automáticamente. No despachar vectores vacíos. Registrar decisiones con contadores en eventos estructurados de `tui/mod.rs` y verificarlos mediante captura determinista, sin datos personales (REQ-4, REQ-7, REQ-8).
4. Probar y actualizar pie, siguiente paso, ayuda y aviso del panel. Simular aceptación sobre copia con `Via::Busqueda`, usando el criterio compartido. Medición realizada: pestañas y ayuda ocupan 52 columnas; tecla y cuentas triples, 28: total 80, incluidos espacios. Verificarlo con ancho Unicode y buffer, sin exigir que entren también acciones de selección (REQ-2, REQ-5, REQ-6).
5. Actualizar README; ejecutar pruebas focalizadas tras cada paso y verificación completa al terminar (REQ-1–REQ-8).

## Riesgos
- `investigar_todo_investiga_las_con_fuente_y_busca_el_origen_de_las_otras` y `t_busca_una_sola_vez_por_item` esperan lanzamiento inmediato: actualizar a confirmación, conservando exclusiones. Ajustar `la_cabecera_muestra_las_sin_origen_antes_y_despues_del_lote`, tests `el_pie_*`, `siguiente_con_solo_sin_origen_dice_si_falta_buscarlos` y textos de ayuda: cambia secuencia/copia, no se debilitan aserciones.
- `servicio::trabajos` interpreta IDs vacíos como todas; evitar gasto accidental.
- Esta feature sustituye el lanzamiento masivo de REQ-12/17 de recetas-directas y el texto del pie de vista-clara: sin reescribir esos specs, cada uno suma una línea en `## Cambios` que apunta a `cuidar-cuota`.

## Prueba
- `cargo fmt`: formato.
- `cargo clippy --all-targets -- -D warnings`: compilación y lint de todos los targets.
- `cargo test --locked`: tabla completa: gratis (REQ-1), pregunta persistente (REQ-2), confirmación/cancelación (REQ-3), concurrencia (REQ-4), ancho (REQ-5), costo (REQ-6), errores (REQ-7), regresiones (REQ-8).
