# Intención: cada receta con sus piezas
Autor: Denis · Estado: aceptado · Fecha: 2026-11-03

## Problema
En Armar aparece como fila propia lo que instala otra cosa:
- `~/.claude/RTK.md` sale "sin origen", pero lo crea `rtk init -g`, como el hook de rtk.
- El `install.sh` de ai-native-sdlc crea skill, import y hook: tres filas y dos recetas iguales.
- Las seis skills de web-quality, de un solo repo, son seis filas.
- De un plugin no se ve qué hooks o skills trae.

Denis: "la idea es que se instalen recetas, no valen la pena que muestren hooks, skills, cosas que
se van a instalar una vez que se instale una skill, un plugin, una herramienta, una cli".

## Resultado esperado
"Que sea algo jerárquico":
1. Una fila por receta: plugin, herramienta o CLI, o skill. Lo que esa receta instala no tiene
   fila propia.
2. El panel de la derecha lista sus piezas (hooks, skills, imports): "si un plugin tiene hooks o
   skills, que se muestren en el cuadrado".
3. Aprobar o excluir la receta vale para sus piezas; en una PC nueva corre solo su receta: "con
   aprobar una cosa ya aprobas todas".
4. Se reconocen solas las piezas que un plugin trae en su carpeta y las que crea la misma receta
   (ai-native-sdlc); el resto (`RTK.md`) se ata a mano, gratis, en Resolver.
5. Las piezas quedan en el cookbook y se ven en la otra PC: "la idea es poder agrupar y comparar
   lo que uno tiene instalado en una PC con la otra".
6. Lo de un mismo marketplace o repo va en árbol: padre arriba y debajo sus hijos (plugins o
   skills, con su receta). Aprobar o excluir al padre vale para todos; un hijo también
   va suelto: "si aceptás al padre, aceptás todo".

## Usuarios y sistemas afectados
Denis; escaneo, Armar, Instalar y cookbook. Extiende `hooks-de-herramientas`.

## Restricciones
- 80×24; reconocer piezas no gasta Claude.
- Una pieza se ata solo a la receta que de verdad la instala.
- Los hooks e imports propios de Denis siguen siendo recetas.

## Fuera de alcance
- Instalar, aprobar o excluir una pieza suelta dentro de un plugin.
- Lo sincronizado por la cuenta de Claude: ya está oculto.

## Preguntas abiertas
Ninguna.

## Cambios
- 2026-11-03: árbol para marketplaces y repos, para poder excluir una skill suelta como `seo`.
  Denis: "orden en forma de árbol". Aprobado.
