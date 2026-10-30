# Plan: rediseño de Recetas
Estado: borrador
Intent/spec aceptados: 2026-10-29; base v0.12.1.

## Archivos que cambian
Existen: `src/modelo.rs`, `src/escaner/mod.rs`, `src/escaner/archivos.rs`, `src/escaner/plugins.rs`, `src/fusion.rs`, `src/servicio.rs`, `src/instalador.rs`, `src/checklist.rs`, `src/acciones.rs`, `src/investigador.rs`, `src/main.rs`, `src/sincro.rs`, `src/tui/app.rs`, `src/tui/vista.rs`, `src/tui/mod.rs`, `tests/comun/mod.rs`, `tests/checklist.rs`, `README.md`, `Cargo.toml`, `Cargo.lock` y los tests de la tabla. Nuevos: ninguno.

## Tests a escribir
Ampliar suites.

| Comportamiento | Ubicación | REQ |
| --- | --- | --- |
| Destinos reales skills/plugins; dos homónimas, una propia elegible | `tests/escaner_archivos.rs`, `tests/escaner_plugins.rs` | REQ-1 |
| Marca persiste: guardar/cargar, renombrar/perder perfiles, otra PC sin archivos; cookbook antiguo válido | `tests/archivo.rs`, `tests/fusion.rs`, `tests/servicio.rs`, `tests/acciones.rs` | REQ-1 |
| Lotes/destinos omiten sincronizados; rechazar solicitud explícita antes de ejecutar | `tests/instalador.rs`, `tests/cli.rs` | REQ-1, REQ-10 |
| Checklist: sincronizado ausente; receta mixta lista solo destinos propios | `tests/checklist.rs` | REQ-1 |
| Git hijo ignora configuración inválida y no hereda variables inyectadas | `tests/escaner_archivos.rs`, `tests/comun/mod.rs` | REQ-1 |
| x sin motivo, o muestra/oculta, x reincluye conservando pasos; selección/filtro vacío válidos | `tests/acciones.rs`, `tests/tui.rs` | REQ-5, REQ-9 |
| Grupos/imports/hooks propios/vinculados/externos; vinculados/externos no bloquean LISTO | `tests/tui.rs` | REQ-2, REQ-6 |
| Directa nueva: una por revisar, cero Claude/s; existentes intactas | `tests/servicio.rs`, `tests/investigador.rs` | REQ-2, REQ-4 |
| Enter aprueba una de dos, selecciona siguiente; cero instalación | `tests/tui.rs` | REQ-3 |
| Enter arma: anuncia una corrida; cero antes de s, una después | `tests/tui.rs` | REQ-3, REQ-4 |
| Aceptar posible cuesta una tras s; descartarlo cuesta cero | `tests/tui.rs` | REQ-3, REQ-4 |
| Pegar link guarda origen; cero corridas/aprobaciones | `tests/tui.rs` | REQ-3 |
| Buscar exige s: una búsqueda, cero armados | `tests/tui.rs` | REQ-3, REQ-4 |
| Cancelar: cero corridas/recetas nuevas | `tests/tui.rs` | REQ-4 |
| t+s: cinco de seis Claude, queda uno; posible intacto, sin encadenar | `tests/tui.rs` | REQ-4 |
| i/r pagas, con/sin origen: cero antes de s, una después | `tests/tui.rs` | REQ-4 |
| Enter en LISTAS indica «2 Instalar»: estado intacto, cero corridas/instalaciones | `tests/tui.rs` | REQ-3 |
| Enter excluido indica «x reincluir»: estado intacto, cero corridas/instalaciones | `tests/tui.rs` | REQ-3 |
| Resolver, Esc/foco, paste con saltos/q, resultados tardíos; id/scroll estables | `tests/tui.rs`, `src/tui/mod.rs` | REQ-9 |
| Cada error/posible/corrida/guardado fallido bloquea LISTO, incluso filtrando | `tests/tui.rs`, `src/tui/mod.rs` | REQ-6 |
| Pie: avisos no pisan última acción; detalle: error permanece hasta resolverse | `tests/tui.rs`, `src/tui/mod.rs` | REQ-8 |
| 80×24, cifras triples, Unicode/NO_COLOR, pestañas/salida en todos los modos | `tests/tui.rs` | REQ-7 |
| Publicación/sin repo/fallo Git: estado, stdout/stderr, fallo parcial versus aviso recuperado | `tests/sincro.rs`, `tests/cli.rs` | REQ-6, REQ-7, REQ-10 |
| capturar_logs: éxito/fallo, contadores, sin secretos/rutas privadas | `src/tui/mod.rs` | REQ-1, REQ-4, REQ-5, REQ-8 |

## Orden de trabajo
Cada fase exige tests rojos, suite verde, README y commit; entrega una TUI usable. Publicar requiere aprobación y actualizar Cargo.toml/Cargo.lock.

1. **Quitar ruido** (REQ-1, REQ-5, REQ-9, REQ-10). Persistir la clasificación por instalación/perfil, compatible con cookbooks antiguos. Compartir el filtro en trabajos, sin_origen, por_armar, instalación, visibles y Checklist/a_mano/lineas_de, incluidos destinos mixtos. Conservar marcas ausentes y referencias al renombrar. Denis deja de ver sincronizados y recupera excluidos con o/x sin perder recetas.
2. **Ordenar y preparar** (REQ-2, REQ-6, REQ-7). Reusar receta_directa/aplicar al escanear. Denis ve POR RESOLVER/LISTAS, directas listas para revisar, hooks agrupados y externos en Checklist. Reordenar pestañas/ayuda.
3. **Guiar acciones** (REQ-3, REQ-4, REQ-9). Ampliar Modo/Efecto y reutilizar confirmaciones para Enter/Resolver e i/r, que se conservan con s si gastan. Implementar las orientaciones de la tabla. Un lector Crossterm maneja paste y restaura foco. Denis conoce el costo antes de confirmar; la CLI no agrega confirmaciones.
4. **Cerrar claramente** (REQ-6–REQ-8, REQ-10). Guardar informa éxito a App; separar resultados/avisos y conservar errores. Reusar Rama/publicar para arranque/salida. Denis ve qué cambió, qué falta, dónde quedó guardado y cómo salir.

## Riesgos
Una carpeta ausente no borra marcas; perfiles vacíos no habilitan ejecución global. Conservar fixes v0.12.1. Los tests antiguos de i/r, link vacío y cuota incorporan s porque REQ-4 sustituye el lanzamiento inmediato, sin perder validaciones.

## Prueba
HomeFalso/tempfile, Git bare y Claude falsos evitan configuración real/red. Ejecutar casos de escaneo con Git en hijos current_exe/--exact, como `tests/investigador.rs`: cada Command usa env_clear, HOME temporal, XDG_CONFIG_HOME, PATH de fixtures, GIT_CONFIG_NOSYSTEM=1 y GIT_CONFIG_GLOBAL/GIT_CONFIG_SYSTEM=/dev/null. Git espía verifica herencia; configs inválidas prueban aislamiento, sin variables globales.
Regla 4.4: logs estructurados en main/tui, éxito incluido; helpers puros aportan contexto. Usar capturar_logs existente sin duplicar subscribers.
Cada fase ejecuta `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked` y PTY aislada: 80×24, resize, pegado, errores, salida/restauración.
