# Intención: hooks que instala su herramienta
Autor: Denis · Estado: aceptado · Fecha: 2026-10-27

## Problema
rebake registra cada hook de los perfiles como un ítem con receta propia, pero muchos los crea una
herramienta al instalarse o al correr. En esta PC: 15 de Orca, el de `rtk` y 3 de `impeccable`.
Denis: "muchos hooks se instalan después de que uno instala la herramienta. Así que no habría que
registrarla ahí. Sino que lo mejor sería que cuando reinstalo en otra máquina, corra el comando y
que se instale solo".

## Resultado esperado
1. Un hook que pone una herramienta no tiene receta propia: queda ligado a esa herramienta y la
   pantalla dice "lo instala <herramienta>". "Cuando instale Orca o RTK, automáticamente se
   vinculan los hooks. Entonces no vale la pena que se reinstalen dos veces."
2. Si rebake instala la herramienta, su receta incluye el comando que crea el hook (p. ej.
   `rtk init -g`): al reinstalar en otra PC el hook aparece solo.
3. Si la herramienta viene de afuera de rebake (como Orca), el hook no se instala y la checklist
   dice que aparece al instalar o abrir esa herramienta.
4. Los hooks propios de Denis (sus scripts, como `sync-plugins.sh`) siguen como hoy.

## Usuarios y sistemas afectados
Denis y sus dos perfiles; el escaneo de hooks, las recetas de herramientas y la checklist.

## Restricciones
- Solo instaladores: el comando sale del repo de la herramienta, no se inventa.
- Sin secretos: los hooks pueden traer tokens en el comando.
- Nunca se copia el hook a mano en `settings.json`.

## Fuera de alcance
- Instalar herramientas de escritorio o de pacman (como Orca).

## Preguntas abiertas
- ¿Cómo reconoce rebake de qué herramienta es un hook? Candidato: por el binario o la ruta del
  comando (`rtk`, `~/.orca/`, `skills/impeccable`). Lo resuelve el spec.

## Cambios
