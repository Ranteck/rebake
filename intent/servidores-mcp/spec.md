# Spec: avisar de los servidores MCP (desde intent.md 2026-10-17)
Estado: aceptado

## Requisitos
- REQ-1 MUST: el escaneo detecta los servidores MCP de alcance usuario de cada perfil, con su
  nombre, comando y argumentos (o URL, si es remoto). Origen: Resultado 1, "detecta los
  servidores MCP de cada perfil (nombre y comando)".
- REQ-2 MUST: de `.claude.json` solo se usa `mcpServers`; ningún otro dato del archivo llega al
  cookbook, a los avisos ni a Claude. `env`, headers y valores con forma de secreto quedan
  `‹oculto›`. Origen: Restricciones, "se lee únicamente `mcpServers`… quedan `‹oculto›`".
- REQ-3 MUST: cada servidor queda en el cookbook como un ítem nuevo de tipo servidor MCP, por
  perfil, y aparece en Recetas como cualquier otro ítem. Origen: Resultado 2.
- REQ-4 MUST: todos los pasos de un servidor MCP son manuales: `p` y `rebake instalar` nunca
  los ejecutan, aunque la investigación o una edición digan otra cosa. Origen: Resultado 2,
  "rebake no lo instala solo"; Restricciones, "siempre manuales".
- REQ-5 MUST: con origen conocido, investigar arma los pasos desde la doc del proyecto, con su
  cita, incluido el registro en Claude Code (`claude mcp add …` con el comando detectado). Sin
  origen, se pide el link como en cualquier receta. Origen: Resultado 3.
- REQ-6 MUST: Pendientes lista cada servidor MCP aprobado con sus pasos, y `rebake instalar`
  los muestra al terminar. Origen: Resultado 4, "Pendientes lo lista: revisá Serena".
- REQ-7 SHOULD: un servidor aprobado sin pasos igual aparece en Pendientes como "revisá
  <nombre>" con el comando detectado. Origen: Resultado 4, "con los pasos si los hay".
- REQ-8 SHOULD: la comprobación (`c`) dice si el servidor ya está registrado en cada perfil.
  Origen: Éxito, "Pendientes dice qué servidores MCP faltan".
- REQ-9 MUST: el README deja de decir que nunca se lee `.claude.json` y dice qué parte se lee.
  Origen: Restricciones, "se reescribe para decir exactamente qué se lee".
- REQ-10 MUST: no se detectan servidores de `.mcp.json` de proyectos, de los `projects` de
  `.claude.json` ni los que trae un plugin. Origen: Fuera de alcance.

## Capacidades y escenarios
### Detectar (REQ-1, REQ-2, REQ-10)
- GIVEN `~/.claude.json` con `mcpServers.serena` (comando `serena`, argumentos
  `start-mcp-server …`) y datos de cuenta y proyectos
- WHEN `rebake escanear`
- THEN el cookbook tiene el servidor `serena` en el perfil `claude` con su comando, y ningún
  otro dato del archivo
- GIVEN un servidor con `env.API_KEY = "sk-…"` o un header `Authorization`
- WHEN se escanea
- THEN el cookbook muestra `‹oculto›` en esos valores
- GIVEN un servidor solo dentro de `projects` o en un `.mcp.json`
- WHEN se escanea
- THEN no aparece

### Receta manual (REQ-3, REQ-4, REQ-5)
- GIVEN el ítem `serena` con origen `oraios/serena`
- WHEN se investiga
- THEN la receta trae `uv tool install …`, `serena init` y `claude mcp add …`, cada uno
  manual y con su cita
- GIVEN una receta de servidor MCP editada con un paso `auto`
- WHEN se instala
- THEN ese paso no se ejecuta

### Avisar al final (REQ-6, REQ-7)
- GIVEN `serena` aprobado
- WHEN se termina de instalar, en la TUI o con `rebake instalar`
- THEN Pendientes y la salida dicen "revisá serena" con sus pasos, o con el comando detectado
  si no tiene pasos

### Comprobar (REQ-8)
- GIVEN `serena` aprobado y registrado en el perfil `claude`
- WHEN `c` en Instalación
- THEN figura como instalado en ese perfil

## Concerns
- `.claude.json` guarda estado de la cuenta; un error al recortarlo expondría datos. Por eso
  REQ-2 es MUST y lleva test propio.
- Los argumentos de un servidor pueden traer secretos sin forma reconocible (p. ej. un token en
  una ruta); el filtro va por patrones, igual que con los hooks.
- Si el perfil `claude` usa `~/.claude.json` y otro perfil `<dir>/.claude.json`, leer el
  archivo equivocado mezclaría perfiles.
- La investigación puede devolver pasos con `curl … | sh`; como son manuales, los corre Denis
  después de leerlos.

## Cambios
- 2026-10-17 (v0.8.1): los pasos por perfil salen una línea por perfil, el perfil por defecto
  sin `CLAUDE_CONFIG_DIR` (con ella Claude Code lee `~/.claude/.claude.json`). Bug de REQ-6.
- 2026-10-17 (v0.8.2): REQ-6 y REQ-7 valen para toda receta aprobada con pasos manuales, no
  solo los servidores MCP (p. ej. `/codex:setup` de graph-engineer). Pendientes las junta en
  "Pasos a mano (rebake no los corre)", con "revisá X" y una línea por perfil; un paso que
  empieza con `/` dice cómo abrir Claude en ese perfil. El resumen de la instalación cuenta
  cuántas dejaron pasos a mano. Pedido de Denis: "podrían incluir con un aviso o una
  revisión, similar al caso de serena". Aprobado.
