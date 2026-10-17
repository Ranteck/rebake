# Plan: avisar de los servidores MCP (desde spec.md 2026-10-17 e intent.md 2026-10-17)
Estado: aceptado

## Archivos que cambian
Existentes (verificados): `src/modelo.rs`, `src/escaner/mod.rs`, `src/escaner/comandos.rs`,
`src/investigador.rs`, `src/instalador.rs`, `src/checklist.rs`, `src/servicio.rs`,
`src/tui/vista.rs`, `tests/investigador.rs`, `tests/instalador.rs`, `tests/checklist.rs`,
`README.md`, `intent/spec.md` (nota "Desde v0.8.0").
Nuevos: `src/escaner/mcp.rs` y `tests/escaner_mcp.rs`.

## Diseño
- **Tipo nuevo** `Tipo::Mcp` (`tipo = "mcp"` en el TOML, id `mcp:<nombre>`). La vista lo agrupa
  como "Servidores MCP". Un cookbook viejo sigue leyéndose igual: solo se suma una variante.
- **Detectar (REQ-1, 2, 10):** `escaner/mcp.rs` lee el `.claude.json` del perfil con la misma
  regla que `instalador::comando`: el perfil `~/.claude` usa `~/.claude.json`, cualquier otro
  `<dir>/.claude.json`. Lo parsea a un tipo que solo tiene `mcpServers` (serde ignora el resto,
  que no queda en memoria) y de cada servidor toma `command`, `args` o `url`. `env` y `headers`
  no se leen. La línea resultante pasa por `secretos::ocultar` y queda en la pista:
  `servidor MCP: serena start-mcp-server …`. Sin archivo, no hay aviso; JSON inválido avisa
  como hoy `leer_json`, sin mostrar contenido. No toca `projects`.
- **Origen:** si `command` es un binario, se reusa `comandos::herramienta` (PATH, pacman,
  `url_en_binario`) solo para sacar el repo; no se agrega un ítem `herramienta:`. Sin repo,
  queda "sin origen" y `i` pide el link (Enter vacío busca en la web).
- **Pista estable:** en `agrupar`, un `mcp:` conserva la pista aunque tenga fuente, porque es
  el comando que se registra y lo que muestra Pendientes.
- **Investigar (REQ-5):** sin receta directa: va a `claude -p` con la pista (ya se manda hoy).
  El prompt suma una regla para `mcp`: pasos de instalación con su cita y al final
  `claude mcp add <nombre> -- <comando detectado>`. Al guardar, todo paso de un `mcp` queda
  `manual`, y `verificar` pasa a `claude mcp get '<nombre>' >/dev/null 2>&1` (REQ-8; tarda
  ~1,8 s, dentro del tope).
- **No instalar (REQ-4):** `instalar_item` no toma pasos `auto` de un ítem `mcp`, aunque el
  TOML diga otra cosa. La protección vive en el instalador, no en la receta.
- **Pendientes (REQ-6, 7):** los pasos manuales de un `mcp` aprobado ya salen por
  `checklist::pasos_manuales`. Se suma una sección "Servidores MCP" en `texto_checklist` y en
  la pestaña: "revisá serena" y debajo sus pasos, o la pista si no tiene pasos.
- **README (REQ-9):** "El escaneo nunca lee `.credentials.json` ni `~/.codex/auth.json`; de
  `.claude.json` lee solo `mcpServers` (comando y argumentos, sin `env` ni headers)". Más una
  fila en "Cómo arma cada receta".

## Tests a escribir
| Test (comportamiento observable) | REQ |
| --- | --- |
| `.claude.json` con `mcpServers.serena`, `oauthAccount`, `projects` y un token: el escaneo da `mcp:serena` con su comando y ninguno de los otros datos aparece en el cookbook | REQ-1, REQ-2 |
| Servidor con `env.API_KEY`, `headers.Authorization` y `--token x` en args: nada de eso queda visible | REQ-2 |
| Servidor remoto (`url`): aparece con la URL sin credenciales | REQ-1 |
| Perfil `~/.claude` lee `~/.claude.json`; perfil `personal` lee el suyo; no se mezclan | REQ-1 |
| Servidor solo en `projects` o en un `.mcp.json`: no aparece | REQ-10 |
| `cookbook.toml` con `tipo = "mcp"` se lee y se guarda igual | REQ-3 |
| Investigar un `mcp` con un `claude` falso que devuelve un paso `auto`: queda `manual`, con `verificar` de `claude mcp get` | REQ-4, REQ-5, REQ-8 |
| `mcp` sin origen: no se investiga solo y pide link | REQ-5 |
| Instalar un `mcp` aprobado con un paso `auto` editado a mano: el comando no corre | REQ-4 |
| Pendientes y `texto_checklist` listan "revisá serena" con sus pasos; sin pasos, con el comando | REQ-6, REQ-7 |

## Orden de trabajo
1. Rama `feat/servidores-mcp` desde `main` (después de mergear `feat/alternar`, que está sin
   publicar, o desde ella si preferís sacar las dos juntas).
2. T1: tipo, escaneo y tests de escaneo (REQ-1, 2, 3, 10), tests primero.
3. T2: investigar, instalar y Pendientes (REQ-4 a 8), tests primero.
4. T3 (yo): README y nota del spec raíz (REQ-9).
   T4: "posible origen", según `intent/recetas-directas/plan.md` (Cambios 2026-10-17).
5. Verificación: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test --locked`.
6. Prueba real y publicar v0.8.0.

## Riesgos
- Leer el `.claude.json` equivocado mezclaría perfiles; lo cubre el test de los dos perfiles.
- El filtro de secretos va por patrones: un token en una ruta de `args` podría quedar visible.
- `claude mcp get` corre bajo `CLAUDE_CONFIG_DIR`; si una versión de Claude Code cambia dónde
  guarda los servidores, la comprobación daría "falta" en vez de error.

## Prueba
- `rebake escanear` en esta PC: aparece `mcp:serena` en el perfil `claude`, y
  `git diff` del cookbook no muestra nada de la cuenta.
- `i` con `https://github.com/oraios/serena`: la receta trae `uv tool install -p 3.13
  serena-agent`, `serena init` y `claude mcp add`, todo manual con cita.
- Aprobada, `c` la marca instalada; `rebake instalar --dry-run` no la ejecuta y la checklist
  dice "revisá serena".
