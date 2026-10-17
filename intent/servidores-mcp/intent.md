# Intención: avisar de los servidores MCP
Autor: Denis · Estado: aceptado · Fecha: 2026-10-17

## Problema
Algunas herramientas no se instalan como plugin ni como skill: Serena se instala con `uv` y se
conecta a Claude Code como servidor MCP, registrado en `.claude.json`. rebake no lo ve, así que
en una PC nueva "se pierde" sin aviso. Denis: "podés ponerlo como situaciones excepcionales,
onda después de instalarlo todo decirle al usuario, tenés que revisar serena, o que busque y
diga los pasos".

## Resultado esperado
1. El escaneo detecta los servidores MCP de cada perfil (nombre y comando).
2. Cada uno queda en el cookbook como caso excepcional: rebake no lo instala solo.
3. Si se conoce su origen, la investigación de siempre busca los pasos en la doc del proyecto
   (para Serena, `uv tool install …`, `serena init` y el `claude mcp add`). Sin origen, se
   pide el link, como con cualquier receta.
4. Al terminar de instalar, Pendientes lo lista: "revisá Serena", con los pasos si los hay.

Éxito: en una PC nueva, después de `p`, Pendientes dice qué servidores MCP faltan y cómo
ponerlos.

## Usuarios y sistemas afectados
Denis; `~/.claude.json` y `~/.claude-personal/.claude.json` (solo `mcpServers`); la pestaña
Pendientes.

## Restricciones
- De `.claude.json` se lee únicamente `mcpServers` (nombre, comando y argumentos). Todo lo demás
  del archivo se descarta al leerlo. `env`, headers y valores con forma de secreto pasan por el
  filtro de secretos y quedan `‹oculto›`.
- Cambia la regla del README "el escaneo nunca lee `.claude.json`": se reescribe para decir
  exactamente qué se lee.
- Reusa escaneo, investigación y Pendientes; los pasos de un servidor MCP son siempre manuales.

## Fuera de alcance
- Instalar o registrar servidores MCP automáticamente.
- Servidores MCP de proyecto (`.mcp.json` de cada repo) y los que trae un plugin, que vuelven
  con el plugin.
- Copiar la configuración propia del servidor (`~/.serena/`).

## Preguntas abiertas
Ninguna.

## Cambios
