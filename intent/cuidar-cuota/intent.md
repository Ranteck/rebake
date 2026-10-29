# Intención: cuidar la cuota al armar todas
Autor: Denis · Estado: aceptado · Fecha: 2026-10-22

## Problema
`t` ("armar todas") arma de una vez todo lo pendiente. Lo directo no gasta nada, pero cada
receta que investiga Claude y cada búsqueda de "posible origen" es una corrida de `claude -p`,
y aceptar un origen dispara otra. Con el cookbook nuevo de Denis son 66, 41 sin origen: un solo
`t` puede gastar unas 80 corridas de la cuota semanal sin avisar antes. Denis: "cuida tus
tokens", "hay poco tokens".

## Resultado esperado
1. `t` arma al instante todas las recetas directas, sin Claude.
2. Lo que necesita Claude va por tandas de 5: antes de cada tanda, el pie dice cuántas
   corridas va a gastar y espera `s`; cualquier otra tecla cancela. Las tandas no se encadenan: cada una es otra
   decisión.
3. El pie separa la cuenta: lo que se arma gratis y lo que gasta Claude.
4. Aceptar un posible origen avisa que cuesta una corrida más.

## Usuarios y sistemas afectados
Denis; la pestaña Recetas de la TUI (tecla `t`, pie y panel de la derecha); `claude -p`.
Cambia el punto 6 de `recetas-directas` ("también al investigar todas"): `t` sigue buscando
orígenes, pero por tandas.

## Restricciones
- Pantalla simple: entra en 80×24, sin teclas con Shift y sin ventanas nuevas.
- Aprobar de a una o todas juntas con confirmación (ver aprobar-todas); ningún repo adivinado entra sin el OK de Denis.
- Nada de investigar varias cosas en una misma sesión de Claude.
- `i` (de a una) no cambia.

## Fuera de alcance
- Medir tokens o el porcentaje de la cuota: rebake cuenta corridas, no tokens.
- La CLI (`rebake investigar`): ahí Denis elige qué investiga.

## Preguntas abiertas
Ninguna.

## Cambios
- 2026-10-22: tandas de 5, lo que propusieron Codex y Antigravity. Aprobado por Denis.
- 2026-10-22: la tanda se confirma con `s` y no con Enter o Esc, como instalar y salir: Enter
  ya acepta un posible origen. Lo revisaron Codex y Antigravity. Aprobado por Denis.
- 2026-10-27: aprobar todas las por revisar con confirmación, en aprobar-todas. Aprobado por Denis.
- 2026-10-29: `t` arma solo lo que tiene origen y deja de buscar orígenes, por menos-ruido; aprobado por Denis.
