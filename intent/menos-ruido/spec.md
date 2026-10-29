# Spec: rediseño de Recetas (desde intent.md 2026-10-29)
Estado: borrador

## Requisitos
- REQ-1 MUST: Sincronizado: carpeta real dentro de la sincronización de cuenta (ej.: `~/.agents/skills/synced/`, `plugins/synced/`), incluidos enlaces `<perfil>/skills/<n>`. Recordarlo por instalación en cookbook: omitir listas/investigación/instalación, también sin archivos en otra PC; rechazar solicitud CLI explícita con guía. Homónimas propias fuera quedan intactas. Origen: Resultado 1.
- REQ-2 MUST: POR RESOLVER: incluidos con decisiones de receta pendientes, errores y posibles no resueltos. LISTAS: aprobadas sin bloqueos. Directas nuevas llegan gratis por revisar; hooks propios/imports siguen; vinculados dentro de su herramienta, externos en Checklist. Origen: Resultado 2/Restricciones.
- REQ-3 MUST: Enter aprueba/avanza, arma, acepta origen o abre Resolver: pegar link gratis, buscar, excluir, descartar posible; nunca desaprueba. LISTAS/excluidos indican siguiente paso. Origen: Resultado 3.
- REQ-4 MUST: TUI: avisar cantidad antes de `s`; otra tecla cancela sin gasto. `t`: solo pendientes con origen confirmado, tandas de hasta cinco, sin posibles/búsquedas/encadenamiento. Origen: Resultado 4/Restricciones.
- REQ-5 MUST: `x` excluye sin exigir motivo; `o` muestra/oculta excluidos; `x` reincluye conservando receta; persisten y no se investigan. Origen: Resultados 2/5.
- REQ-6 MUST: LISTO requiere cero POR RESOLVER, nada corriendo y último guardado exitoso. Hooks vinculados/externos no bloquean: sin receta propia. Indicar publicación confirmada/local y siguiente paso. Origen: Resultado 6.
- REQ-7 MUST: 80×24, NO_COLOR, sin Shift/ventanas nuevas: 1 Armar, 2 Instalar, 3 Checklist; salida siempre visible según foco (`q`; `Esc,q`; `s` si confirma), avisando si publicará y resultado. Origen: Resultados 7–8/Restricciones.
- REQ-8 MUST: Resultado de última acción explica cambios en el pie hasta próxima acción de Denis; avisos de fondo no lo pisan. Errores en detalle del ítem y POR RESOLVER hasta resolverse, sin pantallas nuevas. Origen: Resultado 8.
- REQ-9 MUST: Resolver: flechas/Enter/Esc, foco visible; Esc cancela/restaura. Texto/pegado sin acciones. Reordenar/filtrar/ocultar conserva selección por id y scroll acotado; si desaparece, fila válida/vacío. Origen: Resultados 2–3/Restricciones; tui-design.
- REQ-10 MUST: CLI: datos stdout; avisos/errores stderr, con causa/guía. Operación pedida fallida, incluso parcial, o rechazo devuelve distinto de cero; avisos recuperados de Git conservan código y guardado local. Origen: Resultados 1/8; cookbook-remoto, REQ-8; unix-cli.

## Capacidades y escenarios
### Sincronizados y CLI (REQ-1, REQ-10)
- GIVEN 2 skills homónimas, 1 real en synced y 1 propia fuera WHEN escanear THEN 1 propia elegible.
- GIVEN 1 sincronizado recordado, otra PC sin archivos WHEN investigar/instalar THEN 0 corridas/instalaciones; solicitud explícita falla con guía.
- GIVEN operación exitosa y 1 push fallido WHEN terminar THEN salida 0, aviso stderr y cookbook local.
### Lista y cierre (REQ-2, REQ-5, REQ-6)
- GIVEN 0 recetas pendientes, 0 errores/posibles, 1 hook vinculado, 1 externo, guardado exitoso WHEN terminar THEN LISTO.
- GIVEN 1 excluida WHEN `o,x` THEN 1 reincluida conservando receta.
### Acciones y cuota (REQ-3, REQ-4)
- GIVEN 2 por revisar WHEN Enter THEN 1 aprobada, 1 seleccionada, 0 corridas/instalaciones.
- GIVEN 1 receta Claude con origen WHEN Enter THEN pide `s`: «1 corrida»; `s` ejecuta 1, queda por revisar.
- GIVEN 1 posible cuya receta usa Claude WHEN Enter THEN pide `s`: «1 corrida»; `s` acepta y ejecuta 1.
- GIVEN 1 sin origen WHEN Resolver/pegar link THEN 1 origen guardado, 0 corridas, 0 aprobaciones.
- GIVEN 1 sin origen WHEN Resolver/buscar THEN pide `s`: «1 corrida»; `s` realiza 1 búsqueda, 0 armados.
- GIVEN confirmación de 1 corrida WHEN Esc THEN 0 corridas, 0 recetas nuevas.
- GIVEN 6 pendientes Claude con origen y 1 posible WHEN `t,s` THEN 5 corridas, 1 pendiente sin armar, posible intacto, 0 búsquedas; no encadena.
- GIVEN 1 directa nueva WHEN escanear THEN 1 por revisar, 0 corridas, sin pedir `s`.
### Resultados y foco (REQ-7, REQ-8, REQ-9)
- GIVEN 1 armado fallido WHEN llega aviso de fondo THEN resultado en pie; error en detalle/POR RESOLVER hasta resolverse.
- GIVEN 1 Resolver WHEN pegar texto y Esc THEN 0 acciones, foco restaurado y salida visible.

## Concerns
- Sin confirmación en `rebake investigar`; conservar `--dry-run`/`--si` y selección CLI actual.
