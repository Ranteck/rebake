# Plan: rediseño de Recetas
Estado: borrador
Desde intent y spec aceptados del 2026-10-29; base v0.12.1. Ejecución propia, secuencial.

## Archivos que cambian
Existentes: `src/modelo.rs`, `src/escaner/mod.rs`, `src/escaner/archivos.rs`, `src/escaner/plugins.rs`, `src/fusion.rs`, `src/servicio.rs`, `src/instalador.rs`, `src/acciones.rs`, `src/investigador.rs`, `src/main.rs`, `src/sincro.rs`, `src/tui/app.rs`, `src/tui/vista.rs`, `src/tui/mod.rs`, `tests/comun/mod.rs`, `README.md`, `Cargo.toml`, `Cargo.lock` y los tests de la tabla.
Nuevo: `intent/menos-ruido/plan.md`. No crear módulos ni dependencias para lógica ya existente.

## Tests a escribir
Extender suites existentes; cada fila representa pruebas separadas por caso.

| Comportamiento observable | Ubicación | REQ |
| --- | --- | --- |
| Skills/plugins: destino real synced; dos homónimas, solo propia elegible | `tests/escaner_archivos.rs`, `tests/escaner_plugins.rs` | REQ-1 |
| Marca por instalación sobrevive guardar/cargar, renombrar/perder perfiles y otra PC sin archivos; cookbook antiguo válido | `tests/archivo.rs`, `tests/fusion.rs`, `tests/servicio.rs`, `tests/acciones.rs` | REQ-1 |
| Omitir lotes y destinos sincronizados; rechazo explícito antes de ejecutar | `tests/instalador.rs`, `tests/cli.rs` | REQ-1, REQ-10 |
| x sin motivo, o muestra/oculta, x reincluye conservando pasos; selección/filtro vacío válidos | `tests/acciones.rs`, `tests/tui.rs` | REQ-5, REQ-9 |
| Grupos, imports y hooks propios/vinculados/externos; estos últimos no bloquean LISTO | `tests/tui.rs` | REQ-2, REQ-6 |
| Una directa nueva: una por revisar, cero Claude, sin s; no sobrescribir existentes | `tests/servicio.rs`, `tests/investigador.rs` | REQ-2, REQ-4 |
| Enter: dos por revisar, una aprobada y siguiente seleccionada; cero instalación | `tests/tui.rs` | REQ-3 |
| Enter arma: avisa una corrida, cero antes de s, una después | `tests/tui.rs` | REQ-3, REQ-4 |
| Aceptar posible: una corrida tras s; descartar: cero | `tests/tui.rs` | REQ-3, REQ-4 |
| Pegar link: origen guardado, cero corridas/aprobaciones | `tests/tui.rs` | REQ-3 |
| Buscar origen: una búsqueda tras s, cero armados | `tests/tui.rs` | REQ-3, REQ-4 |
| Cancelar confirmación: cero corridas/recetas nuevas | `tests/tui.rs` | REQ-4 |
| Seis pendientes Claude y un posible: t+s lanza cinco, queda uno; posible intacto, sin encadenar | `tests/tui.rs` | REQ-4 |
| Resolver, Esc/foco, paste con saltos/q y resultados tardíos; id/scroll estables | `tests/tui.rs`, `src/tui/mod.rs` | REQ-9 |
| LISTO bloqueado por cada error/posible/corrida/guardado fallido, incluso con filtro | `tests/tui.rs`, `src/tui/mod.rs` | REQ-6 |
| Última acción permanece en pie ante aviso; error en detalle hasta resolución | `tests/tui.rs`, `src/tui/mod.rs` | REQ-8 |
| 80×24, cifras triples, Unicode/NO_COLOR, pestañas y salida visibles en cada modo | `tests/tui.rs` | REQ-7 |
| Publicación/no repo/fallo Git: estado veraz; stdout/stderr y fallo parcial versus aviso recuperado | `tests/sincro.rs`, `tests/cli.rs` | REQ-6, REQ-7, REQ-10 |
| Eventos éxito/fallo con contadores y sin secretos/rutas privadas, usando capturar_logs | `src/tui/mod.rs` | REQ-1, REQ-4, REQ-5, REQ-8 |

## Orden por fases integrables y publicables
Cada fase: tests rojos, cambio mínimo, tests focalizados y suite completa verdes, README, commit independiente; TUI usable sin flags temporales. Publicar requiere aprobación y actualizar Cargo.toml/Cargo.lock mediante el flujo existente.

1. **Quitar ruido** (REQ-1, REQ-5, REQ-9, REQ-10): añadir clasificación persistente por perfil a Item/Hallazgo, compatible con cookbooks antiguos. Resolver carpetas reales, agrupar/fusionar sin contaminar homónimos propios; conservar marcas ausentes y actualizar referencias al renombrar. Compartir criterio entre trabajos, sin_origen, por_armar, instalación y visibles. Motivo opcional, o/x y restauración de selección antes de ocultar. Entrega: sincronizados inaccesibles y excluidos recuperables.
2. **Ordenar y preparar** (REQ-2, REQ-6): reutilizar receta_directa/aplicar al escanear; separar POR RESOLVER/LISTAS, hooks agrupados y Checklist externo. Conservar el recorrido actual mientras llega Enter nuevo.
3. **Guiar acciones** (REQ-3, REQ-4, REQ-9): ampliar Modo/Efecto existentes para Resolver y confirmación individual; enlazar sin investigar automáticamente, sin confirmar en la CLI. Mantener tandas/IDs fijados, excluir posibles de t; un lector Crossterm procesa paste sin acciones y restaura foco/terminal.
4. **Cerrar claramente** (REQ-6–REQ-8, REQ-10): guardar informa éxito a App; separar resultados de avisos, conservar errores en ítems. Reutilizar Rama/publicar para estado local/subido, pasarlo al arranque y reportar salida; conservar recuperación Git y código original. Reordenar pestañas, pie, ayuda y README.

## Riesgos
No confundir falta de carpeta con desincronización, ni perfiles vacíos con ejecución global; conservar fixes v0.12.1. Ajustar expectativas antiguas solo por cambios aprobados.

## Prueba
HomeFalso/tempfile, Git bare local, Claude/binarios falsos; aislar HOME, XDG, CLAUDE_CONFIG_DIR y configuración Git por proceso, sin modificar variables globales. Nunca leer perfiles reales ni usar red/Claude real. Regla 4.4: logs en main/tui, éxito incluido; helpers puros aportan contexto; capturar_logs existente, sin subscribers duplicados.
Por fase: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked`; luego PTY aislada 80×24: recorrido, resize, pegado, errores, salida y restauración de terminal.
