# Spec: rediseño de Recetas (desde intent.md 2026-10-29)
Estado: borrador

## Requisitos
- REQ-1 MUST: Omitir sincronizados de cuenta al escanear, mostrar, investigar e instalar, incluso antiguos y en otra PC. En CLI, un sincronizado solicitado explícitamente se rechaza con guía, sin gastar/ejecutar. Origen: Resultado 1, «ni desde la CLI».
- REQ-2 MUST: POR RESOLVER reúne sin receta/origen, por revisar, errores y posibles; LISTAS, aprobadas. Al escanear, las directas llegan armadas gratis, por revisar, sin sobrescribir recetas existentes. Hooks vinculados dentro de su herramienta; propios en lista; externos en Checklist; imports conservados. Origen: Resultado 2, «lo directo llega ya armado», y Restricciones.
- REQ-3 MUST: Enter aprueba una por revisar y avanza; arma o acepta origen; sin origen abre Resolver: pegar link gratis, buscar individualmente, excluir o descartar posible. En LISTAS orienta al próximo pendiente/instalación; en excluidos indica cómo reincluir. Nunca desaprueba. Origen: Resultado 3, «acción siguiente del ítem».
- REQ-4 MUST: Antes de cualquier corrida TUI, mostrar cantidad y esperar `s`; otra tecla cancela sin gasto. Aceptar origen puede requerir otra corrida, anunciada aparte. `t` solo arma con origen: gratis o tandas independientes de hasta cinco, sin búsquedas ni encadenamiento. Origen: Resultado 4, «nunca busca orígenes», y Restricciones.
- REQ-5 MUST: `x` excluye inmediatamente, sin exigir motivo; `o` revela/oculta excluidos y `x` reincluye conservando receta. Persisten, cuentan y no se investigan mientras estén excluidos. Origen: Resultados 2 y 5, «motivo es opcional».
- REQ-6 MUST: Mostrar COOKBOOK LISTO solo sin pendientes globales, trabajos activos ni error del último guardado; indicar publicación confirmada o «solo en esta PC» y siguiente paso. Origen: Resultado 6, «si quedó subido al repo».
- REQ-7 MUST: Pestañas: 1 Armar, 2 Instalar, 3 Checklist. En 80×24, sin Shift/ventanas nuevas y respetando NO_COLOR, pie contextual: `q` salir, `Esc,q` desde entradas; indicar `s` cuando requiera confirmación. Avisar si salir intentará publicar y su resultado final. Origen: Resultados 7–8, «Cómo salir», y Restricciones.
- REQ-8 MUST: Cada acción informa qué cambió hasta la próxima acción explícita; avisos de fondo no reemplazan ese resultado. Errores/resultados siguen consultables; guardado fallido impide LISTO. Origen: Resultado 8, «ningún aviso de fondo lo pisa».
- REQ-9 MUST: Resolver usa flechas/Enter/Esc y foco visible; Esc cancela y restaura foco. Texto/pegado, incluso saltos, no ejecutan acciones. Conservar selección por id al reordenar y acotar scroll al filtrar/ocultar; si desaparece, elegir fila válida o ninguna si queda vacío. Origen: Resultados 2–3, «Resolver», y Restricciones; tui-design.
- REQ-10 MUST: CLI: datos a stdout; costos/avisos/errores a stderr, con causa y guía; fallo o rechazo, incluso parcial, termina distinto de cero. Origen: Resultado 1 y Usuarios afectados; unix-cli.

## Capacidades y escenarios
### Exclusión y CLI (REQ-1, REQ-10)
- GIVEN sincronizado antiguo WHEN escanear/investigar/instalar THEN lote lo omite; solicitud explícita falla con guía en stderr.
### Organización (REQ-2, REQ-5)
- GIVEN directa nueva, hooks propios/vinculados/externos e imports WHEN escanear THEN directa por revisar sin Claude; hooks en lista/herramienta/Checklist e imports presentes.
- GIVEN receta excluida WHEN `o`, `x` THEN vuelve conservando pasos; excluir no exige motivo.
### Acción y cuota (REQ-3, REQ-4)
- GIVEN posible/origen conocido WHEN Enter/`t` THEN anuncia costo; sin `s`, cero corridas; aprobado avanza sin instalar.
### Cierre y resultados (REQ-6, REQ-8)
- GIVEN último pendiente WHEN aprobar THEN LISTO solo tras guardar bien; errores/posibles bloquean; publicación pendiente deja estado local y resultado durable.
### Navegación (REQ-7, REQ-9)
- GIVEN Resolver en 80×24 WHEN pegar/Esc/reordenar THEN sin ejecución accidental, foco/selección válidos y salida visible.
- GIVEN repo configurado WHEN salir y fallar la subida THEN informa fallo y conserva estado «solo en esta PC».

## Concerns
- «Subido» requiere publicación confirmada: guardar localmente no prueba sincronización.
- No excluir una skill propia solo por compartir nombre con una sincronizada.
- Mantener `--dry-run`/`--si`; no agregar confirmación a `rebake investigar` ni selectores nuevos a instalar.
