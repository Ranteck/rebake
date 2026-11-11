# Plan: qué hace cada ítem (desde spec.md 2026-10-31 e intent.md 2026-10-30)
Estado: aceptado

## Archivos que cambian
Existentes, verificados:
- `src/modelo.rs`: `Item` gana una descripción opcional que no se escribe si falta.
- `src/escaner/mod.rs`: `Hallazgo` gana la descripción y `agrupar` se queda con la primera, como
  con `fuente`. Acá va el lector del `description` del frontmatter: plano, entre comillas
  simples o dobles, o en bloque `>`/`|` con líneas indentadas, unido en un párrafo. El texto
  pasa por `secretos::ocultar` y el home se vuelve `~`, como en `escaner/comandos.rs`.
- `src/escaner/archivos.rs`: `skills` lee el `SKILL.md` de cada skill no sincronizada.
- `src/escaner/plugins.rs`: un marketplace toma `description` o `metadata.description` de su
  `marketplace.json`, en `installLocation`. Un plugin, la de `.claude-plugin/plugin.json` en su
  `installPath` o, si no la trae, la de la entrada del catálogo que ya ubica `repo_del_plugin`.
- `src/fusion.rs`: un ítem encontrado toma la descripción del escaneo, aunque venga vacía; uno
  nuevo la copia; los no encontrados no se tocan.
- `src/tui/vista.rs`: `lineas_detalle` recibe el ancho interior del panel y, tras la primera
  línea, agrega la descripción partida con `envolver` (hasta 4 líneas, «…» si no entra) o «sin
  descripción», también en la rama de hooks de una herramienta. Se ajustan Armar e Instalar.
- `src/main.rs` y `src/tui/mod.rs`: el evento de escaneo suma `con_descripcion`.
- `Cargo.toml`: versión 0.17.0.
- Tests: `tests/escaner_archivos.rs`, `tests/escaner_plugins.rs`, `tests/fusion.rs`,
  `tests/archivo.rs`, `tests/tui.rs`.

Nuevos: ninguno. Sin crates nuevos: el lector cubre los cuatro formatos que hay en esta PC
(plano, comillas dobles, simples y `>`).

## Tests a escribir
| Test (comportamiento observable) | REQ que cubre |
| --- | --- |
| Skill con `description` plana, entre comillas y en bloque `>` de varias líneas: el hallazgo trae el texto completo en un párrafo | REQ-2, REQ-5 |
| Plugin con descripción en `plugin.json`; plugin sin ella toma la de su entrada del marketplace; marketplace con `description` y con `metadata.description` | REQ-2 |
| Escaneo de fixtures con `PATH` vacío trae las descripciones; en la TUI, `s` escanea sin pedir confirmación ni correr Claude | REQ-3 |
| Fusión: ítem encontrado con descripción nueva la reemplaza, sin descripción la borra, excluido también se actualiza; ítem ausente conserva la guardada | REQ-4 |
| Cookbook con descripción se lee y se guarda igual; uno sin el campo se sigue leyendo | REQ-4 |
| Armar 80×24: skill con descripción la muestra bajo la primera línea, tal cual; también excluida visible | REQ-1, REQ-5 |
| Instalar: ítem ausente en esta PC muestra la descripción guardada | REQ-1, REQ-4 |
| Servidor MCP, herramienta, hook de herramienta y skill sin `description` dicen «sin descripción», no la pista | REQ-6 |
| Descripción con un token y `/home/<usuario>/…`: el cookbook guarda el token oculto y `~/…` | REQ-7 |
| Descripción de 1000 caracteres en 80×24: 4 líneas, la cuarta termina en «…» y se ve la línea que sigue | REQ-8 |

## Orden de trabajo
Codex implementa en un worktree hijo de Orca (`--base-branch main`), un commit por paso;
Antigravity revisa en otro, en solo lectura.
1. Modelo, cookbook y fusión, con sus tests (REQ-4).
2. Lector del frontmatter y skills, con secretos y home (REQ-2, REQ-3, REQ-5, REQ-7).
3. Plugins y marketplaces (REQ-2).
4. Panel en Armar e Instalar y evento de escaneo (REQ-1, REQ-6, REQ-8).
5. Checks, prueba en el cookbook real, revisión, integración en `main` y publicación de v0.17.0.

## Riesgos
- Un rebake anterior a 0.17.0 no abre el cookbook guardado con descripciones: rechaza campos
  desconocidos. Se publica antes de escanear el cookbook real; `install.sh` baja la última.
- Frontmatter raro (comillas sin cerrar, `description` anidado): sin descripción, sin frenar el
  escaneo. Solo un error de lectura distinto de "no existe" suma aviso, como en `leer_json`.
- Si el ancho que recibe `lineas_detalle` incluye los bordes, ratatui vuelve a partir y salen
  5 líneas; lo atrapa el test de REQ-8.
- La Fase 5 de menos-ruido también toca `src/tui/vista.rs` y `src/tui/app.rs`: esta feature se
  integra primero y la Fase 5 parte de ese `main`.

## Prueba
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` y `cargo test` verdes.
- Build release y `rebake escanear` sobre una copia del cookbook real: skills, plugins y
  marketplaces ganan descripción y el diff solo suma ese campo (REQ-2, REQ-4, REQ-7).
- PTY 80×24: `plugin:mutation-testing@trailofbits` muestra su descripción en hasta 4 líneas, un
  servidor MCP dice «sin descripción» e Instalar muestra la guardada (REQ-1, REQ-6, REQ-8).
- El evento de escaneo trae `con_descripcion` en la salida real.
