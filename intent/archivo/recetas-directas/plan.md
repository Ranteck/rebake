# Plan: recetas directas (desde spec.md 2026-10-14 e intent.md 2026-10-14)
Estado: aceptado

## Archivos que cambian
Todos existen (verificado); no se crean archivos: `src/investigador.rs`, `src/servicio.rs`,
`src/main.rs`, `src/tui/app.rs`, `src/tui/vista.rs` (etiqueta de `Via::Busqueda`: "origen buscado
en la web"), `tests/investigador.rs` (sus dos `Trabajo { … }` suman el campo nuevo),
`tests/cli.rs`, `tests/tui.rs`, `README.md` e `intent/spec.md` (nota "Desde v0.5.0").

## Diseño
- `Trabajo` suma la receta directa opcional. `trabajo_para` la calcula y `investigar` la devuelve
  sin ejecutar Claude. Así la CLI y la pantalla la reciben por el mismo canal, sin cambios en
  `en_paralelo` ni en `aplicar`.
- Es directa: marketplace con repo, plugin con fuente, skill con `via = metadatos` (solo sale del
  lock de skills.sh, `src/escaner/archivos.rs:45`). Las herramientas con metadatos no entran. La
  receta lleva el repo de la fuente, así que `aplicar` conserva la vía.
- Cada paso es `por_perfil`, con nota "receta directa", y empieza con una guarda que corta con el
  aviso por stderr (plugin: su marketplace en `claude plugin marketplace list`; skill: `npx` en el
  PATH). El instalador ya muestra esas líneas (`src/main.rs:333`, `src/tui/app.rs:487`).
- Comprobación: el id en `claude plugin list`, el nombre en `claude plugin marketplace list`, o
  `$CLAUDE_CONFIG_DIR/skills/<nombre>/SKILL.md`.
- Modelo: `argumentos` recibe el modelo; `REBAKE_MODELO` (por defecto `sonnet`) se lee junto a
  `binario_claude`.
- Sin origen: `trabajos` sin ids devuelve aparte los pendientes sin fuente y no los investiga.
  Con ids explícitos (`rebake investigar <id>`) se busca como hoy: es un pedido. En la pantalla,
  `i`/`r` sobre una receta sin fuente abre `Entrada::Link`; Enter vacío deja de ser error e
  investiga con búsqueda; Esc ya descarta.

## Tests a escribir
| Test (comportamiento observable) | REQ |
| --- | --- |
| Marketplace con repo: paso `claude plugin marketplace add <repo>` por perfil, por revisar, con un claude falso que falla si se lo llama | REQ-1, REQ-5 |
| Plugin: paso `claude plugin install <id>` por perfil, comprobación con el id, sigue requiriendo su marketplace | REQ-2, REQ-6 |
| Skill del lock: paso `npx -y skills add … --skill <nombre> -g -a claude-code -y`; nota "receta directa" | REQ-3, REQ-6 |
| Skill por symlink y herramienta con metadatos van a Claude | REQ-7 |
| Instalar un plugin con un `claude` falso sin el marketplace: falla con "falta el marketplace …" y las demás siguen | REQ-4 |
| Instalar una skill directa sin `npx` en el PATH: falla con "falta npx: instalá Node.js" | REQ-4 |
| `argumentos` lleva `--model sonnet`; con `REBAKE_MODELO=opus`, el claude falso recibe `--model opus` | REQ-7 |
| `rebake investigar` con 3 pendientes y 1 sin origen: investiga 2 y lista la sin origen | REQ-8, REQ-10 |
| `I` no toca recetas por revisar ni aprobadas, y avisa cuántas quedaron sin origen | REQ-10 |
| `i` sobre una receta sin origen abre el pedido del link y no investiga | REQ-8 |
| En el pedido, Esc no cambia nada; Enter vacío investiga; un link investiga desde ese repo | REQ-8, REQ-9 |
| Investigar sin fuente deja `via = busqueda`, que el detalle muestra como "origen buscado en la web" | REQ-9 |

## Orden de trabajo
Workers de orquestar en worktrees desde la rama; yo reviso, commiteo e integro.
1. **T1 (ejecutor):** `investigador.rs` y `tests/investigador.rs`: receta directa, guardas,
   comprobación y modelo. REQ-1 a REQ-7.
2. **T2 (ejecutor, en paralelo con T1):** `servicio.rs`, `main.rs`, `tui/app.rs`, `tui/vista.rs`,
   `tests/cli.rs` y `tests/tui.rs`. Las sin origen y el pedido del link. REQ-8 a REQ-10.
3. **T3 (mecánico):** README y nota en `intent/spec.md`.
4. **Verificador:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test --locked`.
5. **Prueba real** (abajo) y tu OK para publicar v0.5.0.

## Riesgos
- Choque de T1 y T2: T2 solo usa `trabajo_para`, cuya firma no cambia.
- Un `grep` sobre la salida de `claude plugin list` depende de su formato. Uso `grep -F` con el id
  completo, que figura literal (verificado hoy).
- `claude plugin install` en una PC sin sesión previa no se probó: lo cubren la guarda y el
  marketplace.

## Prueba
- Tests: alrededor de 158 (145 más unos 13).
- Copia de tu cookbook real con un `claude` falso que anota cada llamada: `rebake investigar`
  deja por revisar marketplaces, plugins y skills del lock sin llamarlo, y lista las sin origen
  (REQ-1 a REQ-3, REQ-10).
- En la pantalla real: `i` sobre la statusline pide el link, Esc no cambia nada, Enter vacío
  investiga (REQ-8, REQ-9), y `c` marca "instalada" los plugins que ya tenés (REQ-6).

## Cambios 2026-10-17 (aprobado por Denis): posible origen (REQ-11 a REQ-16)
Sale junto con `servidores-mcp` en v0.8.0, como T4 de ese plan.
- **Archivos:** `src/modelo.rs` (campo `posible = { repo, motivo }` del ítem, opcional: un
  cookbook viejo se lee igual), `src/investigador.rs` (búsqueda corta: prompt y esquema JSON
  propios, `WebSearch`/`WebFetch` sin Bash ni escritura, mismo filtro de secretos que hoy;
  el repo pasa por `normalizar_url` y `es_repo_seguro`), `src/acciones.rs` (aceptar y
  descartar), `src/tui/app.rs` (`i` y `t` lanzan la búsqueda; Enter y `n` sobre un posible),
  `src/tui/mod.rs` (canal de resultados como el de investigar; cuadro a cuadro del spinner con
  el `poll` de 100 ms que ya existe), `src/tui/vista.rs` (spinner en la fila y en el panel;
  panel con el posible y sus teclas), `README.md`.
- **Tests:** `tests/investigador.rs`: con un `claude` falso, la búsqueda guarda el posible y no
  escribe receta (REQ-11); un repo inseguro devuelto por Claude se descarta (REQ-11).
  `tests/acciones.rs`: aceptar mueve el posible a la fuente con vía búsqueda y lo borra;
  descartar lo borra (REQ-13, REQ-14). `tests/tui.rs`: `i` sin origen devuelve el efecto de
  buscar (REQ-11); `t` incluye los sin origen (REQ-12); el panel muestra posible, motivo y
  teclas (REQ-13); Enter sobre un posible investiga, `n` no (REQ-14); con una búsqueda en
  curso la fila y el panel cambian de cuadro entre dos dibujos (REQ-15); un ítem con posible
  no se instala ni se investiga solo (REQ-14). `tests/cli.rs`: `rebake investigar` sin ids
  no busca orígenes (REQ-16). `tests/archivo.rs`: el posible sobrevive guardar y leer (REQ-13).
- **Riesgo:** la búsqueda corta usa la web y puede tardar; corre en el mismo pool en paralelo
  que investigar, así que no bloquea la pantalla.
