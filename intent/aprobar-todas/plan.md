# Plan: aprobar todas y destildar (desde spec.md 2026-10-27 e intent.md 2026-10-27)
Estado: borrador

## Archivos que cambian
- `src/tui/app.rs`: modo de confirmación de aprobación en lote, tecla `v`, aviso ante lista vacía y balance de tandas mixtas.
- `src/tui/vista.rs`: marca `□`, compactación del pie en 80 columnas con `t` y `v`, siguiente paso y ayuda.
- `src/tui/mod.rs`: logging estructurado para aprobación en lote y fin de tanda.
- `README.md`: tabla de teclas en Recetas y marca `□` en contadores.
- `tests/tui.rs`: pruebas deterministas de teclas, confirmación, ancho de pie y balance de tanda.
- `intent/aprobar-todas/plan.md` (nuevo): este plan. Los demás archivos existen y fueron verificados.

## Tests a escribir
Pruebas deterministas con TestBackend, estados simulados e inyección de teclas; sin esperas ni procesos externos.

| Test (comportamiento observable) | REQ que cubre |
| --- | --- |
| Con N por revisar, `v` solicita confirmación en el pie ("¿Aprobar N recetas? s confirma; otra tecla cancela"). | REQ-1 |
| Con la pregunta activa, `s` aprueba todas las por revisar y persiste el recetario; otra tecla cancela sin alterar nada y avisa. | REQ-2 |
| Con 0 por revisar, `v` no pregunta y avisa que no hay nada para aprobar. | REQ-3 |
| Espacio o `a` sobre receta aprobada alterna a por revisar (`□`) y persiste el recetario. | REQ-4, REQ-6 |
| Tanda mixta (investigaciones y búsquedas) reporta balance discriminado (por revisar, posibles orígenes, errores); sugiere `v` solo si hay por revisar. | REQ-5 |
| La lista de recetas, la cabecera de contadores y la ayuda muestran `□` en vez de `○`. | REQ-6 |
| En 80×24 con cuentas triples (999 directas, 999 Claude, 999 por revisar), el pie compacta pestañas y `t`, ocupando 77 columnas sin perder `t`, `v` ni ayuda. | REQ-7 |
| Aprobación confirmada afecta solo recetas por revisar; ítems sin receta y excluidos no cambian ni se inicia instalación. | REQ-8 |

## Orden de trabajo
1. Escribir tests de estados, marcas y tecla `v` en `tests/tui.rs`. Implementar en `src/tui/vista.rs` la marca `□` (U+25A1) en `marca`, `LEYENDA` y cabecera, verificando ancho 1 columna (REQ-6).
2. Probar e implementar en `src/tui/app.rs` el modo de confirmación siguiendo el patrón de `ConfirmarTanda` y `ConfirmarInstalacion`: fijar IDs por revisar; `s` ejecuta aprobación masiva y retorna guardado; otra tecla cancela. Si la lista está vacía, emitir aviso directo con `avisar` sin entrar al modo (REQ-1, REQ-2, REQ-3, REQ-8).
3. Verificar que `a` y espacio sigan alternando la aprobación individual en `app.rs` hacia el nuevo glifo `□` (REQ-4).
4. Probar e implementar en `src/tui/app.rs` el seguimiento de tandas mixtas: al concluir, armar aviso discriminando por revisar (investigaciones), posibles orígenes (búsquedas) y fallos, sugiriendo `v` solo si quedaron por revisar pendientes (REQ-5).
5. Diseñar la compactación del pie en `src/tui/vista.rs` para terminales de 80 columnas cuando conviven `t` y `v`:
   - Pestañas: acortar pestañas inactivas (`2 Pend` con 8 columnas y `3 Inst` con 8 columnas; `1 Recetas` conserva 11 columnas; total: 27 columnas).
   - Ayuda: ` ?  ayuda ` completo (11 columnas).
   - Acción `t`: omite el prefijo "armar" quedando ` t  999 + 999 Claude ` (22 columnas).
   - Acción `v`: ` v  aprobar 999 ` (17 columnas).
   - Total peor caso: 27 + 11 + 22 + 17 = 77 columnas (entra en 80 columnas con 3 columnas de margen).
   - Actualizar `siguiente_paso` y ayuda con la nueva tecla (REQ-7).
6. Agregar eventos estructurados de `tracing::info!` en `src/tui/mod.rs` para `aprobar_todas` y fin de tanda, con cantidades y sin datos privados (regla 4.4).
7. Actualizar tabla de teclas y glifo en `README.md`. Correr suite completa y formateo (REQ-1–REQ-8).

## Riesgos
- Tests existentes que asumen ancho fijo del pie en 80 columnas: `el_pie_mide_80_columnas_con_cuentas_de_tres_cifras_incluso_con_filtro` en `tests/tui.rs` valida que las pestañas completas y `t` miden 80 cuando no hay recetas por revisar; la compactación debe activarse únicamente cuando conviven `t` y `v`, preservando los tests actuales.
- Pruebas que validen el glifo histórico `○`: actualizar aserciones puntuales a `□`.

## Prueba
- `cargo fmt --check`: estilo consistente.
- `cargo clippy --all-targets -- -D warnings`: sin advertencias.
- `cargo test --locked`: ejecución de la suite completa, validando aprobación en lote (REQ-1, REQ-2, REQ-8), lista vacía (REQ-3), destildar (REQ-4), aviso de tanda mixta (REQ-5), marca `□` (REQ-6) y límite de 80 columnas en peor caso (REQ-7).
