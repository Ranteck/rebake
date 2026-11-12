# Intención: rebake se actualiza solo
Autor: Denis · Estado: aceptado · Fecha: 2026-10-14

## Problema
Cada arreglo de rebake sale como versión nueva, pero la PC sigue con la que tenía hasta que
Denis corre el `install.sh` a mano. Puede estar usando una versión atrasada sin saberlo, con
bugs que ya están arreglados.

## Resultado esperado
1. Al abrir, rebake mira qué versión está publicada: "que revise qué versión es la que está
   publicada antes". Lo hace "por debajo mientras uno está en el programa, como ya hace Claude
   Code": la pantalla abre sin esperar.
2. Si la suya está atrasada, la pantalla lo avisa con las dos versiones a la vista y una tecla
   para actualizar cuando Denis quiera.
3. Si la aprieta, se actualiza y sigue con la versión nueva; si no, sigue con la que tiene.
4. Si está al día, no dice nada.
5. Los subcomandos (`rebake instalar --si` en una PC recién formateada) tampoco esperan: si la
   revisión ya respondió al terminar, avisan en una línea que hay versión nueva.

## Usuarios y sistemas afectados
Denis en cada PC donde use rebake; GitHub Releases del repo público.

## Restricciones
- Revisa cada vez que abre, sin demorar el arranque.
- Sin red o si GitHub falla, arranca igual: la revisión nunca frena el uso.
- La versión nueva se verifica con su checksum, como hace el `install.sh`; si no coincide, no
  se reemplaza nada.
- Nunca deja un binario a medio reemplazar.
- AI-Native SDLC, sin sobreingeniería.

## Fuera de alcance
- Actualizar sin preguntar.
- Bajar de versión o elegir una versión puntual (eso ya lo hace `REBAKE_VERSION` en el
  `install.sh`).
- Plataformas que no sean Linux x86_64.

## Preguntas abiertas
Ninguna.

## Cambios
- 2026-10-14: la revisión pasa a segundo plano y se actualiza con una tecla, en vez de preguntar
  antes de abrir: "esa comprobación tendría que hacerse por debajo mientras uno está en el
  programa". Aprobado.
