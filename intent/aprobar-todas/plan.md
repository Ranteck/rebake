# Plan: aprobar todas y destildar (desde spec.md 2026-10-27 e intent.md 2026-10-27)
Estado: aceptado

## Archivos que cambian
- `src/tui/app.rs`: confirmación de `v`, lista vacía, balance de tanda mixta y reuso de `acciones::aprobar`.
- `src/tui/vista.rs`: marca `□`, pie a 78 columnas con `t` y `v`, siguiente paso y ayuda.
- `src/tui/mod.rs`: logging de aprobación en lote y fin de tanda.
- `README.md`: tabla de teclas en Recetas y marca `□` en contadores.
- `tests/tui.rs`: pruebas de teclas, confirmación, prefijo y balance; reuso de 1063.
- `intent/aprobar-todas/plan.md` (nuevo): este plan. Archivos verificados.

## Tests a escribir
Pruebas deterministas con TestBackend y estados simulados; sin procesos externos ni esperas.

| Test (comportamiento observable) | REQ que cubre |
| --- | --- |
| Con N por revisar, `v` pide confirmación en el pie ("¿Aprobar N recetas? s confirma; otra tecla cancela"). | REQ-1 |
| Con la pregunta activa, `s` aprueba con `acciones::aprobar` y persiste; otra tecla cancela sin cambios. | REQ-2 |
| Con 0 por revisar, `v` no pregunta y avisa que no hay nada para aprobar. | REQ-3 |
| Reusar `tests/tui.rs:1063`: espacio o `a` alterna aprobación a `□`, persiste y mantiene comportamiento. | REQ-4, REQ-6 |
| Tanda mixta avisa balance; sugiere `t` con pendientes y lo omite sin ellos; aísla `i` (1704). | REQ-5 |
| Lista, cabecera de contadores y ayuda muestran `□` en vez de `○`. | REQ-6 |
| En 80×24 con cuentas triples (999 directas, 999 Claude, 999 por revisar), el prefijo hasta `v` mide 78 columnas; pestañas, ayuda, `t` y `v` se ven enteros aunque lo posterior se corte. | REQ-7 |
| Aprobación confirmada afecta solo recetas por revisar; ítems sin receta y excluidos no cambian ni instala. | REQ-8 |

## Orden de trabajo
1. Escribir tests de marcas y tecla `v` en `tests/tui.rs`. Implementar en `src/tui/vista.rs` la marca `□` (U+25A1) en `marca`, `LEYENDA` y cabecera, verificando ancho de 1 columna (REQ-6).
2. Probar e implementar en `src/tui/app.rs` la confirmación con el patrón de `ConfirmarTanda`: guardar IDs por revisar; `s` itera llamando a `acciones::aprobar` (sin duplicar lógica) y retorna `Efecto::Guardar`; otra tecla cancela. Si no hay ítems por revisar, `avisar` sin entrar al modo (REQ-1, REQ-2, REQ-3, REQ-8).
3. Reutilizar `tests/tui.rs:1063` (`espacio_y_a_alternan_la_aprobacion_y_guardan`), adaptando las aserciones al nuevo glifo `□` (REQ-4).
4. Probar e implementar en `src/tui/app.rs` el fin de tanda mixta: desglosar en el aviso por revisar, posibles orígenes y fallos; sugerir `v` solo si hay por revisar y `t` solo si restan pendientes (con y sin pendientes); preservar aislamiento de corridas de `i` según `tests/tui.rs:1704` (REQ-5).
5. Diseñar la compactación del pie en `src/tui/vista.rs` para 80 columnas cuando conviven `t` y `v`:
   - Pestañas inactivas compactadas: `2 Pend` (8 cols) y `3 Inst` (8 cols); `1 Recetas` conserva 11 cols (total pestañas: 27 cols).
   - Ayuda con separador: `  ?  ayuda ` ocupa 12 cols (vista.rs:732-734, 843-844).
   - Acción `t`: omite "armar" quedando ` t  999 + 999 Claude ` (22 cols).
   - Acción `v`: ` v  aprobar 999 ` (17 cols).
   - Prefijo total: 27 + 12 + 22 + 17 = 78 cols (entra en 80 cols; las acciones por ítem de vista.rs:867-871 se agregan después y se truncan).
   - En el test de REQ-7, medir el ancho del prefijo completo hasta `v` (78 cols) y verificar visibilidad íntegra de pestañas, ayuda, `t` y `v`. Actualizar `siguiente_paso` y ayuda (REQ-7).
6. Agregar eventos estructurados de `tracing::info!` en `src/tui/mod.rs` para `aprobar_todas` y fin de tanda, con cantidades y sin datos personales (regla 4.4).
7. Actualizar tabla de teclas en Recetas y glifo en `README.md`. Correr suite completa y formateo (REQ-1–REQ-8).

## Riesgos
- Tests de ancho previo (1790): validan 80 cols con pestañas completas y `t` sin recetas por revisar; la compactación debe condicionarse a la convivencia de `t` y `v`.
- Glifo histórico `○`: actualizar aserciones existentes en tests a `□`.

## Prueba
- `cargo fmt --check`: formato.
- `cargo clippy --all-targets -- -D warnings`: compilación y lint limpios.
- `cargo test --locked`: tabla completa: masiva con `acciones::aprobar` (REQ-1, REQ-2, REQ-8), lista vacía (REQ-3), alternar con `□` (REQ-4), tanda mixta con/sin pendientes e `i` aislado (REQ-5), marca (REQ-6), prefijo en 78 cols sin desbordar 80 (REQ-7).
