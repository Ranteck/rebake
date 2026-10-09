# Intención: cookbook desde un repo
Autor: Denis · Estado: aceptado · Fecha: 2026-10-08

## Problema
El cookbook de Denis ya vive aparte, como pide el intent raíz: "ahora el archivo está en un
repo, privado, y ese repo tiene un solo archivo" (`Ranteck/rebake-cookbok`). Pero rebake solo
sabe leer un archivo local, así que hay que clonar el repo y enlazarlo a mano, y después
acordarse de commitear y pushear lo que rebake cambió.

## Resultado esperado
"Me gustaría poder pasarle la url y que automáticamente lo analice rebake":
1. Le paso la URL una vez y rebake clona el repo en la carpeta del cookbook; desde ahí
   `rebake` a secas lo usa como cookbook para todo (TUI, escanear, investigar, instalar).
2. Al arrancar trae lo último del repo; al terminar, si el cookbook cambió, hace un solo
   commit con un resumen y lo pushea.
3. En una PC nueva: instalar rebake, pasarle la URL e instalar.

Éxito: con el repo privado y la URL, rebake trabaja sobre el cookbook del repo y cada cambio
termina en el repo sin tocar git a mano.

## Usuarios y sistemas afectados
Denis y cualquiera que use rebake con su cookbook en un repo git propio; git y las
credenciales que el usuario ya tiene configuradas (en Denis, `gh auth git-credential`).

## Restricciones
- Del intent raíz: herramienta pública, cookbook aparte, sin secretos, sin sobreingeniería.
- rebake no maneja credenciales: usa el `git` del usuario y su autenticación.
- Solo sincroniza un clon hecho por rebake: nunca pushea a un repo que no se lo pidió
  (dotfiles, symlinks).
- Un fallo de red o de git no hace perder trabajo: lo guardado queda en disco y se avisa.

## Fuera de alcance
- Diagnóstico de qué recetas faltan instalar en esta PC (otra feature).
- Resolver conflictos de merge: si el repo divergió, rebake avisa y lo resolvés vos.
- Proveedores con API propia o descarga sin git.

## Preguntas abiertas
Ninguna; las decisiones de diseño van al spec.

## Cambios
