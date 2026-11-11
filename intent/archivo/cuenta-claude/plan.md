# Plan: elegir con qué cuenta corre Claude (desde spec.md 2026-10-23 e intent.md 2026-10-23)
Estado: aceptado

## Archivos que cambian
- `src/main.rs`: opción global, resolución y fijación inicial; evento de resolución.
- `src/investigador.rs`: resolución, `OnceLock`, aplicación del entorno y tests puros.
- `tests/cli.rs`, `tests/investigador.rs`: tests de proceso.
- `README.md`: tabla nueva de opciones; ampliar tabla de variables y seguridad.
- `.github/workflows/release.yml`: instalar zsh junto a musl-tools antes de los tests.
- `intent/cuenta-claude/plan.md`: este plan. Todos los archivos existen.

## Tests a escribir
Reutilizar `HomeFalso::binario` de `tests/comun/mod.rs`: shells falsas contadoras y Claude
falso. Casos con estado fijado, en procesos separados; zsh/bash reales usan `HOME` y
`ZDOTDIR` temporales. Sin credenciales, red ni configuración real.

| Test (comportamiento observable) | REQ que cubre |
| --- | --- |
| Opción antes/después de `investigar` y al arrancar TUI; los dos lanzadores reciben la cuenta elegida. | REQ-1 |
| Por proceso, una fijación; la shell falsa recibe modo interactivo y su contador queda en 1 tras varias investigaciones y búsquedas, también en paralelo. | REQ-1, REQ-2 |
| zsh y bash reales con inicio temporal: alias, función y alias encadenado resuelven sin ejecutar Claude real. | REQ-2, REQ-3 |
| Vacío, espacios, punto y coma y guion inicial: error claro y contador de shell en cero. | REQ-3, REQ-4 |
| Programa del PATH con marcador y nombre inexistente: rechazo sin ejecutarlos; marcador ausente. | REQ-3, REQ-4 |
| Alias sin Claude, fish, `SHELL` vacía, fallo, timeout y salida ausente/malformada: error claro, cero corridas. | REQ-4 |
| Función pura sobre `Command`: heredar no altera entorno, predeterminado quita la variable y directorio la fija; procesos comprueban precedencia. Sin opción, herencia y cero consultas. | REQ-2, REQ-5 |
| Los cuatro subcomandos sin Claude ignoran la opción incluso con shell incompatible; escaneo de ambos perfiles e instalación por perfil conservados. | REQ-6 |
| Log: un evento por resolución exitosa, rechazada o fallida; campos categóricos, sin alias, ruta, salida de shell ni secretos. | REQ-2, REQ-3, REQ-4 |

## Orden de trabajo
1. Probar `--profile` global como `--archivo`. Resolver solo con opción y entrada TUI o
   `investigar`, después de iniciar logs y antes de abrir terminal o lanzar corridas.
   Los demás subcomandos la ignoran (REQ-1, REQ-5, REQ-6).
2. Probar e implementar resolución en `src/investigador.rs`. Validar nombre antes de consultar
   `SHELL`; admitir solo zsh/bash. Una única sonda interactiva carga el inicio, consulta el
   tipo mediante el builtin propio de cada shell y solo invoca alias/función. Interceptar
   `claude` para capturar `CLAUDE_CONFIG_DIR`, sin corrida real. Quitar la variable heredada
   antes de iniciar la shell. Reutilizar `proceso::ejecutar` con tope breve; exigir éxito y
   un registro identificado válido. Salida ausente o malformada nunca significa perfil
   predeterminado; devolver causas claras sin exponer salida cruda (REQ-2–REQ-4).
3. Fijar el estado resuelto una sola vez en un `OnceLock` de `investigador`, antes de
   cualquier corrida. Distinguir heredar, predeterminado explícito y directorio explícito.
   Los dos lanzadores leen ese estado y aplican el entorno mediante una función pura:
   recibe estado y configura el `Command`, sin leer globales. Sin fijación, heredar;
   rechazar una segunda fijación. Conservar firmas públicas, callers, `REBAKE_CLAUDE`,
   argumentos, resolución directa y concurrencia: la TUI usa los mismos lanzadores.
   Probar aplicación pura sin `OnceLock` y propagación en procesos separados; no resetear
   el estado ni mutar entorno global. La instalación conserva su manejo por perfil
   (REQ-1, REQ-2, REQ-5, REQ-6).
4. Registrar en la entrada un evento por resolución: acción, shell admitida, resultado y
   causa categórica. Éxito y rechazo esperado en info; fallo operativo en error. Integrar
   fallos con el cierre existente de `main`, sin doble log. Verificar los eventos capturando
   el log real; las funciones de biblioteca devuelven contexto, no loguean.
5. En Toolchain de `.github/workflows/release.yml`, instalar zsh junto a musl-tools antes
   de `cargo test`. Ambas shells son obligatorias; sin condiciones ni skips (REQ-2).
6. Actualizar README: opción y ejemplos, alcance, `SHELL`, precedencia de `CLAUDE_CONFIG_DIR`,
   relación con `REBAKE_CLAUDE` y efectos propios de alias/funciones. Correr las pruebas.

## Riesgos
- La shell ejecuta su inicio y los efectos adicionales de alias/funciones propios.
- Confundir herencia con predeterminado explícito puede gastar otra cuenta.
- Fijar tarde el estado permitiría corridas con otra cuenta; inicializar antes del trabajo.
- Una respuesta ausente, ambigua o fallida debe frenar, sin elegir otra cuenta.

## Prueba
- `cargo test --locked --lib --test cli --test investigador --test instalador --test tui_arranque`:
  resolución, propagación, log y regresiones de la tabla.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` y `cargo test --locked`:
  formato, lint y suite completa.
