# Plan: una pantalla que se entienda (desde spec.md 2026-10-15 e intent.md 2026-10-15)
Estado: aceptado

## Archivos que cambian
Existentes (verificados):
- `src/tui/vista.rs`: casi todo el cambio visible (cabecera, leyenda, línea de pestaña,
  lista, detalle, pie, ayuda, colores).
- `src/tui/app.rs`: `Modo::Ayuda` con `?`/`Esc` y el campo `color: bool`. El variante
  `Pestana::Checklist` no se renombra: cambia solo el texto que se ve.
- `src/tui/mod.rs`: lee `NO_COLOR` una vez al arrancar y lo pasa a `App` (config en un solo
  lugar).
- `Cargo.toml`: `unicode-width` como dependencia directa. Ya está compilada por ratatui
  (misma versión del lock), así que no suma código nuevo al árbol.
- `tests/tui.rs`: tests nuevos y textos actualizados.
- `README.md`: tabla de teclas (`?`), nombre Pendientes, línea que explica el orden de uso.

Nuevo, sin commit: `examples/captura.rs` se borra al final (era para comparar el antes y el
después con el cookbook real).

## Tests a escribir
| Test (comportamiento observable) | REQ que cubre |
| --- | --- |
| Recetas muestra la leyenda de marcas | REQ-1 |
| El camino resalta el paso según los ítems (vacío → escanear; pendientes con origen → armar; por revisar → aprobar; solo aprobadas → instalar) | REQ-2 |
| Con 62 por revisar, el pie dice "por revisar" con la cantidad y `a` | REQ-3 |
| `claude-md-management@claude-plugins-official` en 40 columnas se ve como `claude-md-management`, y un nombre largo termina en `…` | REQ-4 |
| Un nombre con emoji o CJK no corre la columna de estado | REQ-4 |
| El detalle de un marketplace dice "Viene de" y "lo dice Claude Code", sin `metadatos` ni `Doc:` | REQ-5 |
| En 80x24 la cabecera conserva los cuatro contadores | REQ-6 |
| Instalación sin aprobadas: el pie no ofrece `t`, `c`, `p` ni `espacio` | REQ-7 |
| Con color apagado, el buffer no tiene colores y las marcas siguen | REQ-8 |
| `?` abre la ayuda, `Esc` la cierra y la selección queda igual | REQ-9 |
| `?` escrito en el filtro se suma al texto | REQ-9 |
| Cada pestaña muestra su línea de para qué es; `2` abre Pendientes | REQ-10 |

Los tests que hoy buscan `Fuente:`, `por perfil`, `Checklist` o `Pasos` se actualizan al
texto nuevo. No se debilitan: el cambio de texto es lo pedido (Concerns del spec).

## Orden de trabajo
1. Tests de REQ-1, 2, 3 y 10 en rojo; leyenda, camino, línea de pestaña y "Siguiente:" con
   la cantidad de por revisar.
2. REQ-4: truncar por ancho en pantalla con `…` y sacar `@claude-plugins-official` del nombre
   en la lista. Reemplaza el `shortcut:` actual de `linea_item`.
3. REQ-5: textos del detalle (`nombre_via`, etiquetas, pasos).
4. REQ-6 y REQ-7: cabecera abreviada cuando no entra; pie de Instalación vacía.
5. REQ-8: estilos semánticos en una tabla dentro de `vista.rs`; `color` en `App` desde
   `tui/mod.rs`.
6. REQ-9: `Modo::Ayuda` y su dibujo como panel encima del cuerpo.
7. README, `cargo test`, `cargo clippy`, captura con el cookbook real en 80x24 y 120x32,
   borrar `examples/`.

## Riesgos
- Agregar una línea de pestaña y una de leyenda le quita dos filas a la lista en 80x24.
  Mitigación: la leyenda va dentro del borde del panel (título inferior) y no ocupa fila.
- Un `Modo` nuevo puede tragarse teclas: la ayuda solo atiende `?`, `Esc` y `q`.
- Los colores fijos (verde, amarillo, rojo) se usan como colores ANSI con nombre, que la
  terminal adapta a su paleta. No se fija RGB.

## Prueba
- `cargo test` (los 181 actuales más los nuevos) y `cargo clippy -- -D warnings`.
- Captura con el cookbook real en 120x32 y 80x24: se ven leyenda, camino con "aprobar",
  "Siguiente: 62…", nombres sin `@claude-plugins-official` y la cabecera completa.
- Manual (Denis): abrir `rebake` en kitty, apretar `?`, `1` `2` `3`, y probar con
  `NO_COLOR=1 rebake`. Al salir, el cursor y el eco de la shell vuelven normales.
