# Spec: elegir con qué cuenta corre Claude (desde intent.md 2026-10-23)
Estado: aceptado

## Requisitos
- REQ-1 MUST: aceptar `rebake --profile clp` y `rebake investigar --profile clp`.
  Toda corrida de `claude -p` usa esa cuenta: CLI, TUI (`i`, `t`), buscar origen y aceptar
  uno posible. Origen: Resultado esperado 1, «toda corrida», y Usuarios y sistemas afectados.
- REQ-2 MUST: resolver al arrancar con una sola consulta a la shell de `SHELL`, en modo
  interactivo para leer su archivo de inicio; soportar solo zsh y bash. La cuenta elegida
  prevalece sobre el entorno heredado, incluido el perfil por defecto. Origen: Resultado
  esperado 1, «como si fuera clp -p», y 2, «una vez, al arrancar».
- REQ-3 MUST: aceptar nombres no vacíos con letras ASCII, números, guion o guion bajo,
  sin empezar con guion. Rechazar otros antes de consultar la shell.
  Consultar primero qué es el nombre; invocarlo solo si es alias o función. Nunca ejecutar
  programas del PATH ni nombres desconocidos. Origen: Restricciones, «Solo acepta alias o
  funciones de nombre simple (letras, números, guion o guion bajo, sin empezar con guion):
  nunca ejecuta un programa ni un texto con espacios o ;».
- REQ-4 MUST: ante nombre rechazado, alias o función que no llama a `claude`, shell fallida,
  no soportada o `SHELL` vacía, frenar sin corridas y explicar la causa. Origen: Resultado
  esperado 3, «frena antes de gastar nada y lo dice».
- REQ-5 MUST: sin `--profile`, conservar el comportamiento actual y la cuenta del entorno
  heredado. Origen: Resultado esperado 4, «todo sigue como hoy».
- REQ-6 MUST: en `escanear`, `clonar`, `instalar` y `renombrar-perfil`, aceptar e ignorar
  `--profile`, sin consultar la shell. Mantener escaneo e instalación por perfil; no pedir
  configuración nueva. Origen: Restricciones, «Solo cambia qué cuenta se gasta» y
  «Sin configurar nada nuevo».

## Capacidades y escenarios
### Cuenta elegida (REQ-1, REQ-2)
- GIVEN una shell falsa compatible que cuenta consultas y resuelve `clp` al perfil personal
- WHEN `rebake --profile clp` ejecuta cualquier corrida de la TUI
- THEN usa la cuenta personal y el contador sigue en 1, aunque haya varias corridas.

- GIVEN la misma shell falsa
- WHEN corre `rebake investigar --profile clp`
- THEN todas las corridas usan la cuenta personal y el contador queda en 1.

### Perfil por defecto explícito (REQ-2)
- GIVEN entorno personal y `cl` con perfil por defecto
- WHEN `--profile cl`
- THEN usa la cuenta por defecto, aunque no haya ruta explícita.

### Rechazos (REQ-3, REQ-4)
- GIVEN el nombre `-clp`
- WHEN se pasa `--profile=-clp`
- THEN explica el formato válido, sin consultar la shell ni gastar corridas.

- GIVEN `reboot` es programa del PATH
- WHEN `--profile reboot`
- THEN lo rechaza sin ejecutarlo ni gastar corridas.

- GIVEN `desconocido` no existe
- WHEN `--profile desconocido`
- THEN avisa que no existe, sin ejecutarlo ni gastar corridas.

- GIVEN `otro` es alias que no llama a `claude`
- WHEN `--profile otro`
- THEN avisa la causa, sin corridas.

- GIVEN `SHELL` indica fish
- WHEN `--profile clp`
- THEN avisa shell no soportada, sin corridas.

- GIVEN `SHELL` vacía
- WHEN `--profile clp`
- THEN avisa que falta `SHELL`, sin corridas.

- GIVEN la shell falla
- WHEN resuelve `clp`
- THEN informa el fallo, sin corridas.

### Compatibilidad (REQ-5, REQ-6)
- GIVEN una cuenta heredada
- WHEN sin `--profile`
- THEN conserva esa cuenta y comportamiento.

- GIVEN dos perfiles
- WHEN un subcomando de REQ-6 recibe `--profile clp`
- THEN conserva el escaneo de ambos y la instalación por perfil, sin consultas ni
  configuración nueva.

## Concerns
- Los alias y funciones propios pueden hacer otras cosas además de llamar a `claude`:
  se ejecutan al resolverlos y quedan bajo responsabilidad del usuario.
- La TUI no muestra la cuenta; no se agrega indicador.
- No se mide cuota ni se cambia de cuenta automáticamente.
