# Intención: aprobar y excluir con la misma tecla, ida y vuelta
Autor: Denis · Estado: aceptado · Fecha: 2026-10-16

## Problema
En Recetas, `a` aprueba pero no hay vuelta atrás: una receta aprobada no se puede
desaprobar, y una excluida solo vuelve editándola con `e`. Denis: "when I select an item
with a, it becomes approved. But if I want to exclude, shouldn't it use the same key? as a
toggle?" y "why isn't the space bar used as a toggle between accept and don't accept?".

## Resultado esperado
En la pestaña Recetas:
1. Espacio y `a` alternan la aprobación: por revisar → aprobada → por revisar.
2. `x` alterna la exclusión: en un ítem normal pide el motivo y lo excluye; en uno excluido
   lo devuelve (a por revisar si tiene receta, a sin receta si no).
3. El pie y la ayuda dicen qué hace cada tecla según el ítem elegido, y que en Recetas el
   cambio queda guardado en el cookbook (en Instalación, espacio solo vale para esa
   instalación).

Éxito: Denis aprueba y desaprueba con espacio, excluye y recupera con `x`, sin abrir el
editor.

## Usuarios y sistemas afectados
Quien arma el cookbook en la TUI de rebake; el cookbook (estado de cada ítem).

## Restricciones
- Las del intent raíz.
- Aprobar sigue pidiendo receta: un ítem sin receta no se aprueba.
- Excluir sigue pidiendo motivo.
- Aprobar y excluir quedan en teclas distintas: una tecla extra no debe excluir sin querer.
- Ninguna tecla depende de Shift (v0.6.7).

## Fuera de alcance
- Aprobar o excluir en lote.
- Cambiar qué hace espacio en Instalación o en Pendientes.
- Rondas de estados con una sola tecla (por revisar → aprobada → excluida).

## Preguntas abiertas
Ninguna.

## Cambios
- 2026-10-29: `x` excluye de un toque y el motivo pasa a ser opcional, por menos-ruido; aprobado por Denis.
