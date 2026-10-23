# Spec: elegir con qué cuenta corre Claude (desde intent.md 2026-10-23)
Estado: borrador

## Requisitos
- REQ-1 MUST: aceptar `--profile <nombre>` tanto en `rebake --profile clp` como en
  `rebake investigar --profile clp`. Toda corrida de `claude -p` usa la cuenta elegida,
  en CLI y TUI: `i`, `t`, buscar origen y aceptar un posible origen. Origen: Resultado
  esperado 1, «toda corrida», y Usuarios y sistemas afectados.
- REQ-2 MUST: resolver el alias o función en la shell del usuario una sola vez, al arrancar.
  La cuenta elegida prevalece sobre el entorno heredado, también cuando el alias usa el
  perfil por defecto. Origen: Resultado esperado 1, «como si fuera clp -p», y 2,
  «le pregunta a la shell una vez, al arrancar».
- REQ-3 MUST: aceptar solo nombres no vacíos, compuestos por letras ASCII, números, guion o
  guion bajo. Rechazar el resto antes de consultar la shell, con un mensaje claro y sin
  gastar corridas. Origen: Resultado esperado 2, «alias o función», y 3,
  «frena antes de gastar nada y lo dice»; restricción del pedido de este spec.
- REQ-4 MUST: si el nombre no existe, no llama a `claude` o la shell falla o es incompatible,
  frenar al arrancar, antes de gastar corridas, con un mensaje que explique el problema.
  Origen: Resultado esperado 2, «le pregunta a la shell», y 3, «frena antes de gastar
  nada y lo dice».
- REQ-5 MUST: sin `--profile`, conservar el comportamiento actual, incluida la cuenta que
  resulte del entorno heredado. Origen: Resultado esperado 4, «todo sigue como hoy».
- REQ-6 MUST: mantener el escaneo de ambos perfiles y la instalación con la cuenta de cada
  perfil, sin que `--profile` la cambie. No exigir configuración nueva en la shell ni en
  el cookbook. Origen: Restricciones, «Solo cambia qué cuenta se gasta» y
  «Sin configurar nada nuevo».

## Capacidades y escenarios
### Cuenta elegida en CLI y TUI (REQ-1, REQ-2)
- GIVEN `clp` llama a `claude` con la cuenta personal
- WHEN Denis abre `rebake --profile clp` y usa `i`, `t`, busca origen o acepta uno posible
- THEN cada corrida usa la cuenta personal, sin volver a consultar la shell.

- GIVEN el mismo `clp`
- WHEN corre `rebake investigar --profile clp`
- THEN la shell se consulta una vez al arrancar y todas las corridas usan esa cuenta.

### Perfil por defecto explícito (REQ-2)
- GIVEN el entorno apunta al perfil personal y `cl` usa el perfil por defecto
- WHEN arranca con `--profile cl`
- THEN las corridas usan la cuenta por defecto, aunque el alias no indique una ruta.

### Rechazos (REQ-3, REQ-4)
- GIVEN un nombre vacío o con espacios, punto y coma u otros caracteres no permitidos
- WHEN arranca con ese `--profile`
- THEN lo rechaza antes de consultar la shell y explica el formato válido, sin corridas.

- GIVEN un nombre válido que no existe o no llama a `claude`, o una shell que falla
  o es incompatible, como fish
- WHEN intenta resolverlo al arrancar
- THEN frena con un mensaje claro sobre la causa y no gasta ninguna corrida.

### Compatibilidad (REQ-5, REQ-6)
- GIVEN un entorno con una cuenta de Claude seleccionada
- WHEN arranca sin `--profile`
- THEN usa la misma cuenta y conserva el comportamiento actual.

- GIVEN `--profile clp` y los dos perfiles instalados
- WHEN escanea o instala
- THEN escanea ambos e instala cada uno con su cuenta, sin pedir configuración nueva.

## Concerns
- Los alias y funciones dependen de la shell y su configuración; fish debe fallar claramente.
- La TUI hoy no muestra la cuenta elegida. Este alcance no agrega ese indicador.
- Si la cuenta se queda sin cuota, no se cambia sola ni se mide la cuota.
