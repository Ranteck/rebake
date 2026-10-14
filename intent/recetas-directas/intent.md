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
2. Quedan "por revisar", como las investigadas.
3. Claude se usa solo para lo que no tiene forma fija (hooks, imports, statusline, herramientas,
   skills sin lock).
4. La receta directa dice de dónde salió y trae su comprobación, para que `c` diga si ya está
   instalada.
5. Lo que investiga Claude corre con Sonnet, cambiable con `REBAKE_MODELO`.
6. Sin origen conocido, la pantalla pide el link al investigar, en vez de buscar en la web un
   repo "parecido". Sin link no se investiga y no se instala.

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
