# Plan: elegir con qué cuenta corre Claude (desde spec.md 2026-10-23 e intent.md 2026-10-23)
Estado: borrador

## Archivos que cambian
- `src/main.rs`: opción global, resolución inicial, propagación y evento de resolución.
- `src/investigador.rs`: estado de cuenta, resolución y entorno de ambos lanzadores.
- `src/tui/mod.rs`: `Arranque`, bucle y despacho; pruebas internas sin terminal.
- `tests/cli.rs`, `tests/investigador.rs`: ampliar pruebas y fixtures existentes.
- `README.md`: tabla nueva de opciones; ampliar tabla de variables y seguridad.
- `intent/cuenta-claude/intent.md`, `intent/cuenta-claude/spec.md`: enmienda y aprobación.
- `intent/cuenta-claude/plan.md` (nuevo): este plan. Los demás archivos existen.

## Tests a escribir
Reutilizar `HomeFalso::binario` de `tests/comun/mod.rs`: shell falsa llamada zsh/bash,
contador y Claude falso. Entorno por proceso; sin credenciales, red ni home real.

| Test (comportamiento observable) | REQ que cubre |
| --- | --- |
| Opción antes/después de `investigar` y en la entrada TUI; todas las corridas reciben la cuenta elegida. | REQ-1 |
| La shell falsa recibe modo interactivo: contador en 1 tras varias investigaciones, búsquedas y despachos TUI de `i`, `t` y aceptación de origen. | REQ-1, REQ-2 |
| zsh y bash reales con inicio temporal: alias, función y alias encadenado resuelven sin ejecutar Claude real. | REQ-2, REQ-3 |
| Vacío, espacios, punto y coma y guion inicial: error claro y contador de shell en cero. | REQ-3, REQ-4 |
| Programa del PATH con marcador y nombre inexistente: rechazo sin ejecutarlos; marcador ausente. | REQ-3, REQ-4 |
| Alias sin Claude, fish, `SHELL` vacía, fallo, timeout y salida ausente/malformada: error claro, cero corridas. | REQ-4 |
| Cuenta explícita reemplaza la heredada; predeterminada explícita quita la heredada; sin opción se conserva y no consulta shell. | REQ-2, REQ-5 |
| Los cuatro subcomandos sin Claude ignoran la opción incluso con shell incompatible; escaneo de ambos perfiles e instalación por perfil conservados. | REQ-6 |
| Log: un evento por resolución exitosa, rechazada o fallida; campos categóricos, sin alias, ruta, salida de shell ni secretos. | REQ-2, REQ-3, REQ-4 |

## Orden de trabajo
1. Agregar tests de parsing y alcance. Declarar `--profile` global como `--archivo`; resolver
   solamente con opción y entrada TUI o `investigar`, después de iniciar logs y antes de
   abrir la terminal o lanzar corridas. Los otros subcomandos la ignoran (REQ-1, REQ-5, REQ-6).
2. Probar e implementar resolución en `src/investigador.rs`. Validar nombre antes de consultar
   `SHELL`; admitir solo zsh/bash. Una única sonda interactiva carga el inicio, consulta el
   tipo mediante el builtin propio de cada shell y solo invoca alias/función. Interceptar
   `claude` para capturar `CLAUDE_CONFIG_DIR`, sin corrida real. Quitar la variable heredada
   antes de iniciar la shell. Reutilizar `proceso::ejecutar` con tope breve; exigir éxito y
   un registro identificado válido. Salida ausente o malformada nunca significa perfil
   predeterminado; devolver causas claras sin exponer salida cruda (REQ-2–REQ-4).
3. Representar tres estados: heredar, predeterminado explícito y directorio explícito.
   Aplicarlos al `Command` de investigar y buscar origen; conservar binario y argumentos
   actuales, incluido `REBAKE_CLAUDE`. Pasar el estado resuelto por CLI, `Arranque`, bucle y
   ambos despachos, compartido por los hilos. Actualizar todos los callers y fixtures
   afectados; conservar resolución directa y parámetros de concurrencia. Usar entorno por
   hijo, sin mutar el global ni cambiar el instalador (REQ-1, REQ-2, REQ-5, REQ-6).
4. Registrar en la entrada un evento por resolución: acción, shell admitida, resultado y
   causa categórica. Éxito y rechazo esperado en info; fallo operativo en error. Integrar
   fallos con el cierre existente de `main`, sin doble log. Verificar los eventos capturando
   el log real; las funciones de biblioteca devuelven contexto, no loguean.
5. Actualizar README: opción y ejemplos, alcance, `SHELL`, precedencia de `CLAUDE_CONFIG_DIR`,
   relación con `REBAKE_CLAUDE` y efectos propios de alias/funciones. Correr las pruebas.

## Riesgos
- La shell ejecuta su inicio y los efectos adicionales de alias/funciones propios.
- Confundir herencia con predeterminado explícito puede gastar otra cuenta.
- Mutar entorno global competiría con el hilo de actualización y los trabajos paralelos.
- Una respuesta ausente, ambigua o fallida debe frenar, sin elegir otra cuenta.

## Prueba
- `cargo test --locked --lib --test cli --test investigador --test instalador --test tui_arranque`:
  resolución, propagación, log y regresiones de la tabla.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` y `cargo test --locked`:
  formato, lint y suite completa.
- zsh/bash con inicio temporal y Claude falso: comprobar cuenta efectiva y una consulta,
  sin gastar cuota ni tocar perfiles reales.
