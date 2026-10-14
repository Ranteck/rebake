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
