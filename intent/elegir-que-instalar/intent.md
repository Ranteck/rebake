# Intención: elegir qué instalar desde la URL del cookbook
Autor: Denis · Estado: aceptado · Fecha: 2026-10-13

## Problema
Instalar un cookbook ya armado pide pasos sueltos y a ciegas: clonar, abrir `rebake`, y van
todas las aprobadas sin saber cuáles ya están ni poder dejar alguna afuera. Denis quiere
"poder pasarle la URL sin hacer git clone".

## Resultado esperado
"El flujo que se me ocurre si yo tengo una receta es pasarle la URL de la receta, que hacer un
rebake, que me aparezca el gráfico y poder seleccionar si quiero instalarlas todas, seleccionar
si quiero cuál no quiero instalar y qué es lo que ya tengo instalado." Y que sea "medio similar
al flujo de instalar skills o plugins de claude code".

1. `rebake <url>` clona la primera vez y abre la pantalla; después alcanza con `rebake`.
2. La pantalla muestra, por receta, qué ya está instalado en esta PC y qué falta.
3. Se puede instalar todo o desmarcar las recetas que no se quieren.
4. La pantalla marca las recetas que llegaron o cambiaron en la última traída del repo.
5. `rebake instalar` lista esas recetas, con las mismas marcas, al pedir confirmación.

Éxito: en una PC nueva, con la URL de un cookbook preparado, un comando abre la pantalla;
Denis ve qué falta, saca lo que no quiere e instala el resto.

## Usuarios y sistemas afectados
Quien use rebake con su cookbook en un repo git; los perfiles de Claude Code de la PC.

## Restricciones
- Las del intent raíz.
- Desmarcar vale solo para esa instalación en esa PC: no se guarda ni se sube.
- Se elige por receta, no por perfil: una receta marcada se instala donde falta.
- Abrir un cookbook no ejecuta nada suyo: la comprobación corre cuando Denis la pide.
- Con un clon de otra URL, avisa y no pisa nada.
- La sincronización de `cookbook-remoto` no cambia.

## Fuera de alcance
- Excluir una receta para siempre: ya existe.
- Elegir en qué perfil va cada receta: está en la receta.
- Mirar un cookbook ajeno sin dejar un clon.
- Investigar y aprobar recetas: se parte de un cookbook preparado.

## Preguntas abiertas
Ninguna.

## Cambios
- 2026-10-13: se suman la frase sobre los plugins de Claude Code y el punto 5; el diseño los
  usaba y no figuraban. Aprobado por Denis.
