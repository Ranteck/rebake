# Intención: aprobar todas y destildar
Autor: Denis · Estado: aceptado · Fecha: 2026-10-27

## Problema
Después de las tandas quedan muchas recetas por revisar (60 de 66) y hoy se aprueban de a una con
`a`: "¿cómo hago para aprobar todas?". Tampoco se ve cuándo termina una tanda ni qué sigue:
"¿cómo me entero que terminó o qué tengo que hacer?". Y la marca ○ de "por revisar" se lee como
"investigando": Denis pensó que esas 60 seguían en curso.

## Resultado esperado
1. Una tecla aprueba todas las por revisar: pregunta "¿Aprobar N recetas?" y `s` confirma, como
   las tandas.
2. Después se destilda la que no se quiere: espacio sobre una aprobada la vuelve a por revisar.
   "La idea es que uno pueda aprobar todo pero luego destildar la que no quiero. Entonces la
   estrategia es al revés esta vez."
3. Al terminar una tanda, el pie avisa qué dejó (por revisar, con error) y qué sigue: aprobar o
   lanzar otra con `t`.
4. "Por revisar" usa otra marca, que se lea como "lista para tu OK" y no como "en curso"
   ("cambiá el ○").

## Usuarios y sistemas afectados
Denis; la pestaña Recetas de la TUI (teclas, pie y lista). Cambia la restricción de
`cuidar-cuota` "Aprobar sigue siendo de a una receta".

## Restricciones
- Pantalla simple: entra en 80×24, sin teclas con Shift y sin ventanas nuevas.
- Aprobar de a una sigue igual; las que no tienen receta no se aprueban.
- La pestaña Instalación sigue mostrando qué va a correr antes de instalar.

## Fuera de alcance
- Instalar al aprobar.
- Cambiar cómo se arman o investigan las recetas.

## Preguntas abiertas
Ninguna.

## Cambios
