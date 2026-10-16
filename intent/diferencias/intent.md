# Intención: ver la diferencia entre el cookbook y esta PC
Autor: Denis · Estado: borrador · Fecha: 2026-10-15

## Problema
Para saber qué del cookbook tiene esta PC hay que juntar piezas sueltas: `c` en Instalación
dice "instalada" o "falta", Recetas marca `(ausente)`, y lo instalado que el cookbook no
tiene solo aparece al escanear, mezclado con el resto. Denis: "check all the recipes that I
have installed on my PC. If I already have a cookbook, the other tab should show me the
difference between the cookbook and what I have installed."

## Resultado esperado
Con un cookbook ya armado, una vista muestra cada ítem en uno de tres grupos:
1. En el cookbook e instalado en esta PC.
2. En el cookbook pero falta en esta PC (o en alguno de sus perfiles).
3. Instalado en esta PC pero todavía no está en el cookbook.

Éxito: Denis abre `rebake` en una PC y de un vistazo sabe qué le falta instalar y qué le
falta agregar al cookbook.

## Usuarios y sistemas afectados
Quien use rebake con un cookbook existente; los dos perfiles de Claude Code de la PC.

## Restricciones
- Las del intent raíz y las de `elegir-que-instalar`: abrir un cookbook no ejecuta nada
  suyo; la comprobación corre cuando Denis la pide.
- Reusa la detección que ya existe (escaneo y comprobación); no inventa otra.

## Fuera de alcance
- Instalar o agregar al cookbook automáticamente desde esta vista.
- Comparar contra cookbooks ajenos.

## Preguntas abiertas
- ¿Reemplaza a la pestaña Instalación o es una vista nueva?
- ¿El grupo 3 se calcula escaneando al abrir o solo cuando Denis aprieta `s`?

## Cambios
