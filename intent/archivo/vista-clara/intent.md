# Intención: una pantalla que se entienda
Autor: Denis · Estado: aceptado · Fecha: 2026-10-15

## Problema
Denis abre `rebake` y no entiende lo que ve: "I'm still not understanding". Con su cookbook
real (82 ítems) la pantalla muestra, entre otras cosas:
- Símbolos sin leyenda (`?`, `○`, `✓`, `✗`) y la palabra "receta" para cosas que todavía
  no tienen receta.
- No dice en qué paso del camino está (escanear → armar recetas → aprobar → instalar) ni
  cuál es el próximo: con 62 "por revisar" no aparece "Siguiente:".
- Nombres cortados en la lista (`claude-md-management@claude-plugins-officia`).
- Un detalle con jerga interna (`metadatos`, `Doc: receta directa…`, `por perfil`).
- La cabecera se corta en 80 columnas, y la pestaña Instalación vacía muestra teclas que
  no hacen nada ahí.
- Las pestañas no dicen para qué son: "what feels weird are the Checklist and Instalación
  tabs. What are these for?"

## Resultado esperado
"Use /tui-design and /unix-cli to improve the view": que la pantalla siga esas guías y
alguien que la abre por primera vez entienda, sin leer el README:
1. Qué significa cada marca y cada estado.
2. En qué paso está el cookbook y cuál es la próxima tecla útil.
3. De dónde sale cada ítem y qué va a ejecutar su receta, en palabras simples.
4. Que se lea bien en 80x24 y en terminales anchas, con y sin color.

Éxito: con el cookbook real, Denis abre `rebake` y sabe qué hacer sin preguntar.

## Usuarios y sistemas afectados
Quien use la TUI de rebake; la salida de los subcomandos (`escanear`, `instalar`, etc.) solo
donde viole las reglas de unix-cli.

## Restricciones
- Las del intent raíz: TUI en Rust con ratatui, sin dependencias nuevas salvo que haga falta.
- Las teclas actuales no cambian de significado ni dependen de Shift (v0.6.7).
- Sin color obligatorio: cada estado se distingue con texto o forma y se respeta `NO_COLOR`.

## Fuera de alcance
- Comportamiento nuevo: aprobar en lote, otro flujo de investigación o instalación.
- Cambiar el formato del cookbook.

## Preguntas abiertas
Ninguna.

## Cambios
- 2026-10-15: Denis no sabía para qué sirven las pestañas Checklist e Instalación; se suma
  que cada pestaña diga para qué es. Aprobar en lote y ver la diferencia entre el cookbook y
  la PC quedan para otras features (la segunda en `intent/diferencias/`). Aprobado por Denis.
- 2026-10-29: cambian las teclas, el orden de las pestañas y la acción contextual de Enter, por menos-ruido; aprobado por Denis.
