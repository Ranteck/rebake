# Spec: qué hace cada ítem (desde intent.md 2026-10-30)
Estado: aceptado

## Requisitos
- REQ-1 MUST: en Armar e Instalar, el panel de detalle muestra la descripción del ítem elegido,
  en cualquier estado (también excluido), debajo de su primera línea. Origen: Resultado 1 ("así
  como hace Claude Code") y 3 ("en Instalar, antes de instalar").
- REQ-2 MUST: es la que declara el autor: skill, el `description` de su `SKILL.md`; plugin, su
  manifiesto o, si no la trae, su entrada del marketplace; marketplace, su manifiesto. Entera
  aunque ocupe varias líneas del archivo. Origen: Resultado 1 ("la del plugin, la del
  marketplace o la del `SKILL.md`").
- REQ-3 MUST: sale del escaneo, de archivos de esta PC, sin Claude ni red: no suma corridas ni
  pedidos de `s`. Origen: Resultado 2.
- REQ-4 MUST: se guarda en el cookbook. Cada escaneo la actualiza en los ítems que encuentra, y
  la borra si el autor la quitó; un ítem ausente en esta PC conserva la guardada. Origen:
  Resultado 3 ("en la otra PC también se ve").
- REQ-5 MUST: se muestra con el texto del autor, sin traducir, resumir ni reescribir. Origen:
  Resultado 4; Restricciones ("no se inventa").
- REQ-6 MUST: sin descripción guardada, el panel dice «sin descripción» y no pone otro texto
  (pista, nombre, Claude). Origen: Resultado 5; Fuera de alcance.
- REQ-7 MUST: antes de guardarse pasa por el filtro de secretos y de rutas del home del resto
  del escaneo. Origen: intent raíz, Restricciones ("Sin secretos").
- REQ-8 SHOULD: ocupa hasta 4 líneas del panel; si no entra, la cuarta termina en «…». Origen:
  Restricciones ("80×24, sin ventanas nuevas").

## Capacidades y escenarios
### Ver qué hace (REQ-1, REQ-2, REQ-5)
- GIVEN una skill con `description` WHEN escaneo y la elijo en Armar THEN el panel muestra ese
  texto sin traducir, bajo la primera línea.
- GIVEN un plugin sin descripción en su manifiesto y con una en su entrada del marketplace WHEN
  escaneo THEN el panel muestra la del marketplace.
- GIVEN una `description` de varias líneas WHEN escaneo THEN el panel la muestra como un párrafo.
- GIVEN un ítem excluido con descripción WHEN lo elijo con las excluidas visibles THEN la muestra.

### Gratis (REQ-3)
- GIVEN skills y plugins instalados WHEN escaneo THEN no corre Claude, no se pide `s` ni se usa
  la red.

### En la otra PC (REQ-4)
- GIVEN un cookbook con la descripción de una skill que esta PC no tiene WHEN la elijo en
  Instalar THEN el panel la muestra; WHEN escaneo THEN sigue igual.
- GIVEN un autor que cambió o quitó su descripción WHEN escaneo una PC que tiene el ítem THEN el
  cookbook guarda la nueva, o ninguna.

### Sin descripción (REQ-6)
- GIVEN un servidor MCP, una herramienta, un hook o un `SKILL.md` sin `description` WHEN lo elijo
  THEN el panel dice «sin descripción», no el nombre ni la pista.

### Sin secretos (REQ-7)
- GIVEN una descripción con un token y una ruta `/home/<usuario>/…` WHEN escaneo THEN el cookbook
  guarda el token oculto y la ruta como `~/…`.

### En 80×24 (REQ-8)
- GIVEN una descripción de 1000 caracteres WHEN la elijo en 80×24 THEN ocupa 4 líneas y la
  última termina en «…».

## Concerns
- Largo: acá las descripciones de skills tienen mediana de 336 caracteres (≈8 líneas del panel
  en 80×24) y máximo de 1015. Enteras taparían la receta; con REQ-8 se pierde el final, casi
  siempre el "Use when…". Alternativa: enteras, con la receta fuera de la vista.
- Compatibilidad: el cookbook rechaza campos desconocidos; un rebake anterior no lo abre después
  de guardar descripciones. Hay que actualizar rebake en cada PC.
- Hasta el primer escaneo con esta versión todo dice «sin descripción»: el texto no afirma que
  el autor no la declare.
- Privacidad: van al cookbook las descripciones de tus skills propias (p. ej. `daily-flock`).
