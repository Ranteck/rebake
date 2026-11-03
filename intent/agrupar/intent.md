# Intención: cada receta con sus piezas
Autor: Denis · Estado: aceptado · Fecha: 2026-11-03

## Problema
En Armar aparece como fila propia lo que instala otra cosa:
- `RTK.md` sale "sin origen", pero lo crea `rtk init -g`, como el hook de rtk.
- El `install.sh` de ai-native-sdlc crea skill, import y hook: tres filas, dos recetas iguales.
- Las seis skills de web-quality, de un repo, son seis filas.
- De un plugin no se ve qué trae.

Denis: "la idea es que se instalen recetas, no valen la pena que muestren hooks, skills, cosas que
se van a instalar" con otra.

## Resultado esperado
"Que sea algo jerárquico":
1. Una fila por receta (plugin, herramienta o CLI, skill); lo que instala no tiene fila propia.
2. El panel derecho lista sus piezas: "si un plugin tiene hooks o
   skills, que se muestren en el cuadrado".
3. Aprobar o excluir la receta vale para sus piezas; en una PC nueva corre solo su receta: "con
   aprobar una cosa ya aprobas todas".
4. Se reconocen solas las piezas de la carpeta de un plugin y las que crea la misma receta; el
   resto (`RTK.md`) se ata a mano en Resolver.
5. Las piezas quedan en el cookbook y se ven en la otra PC: "agrupar y comparar lo que uno tiene
   instalado en una PC con la otra".
6. Lo de un mismo marketplace o repo va en árbol: el padre y debajo sus hijos, cada uno con su
   receta. Aprobar o excluir al padre vale para todos; un hijo también va suelto: "si aceptás al
   padre, aceptás todo".
7. Al confirmar el origen de una skill suelta, rebake busca en ese repo las otras sin origen con
   el mismo nombre y les ofrece ese origen de una vez, sin Claude.

## Usuarios y sistemas afectados
Denis; escaneo, Armar, Instalar, cookbook. Extiende `hooks-de-herramientas`.

## Restricciones
- 80×24; reconocer piezas y hermanas no gasta Claude.
- Una pieza se ata solo a la receta que de verdad la instala.
- Los hooks e imports propios de Denis siguen siendo recetas.

## Fuera de alcance
- Instalar, aprobar o excluir una pieza suelta dentro de un plugin.
- Lo sincronizado por la cuenta de Claude.

## Preguntas abiertas
Ninguna.

## Cambios
- 2026-11-03: árbol para marketplaces y repos, para excluir una skill suelta como `seo`.
  Denis: "orden en forma de árbol". Aprobado.
- 2026-11-03: las hermanas sin origen toman el de la primera confirmada. Denis: "si no sabe,
  podrían buscar…". Aprobado.
