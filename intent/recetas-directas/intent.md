# Intención: recetas directas, sin investigar
Autor: Denis · Estado: aceptado · Fecha: 2026-10-14

## Problema
Hoy toda receta sale de `claude -p`, una corrida por cosa. Pero "no siempre es necesario correr
claude para instalar las skills, hay muchas skills que se instalan por comando ya sea npx o
haciendo claude install o add". De las 73 recetas pendientes de Denis, 42 son marketplaces y
plugins cuyo comando rebake ya conoce: una corrida de Claude por cada una es cuota gastada.

## Resultado esperado
1. Al investigar, rebake arma solo la receta de lo que tiene instalador conocido, sin `claude -p`:
   - marketplace con repo: `claude plugin marketplace add <repo>`;
   - plugin: `claude plugin install <nombre>@<marketplace>`;
   - skill instalada con skills.sh: `npx skills add <dueño/repo> --skill <nombre>`.
   Si falta `npx` o el marketplace del plugin, la receta lo avisa al instalar.
2. Quedan "por revisar", como las investigadas.
3. Claude queda para lo que no tiene forma fija (hooks, statusline, herramientas).
4. La receta directa dice de dónde salió y trae su comprobación, para que `c` diga si ya está
   instalada.
5. Lo que investiga Claude corre con Sonnet, cambiable con `REBAKE_MODELO`.
6. Sin origen conocido, la pantalla pide el link. Si Denis no lo sabe, puede pedir que lo busque
   en la web, y la receta dice que su origen fue buscado. Sin link ni búsqueda no se investiga y
   no se instala.

## Usuarios y sistemas afectados
Denis y su cookbook; el paso "investigar" de rebake; Claude Code y skills.sh.

## Restricciones
- No cambia el paso de aprobación ni la instalación.
- Solo instaladores oficiales del tipo de cosa: no se inventa un comando para lo que no lo tiene.
- Una receta ya escrita o aprobada no se pisa.

## Fuera de alcance
- Aprobar automáticamente.
- Skills que son un symlink a un repo propio: siguen yendo a Claude.
- Verificar que el comando funcione antes de aprobar.
- Investigar varias cosas en una sola sesión de Claude (lotes).

## Preguntas abiertas
Ninguna.

## Cambios
- 2026-10-14: comprobación (punto 4), Sonnet (punto 5), link pedido sin origen (punto 6) y sin
  lotes. Decisión de Denis tras probar tres modelos: dos inventaron el repo de la statusline.
- 2026-10-14: aviso si falta `npx` o el marketplace (punto 1) y búsqueda web a pedido cuando no
  se sabe el link (punto 6). Decisión de Denis.
