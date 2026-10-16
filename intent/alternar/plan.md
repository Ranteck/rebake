# Plan: aprobar y excluir con la misma tecla, ida y vuelta (desde spec.md 2026-10-16 e intent.md 2026-10-16)
Estado: aceptado

## Archivos que cambian
Existentes (verificados):
- `src/acciones.rs`: `alternar_aprobacion` (usa `aprobar` para ir, vuelve a por revisar
  desde aprobada) e `incluir` (excluida → por revisar si tiene pasos, sin receta si no;
  borra el motivo). `aprobar` y `excluir` no cambian: `tests/cli.rs` usa `aprobar`.
- `src/tui/app.rs`: en `recetas()`, espacio y `a` llaman a `alternar_aprobacion`; `x` sobre
  un excluido llama a `incluir` en lugar de pedir motivo.
- `src/tui/vista.rs`: `teclas_de_receta` con "aprobar"/"desaprobar" y "excluir"/"volver a
  incluir" según el estado; texto de la ayuda.
- `tests/acciones.rs`, `tests/tui.rs`: tests nuevos.
- `README.md`: tabla de teclas (`espacio` / `a`, `x`).

## Tests a escribir
| Test (comportamiento observable) | REQ que cubre |
| --- | --- |
| `alternar_aprobacion`: por revisar → aprobada → por revisar | REQ-1 |
| Espacio y `a` dan el mismo efecto y mensaje sobre la misma receta | REQ-1, REQ-5 |
| Espacio sobre sin receta y `a` sobre excluido: estado igual, `Efecto::Nada`, mensaje con el motivo | REQ-2 |
| `x` + "duplicado" + Enter excluye; motivo vacío no | REQ-3 |
| `incluir`: excluida con pasos → por revisar; sin pasos → sin receta; motivo borrado | REQ-4 |
| `x` sobre excluido devuelve `Efecto::Guardar` sin abrir el campo de motivo | REQ-4, REQ-5 |
| El pie ofrece "desaprobar" en una aprobada y "volver a incluir" en una excluida | REQ-6 |
| La ayuda menciona que espacio guarda en Recetas y solo marca en Instalación | REQ-7 |
| Espacio en Instalación no cambia el cookbook (el test `marcar_no_cambia_el_cookbook` ya existe y sigue igual) | REQ-8 |

Los tests del pie que hoy esperan `a  aprobar   e  editar…` se actualizan a
`espacio/a  aprobar`: el cambio de texto es lo pedido (REQ-6).

## Orden de trabajo
1. Tests de `acciones` en rojo; `alternar_aprobacion` e `incluir` (REQ-1, 2, 4).
2. Tests de la TUI en rojo; teclas en `app.rs` (REQ-1 a 5).
3. Pie y ayuda en `vista.rs` (REQ-6, 7).
4. README; `cargo test`, `cargo clippy -- -D warnings`.

## Riesgos
- Espacio en Recetas antes no hacía nada: alguien que lo apriete por costumbre ahora aprueba.
  El mensaje del pie lo dice y se deshace con la misma tecla.
- El orden de los brazos del `match` de teclas: espacio debe atenderse en Recetas antes del
  brazo genérico, y no en Instalación ni Pendientes.

## Prueba
- `cargo test` (193 actuales más los nuevos) y `cargo clippy --all-targets -- -D warnings`.
- Manual (Denis): en Recetas, espacio dos veces sobre una receta (aprueba y desaprueba),
  `x` + motivo y `x` de nuevo (excluye y recupera); en Instalación, espacio sigue marcando.
