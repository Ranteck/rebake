# Intención: qué hace cada ítem
Autor: Denis · Estado: aceptado · Fecha: 2026-10-30

## Problema
En Armar aparecen cosas que Denis no reconoce y no sabe si dejar o excluir: "tampoco sé qué es
mutation, trailofbits". Para decidir tiene que salir a buscar qué hace cada una.

## Resultado esperado
"Estaría bueno que en la derecha en el cuadrado aparezca una descripción, así como hace Claude
Code cuando uno instala un plugin, una descripción de lo que hace."
1. El panel de detalle muestra qué hace el ítem elegido, con la descripción que declara su autor:
   la del plugin, la del marketplace o la del `SKILL.md`.
2. Sale gratis de los archivos que esta PC ya tiene al escanear: sin Claude ni red.
3. Se guarda en el cookbook: en la otra PC también se ve, en Instalar, antes de instalar.
4. Se muestra tal cual, en el idioma del autor (casi siempre inglés), sin traducir.
5. Si el ítem no declara descripción (servidores MCP, herramientas, hooks), el panel lo dice y no
   inventa una.

## Usuarios y sistemas afectados
Denis; escaneo y panel de detalle de la TUI.

## Restricciones
- 80×24, sin ventanas nuevas.
- La descripción es la del autor, como la receta sale del repo de origen: no se inventa.

## Fuera de alcance
- Escribir descripciones para lo que no declara una.

## Preguntas abiertas
Ninguna.

## Cambios
