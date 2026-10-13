# Intención: elegir qué instalar desde la URL del cookbook
Autor: Denis · Estado: aceptado · Fecha: 2026-10-13

## Problema
Con un cookbook ya armado, llegar a instalarlo pide varios pasos sueltos y a ciegas: clonar con
`rebake clonar <url>`, abrir `rebake` aparte, y al instalar van todas las recetas aprobadas sin
saber cuáles ya están en esta PC ni poder dejar alguna afuera por esta vez. Denis lo quiere de
corrido: "poder pasarle la URL sin hacer git clone".

## Resultado esperado
"El flujo que se me ocurre si yo tengo una receta es pasarle la URL de la receta, que hacer un
rebake, que me aparezca el gráfico y poder seleccionar si quiero instalarlas todas, seleccionar
si quiero cuál no quiero instalar y qué es lo que ya tengo instalado."

1. `rebake <url>` clona el cookbook la primera vez y abre la pantalla. Las veces siguientes
   alcanza con `rebake`.
2. La pantalla muestra, por receta, qué ya está instalado en esta PC y qué falta.
3. Se puede instalar todo o desmarcar las recetas que no se quieren instalar.
4. La pantalla marca las recetas que llegaron o cambiaron la última vez que rebake trajo el
   cookbook del repo, para verlas antes de instalar.

Éxito: en una PC nueva, con la URL de un cookbook ya preparado, un comando abre la pantalla;
desde ahí Denis ve qué tiene y qué falta, saca lo que no quiere y instala el resto.

## Usuarios y sistemas afectados
Denis y cualquiera que use rebake con su cookbook en un repo git; los perfiles de Claude Code
de la PC; el repo del cookbook.

## Restricciones
- Del intent raíz: CLI con TUI, solo instaladores, sin secretos, sin sobreingeniería.
- Desmarcar una receta vale solo para esa instalación en esa PC: no se guarda en el cookbook ni
  se sube al repo. Decisión de Denis.
- Se elige por receta, no por perfil: una receta marcada se instala en los perfiles donde
  falta. Decisión de Denis.
- Abrir un cookbook no ejecuta nada suyo: la comprobación de qué está instalado corre cuando
  Denis la pide. Decisión de Denis.
- Si ya hay un clon y se pasa otra URL, rebake avisa y no pisa nada.
- La sincronización de `cookbook-remoto` (traer antes, publicar después) no cambia.

## Fuera de alcance
- Excluir una receta para siempre: ya existe y queda como está.
- Elegir en qué perfil se instala cada receta: eso está en la receta.
- Mirar un cookbook ajeno sin dejar un clon.
- Investigar y aprobar recetas: es el flujo que ya existe, y este parte de un cookbook
  preparado.

## Preguntas abiertas
Ninguna; las decisiones de diseño van al spec.

## Cambios
