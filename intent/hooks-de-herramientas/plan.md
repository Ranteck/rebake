# Plan: hooks que instala su herramienta (desde spec.md 2026-10-27 e intent.md 2026-10-27)
Estado: borrador

## Archivos que cambian
Existentes, verificados:

- `src/modelo.rs`: responsable opcional validado (ítem o externo), sin nuevo estado ni cambiar versión del cookbook.
- `src/escaner/comandos.rs`, `src/servicio.rs`: reconocimiento posterior a fusión, presencia y filtros compartidos.
- `src/acciones.rs`, `src/investigador.rs`, `src/instalador.rs`: guardas, investigación, registro y comprobación.
- `src/checklist.rs`, `src/tui/app.rs`, `src/tui/vista.rs`: agrupación, acciones, selección y presentación.
- `src/main.rs`, `src/tui/mod.rs`: integración CLI/TUI y eventos.
- `tests/archivo.rs`, `tests/escaner_comandos.rs`, `tests/servicio.rs`, `tests/acciones.rs`, `tests/investigador.rs`, `tests/instalador.rs`, `tests/checklist.rs`, `tests/tui.rs`, `tests/cli.rs`: casos y fixtures afectados.
- `README.md`: reconocimiento, registro, comprobación, pendientes y seguridad.
- `intent/hooks-de-herramientas/spec.md`: aceptación aprobada.

Nuevo: `intent/hooks-de-herramientas/plan.md`. Sin módulos nuevos, dependencias ni cambios a
`.github/workflows/release.yml`, `Cargo.toml` o `Cargo.lock`.

## Tests a escribir
Reutilizar `HomeFalso`, `Entorno`, `PacmanFalso` de `tests/comun/mod.rs`, Claude/instaladores
falsos y `TestBackend`. Configuraciones sintéticas, URLs example.com; HOME y entorno de
perfiles temporales por proceso. Sin leer configuración real, red, credenciales ni Claude real.
Cada test nuevo primero rojo; conservar asserts existentes.

| Test (comportamiento observable) | REQ que cubre |
| --- | --- |
| Ejecutable tras asignaciones; carpeta y nombre coincidentes; condicionales `if` y guardas que ejecutan ruta, con expansiones de home; carpeta externa sin ítem; menciones, intérpretes, prefijos, escapes y ambigüedad no vinculan. | REQ-1 |
| `load-intent.sh` y scripts propios conservan receta; compartir carpeta/repo o `requiere` no atribuye autoría. | REQ-1, REQ-6 |
| TOML viejo y nuevo, ida/vuelta y vínculos inválidos; reescaneo conserva identidad, perfiles y exclusiones, sin receta independiente. | REQ-1, REQ-2 |
| CLI explícita/lotes y TUI: hooks vinculados no buscan origen, investigan, aprueban ni ejecutan, incluso antes aprobados; resultados tardíos no restauran recetas. Marca visible a 80 columnas. | REQ-2 |
| Claude falso recibe hooks/perfiles; receta instala y registra con cita; instalador integrado no duplica; documentación ausente deja pendiente. Receta directa no evita registro. | REQ-3 |
| Comprobación sin aprobación: herramienta presente, hook faltante en un perfil; lectura fallida informa error; instalar registra pese al binario existente; segunda instalación no duplica. | REQ-4 |
| Quince hooks externos producen una línea por herramienta con perfiles y aviso en CLI/TUI, incluso sin recetas instalables; nunca se ejecutan. | REQ-5 |
| Secretos sintéticos ausentes de TOML, pantalla, prompts y logs; ningún comando detectado se ejecuta ni se copia configuración. Eventos de éxito, pendiente y fallo con campos seguros. | REQ-7 |

## Orden de trabajo
1. Añadir responsable opcional a Item, validarlo al leer/guardar y conservar cookbooks existentes.
   Después de fusionar el escaneo, resolver contra todos los ítems. Reutilizar tokenización,
   rutas y ocultación existentes; reconocer solamente ejecución directa y condicional inequívoca,
   sin evaluar shell. Comparar rutas por componentes; ante incertidumbre, no vincular (REQ-1, REQ-6).
2. Conservar ítems/perfiles y exclusiones; retirar recetas propias de hooks vinculados.
   Aplicar prohibiciones en acciones/servicio/investigador/instalador, descartar resultados tardíos y reflejarlo en teclas/lotes,
   incluso con recetas antiguas. Mantener `requiere` separado de autoría (REQ-2).
3. Aportar hooks y perfiles al trabajo de su responsable. Adaptar recetas directas y sus
   callers; cuando necesiten documentación, investigar con Claude. Validar respuesta explícita:
   registro citado después de instalación, creación integrada o pendiente; preservar aprobación,
   citas y alcance por perfil, sin inventar comandos (REQ-3).
4. Separar IDs comprobables de instalables: `c` incluye hooks vinculados sin aprobarlos.
   Comprobar presencia leyendo configuración, nunca ejecutando hooks. Ni selección TUI ni
   precomprobación del instalador saltean al responsable mientras falte un hook (REQ-4).
5. Agrupar externos en checklist compartida y mostrarla también sin recetas instalables;
   actualizar marcas/detalles y README (REQ-2, REQ-5).
6. Ampliar eventos existentes en entradas CLI/TUI para escaneo, investigación, comprobación
   e instalación: resultado, causa categórica y conteos de vínculos, externos y faltantes,
   incluyendo éxitos. Bibliotecas devuelven contexto; errores se reportan una vez según
   house rules. Sin comandos, IDs de hooks, rutas privadas ni secretos. Capturar eventos
   reales en tests (REQ-7; regla 4.4).

## Riesgos
- Atribución errónea, pérdida de exclusiones, perfiles ilegibles y recetas antiguas ejecutables.
- Instaladores que ignoran perfiles; documentar límites.
- El nuevo campo requiere actualizar lectores antiguos.

## Prueba
- `cargo test --locked --test archivo --test escaner_comandos --test servicio --test acciones --test investigador --test instalador --test checklist --test tui --test cli`: tabla completa y regresiones.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked`: formato, lint y suite.
