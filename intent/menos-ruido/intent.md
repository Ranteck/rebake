# Intención: rediseño de Recetas, menos ruido y un final claro
Autor: Denis · Estado: aceptado · Fecha: 2026-10-29

## Problema
"Esto necesita una vuelta de rosca para que sea más entendible"; "hay mucho que es ruido":
- Aparece lo que Claude sincroniza desde la cuenta (pdf, pptx…): "los que vienen imports de
  claude, no tienen que aparecer porque ya hace el sync el mismo claude".
- Lo excluido y lo aprobado tapan lo pendiente.
- No hay final claro: Enter no hizo nada y `t` gastó corridas buscando orígenes: "¿Para qué, si
  yo antes había estado tomándome el tiempo de seleccionar una por una cuál entraba y cuál no?".

## Resultado esperado
1. Lo sincronizado por la cuenta de Claude no aparece, no se investiga ni se instala, en ninguna
   PC ni desde la CLI, aunque ya esté en el cookbook.
2. La lista se parte en POR RESOLVER arriba y LISTAS abajo; lo directo llega ya armado, gratis.
   Lo excluido se esconde tras un contador y una tecla. Los hooks de una herramienta se ven
   dentro de ella; los propios siguen en la lista y los externos, en Checklist.
3. Enter hace la acción siguiente del ítem: aprueba y pasa al próximo; arma o acepta un posible
   origen, pidiendo `s` si gasta Claude; en lo sin origen abre "Resolver" (pegar link gratis,
   buscar por 1 corrida, excluir; un posible también se descarta). Nunca desaprueba.
4. `t` arma solo lo que tiene origen; nunca busca orígenes.
5. `x` excluye de un toque; el motivo es opcional.
6. Con POR RESOLVER en cero, nada corriendo y el último guardado sin error, la pantalla dice
   "COOKBOOK LISTO", si quedó subido al repo o solo en esta PC, y qué sigue en la otra PC;
   errores y posibles quedan en POR RESOLVER.
7. Pestañas en el orden real: Armar, Instalar, Checklist.
8. El resultado de cada acción (escanear, armar, guardar) dice qué cambió y queda a la vista
   hasta la próxima; ningún aviso de fondo lo pisa. Cómo salir está siempre a la vista, y
   salir dice si sube el cookbook al repo.

## Usuarios y sistemas afectados
Denis; escaneo, `rebake investigar`, `rebake instalar` y la TUI. Enmienda `vista-clara`,
`cuidar-cuota`, `recetas-directas`, `aprobar-todas` y `alternar`.

## Restricciones
- 80×24, sin Shift ni ventanas nuevas.
- En la TUI, ninguna corrida de Claude sin mostrar cuántas son y pedir `s`.
- Los imports de `CLAUDE.md` (house-rules) siguen siendo recetas.

## Fuera de alcance
- Confirmar en `rebake investigar`: ahí Denis elige qué investiga.
- Cambiar cómo se investiga o instala.

## Preguntas abiertas
Ninguna.

## Cambios
- 2026-10-29: salir a la vista y avisando si sube al repo. Denis: "¿q+s? Otra cosa que no te
  dice".
