# Graph Report - rebake  (2026-10-18)

## Corpus Check
- 78 files · ~82,557 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 3 file(s) not represented in the graph (top: (none) 3)

## Summary
- 1026 nodes · 2652 edges · 39 communities (33 shown, 6 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 203 edges (avg confidence: 0.86)
- Token cost: 292,727 input · 0 output

## Community Hubs (Navigation)
- Tests de la TUI
- Servidores MCP
- Recetas directas y posible origen
- Escaneo y fusión
- Acciones sobre recetas
- Home falso de tests
- Evidencia y Serena
- Perfiles y escaneo de archivos
- Checklist y pasos a mano
- Release y alternar
- Tests de auto-actualización
- Instalador y comprobación
- Filtro de secretos
- Entrada de la CLI
- Widgets de ratatui
- Cookbook desde un repo
- Archivo del cookbook
- Bucle de la TUI
- Estado de la App
- Teclas de Recetas
- Procesos con tope
- Posible origen y novedades
- Auto-actualización
- install.sh y releases
- Mensajes de commit
- Vista clara
- Filas de Pendientes
- Tests de procesos
- Dibujo de pestañas
- Estados y colores
- Detalle de un ítem
- Modos de entrada
- Pie y leyenda
- Camino y siguiente paso
- Marca de clon de rebake
- Binario rebake

## God Nodes (most connected - your core abstractions)
1. `App` - 85 edges
2. `Recetario` - 66 edges
3. `Item` - 43 edges
4. `Tipo` - 33 edges
5. `pantalla()` - 33 edges
6. `recetario()` - 30 edges
7. `HomeFalso` - 29 edges
8. `rebake()` - 29 edges
9. `tecla()` - 27 edges
10. `remoto()` - 26 edges

## Surprising Connections (you probably didn't know these)
- `a_mano()` --implements--> `Pasos a mano (rebake no los corre) en Pendientes`  [AMBIGUOUS]
  src/checklist.rs → intent/servidores-mcp/spec.md
- `plan_instalacion()` --implements--> `Pestaña Instalación con selección de recetas`  [INFERRED]
  src/tui/vista.rs → intent/elegir-que-instalar/spec.md
- `aceptar_posible()` --implements--> `Posible origen (búsqueda web con confirmación)`  [INFERRED]
  src/acciones.rs → intent/recetas-directas/intent.md
- `descartar_posible()` --implements--> `Posible origen (búsqueda web con confirmación)`  [INFERRED]
  src/acciones.rs → intent/recetas-directas/intent.md
- `instalar()` --implements--> `Verificación por checksum del binario nuevo`  [INFERRED]
  src/actualizar.rs → intent/auto-actualizar/intent.md

## Import Cycles
- 1-file cycle: `src/tui/mod.rs -> src/tui/mod.rs`
- 2-file cycle: `src/servicio.rs -> src/tui/vista.rs -> src/servicio.rs`
- 2-file cycle: `src/tui/app.rs -> src/tui/vista.rs -> src/tui/app.rs`
- 3-file cycle: `src/servicio.rs -> src/tui/vista.rs -> src/tui/app.rs -> src/servicio.rs`

## Hyperedges (group relationships)
- **AI-Native SDLC artifact chain (intent, spec, plan, evidence)** — intent_intent, intent_spec, intent_plan, intent_evidencia [EXTRACTED 1.00]
- **Recipe lifecycle: scan, resolve source, research, review, install, checklist** — intent_spec_data_flow, intent_spec_req_2, intent_spec_req_4, intent_spec_req_5, intent_spec_req_7, intent_spec_req_10, intent_spec_req_11 [EXTRACTED 1.00]
- **Release distribution and self-update via install.sh** — _github_workflows_release_release, _github_workflows_release_sha256_checksum, _github_workflows_release_draft_first_publish, readme_install_script, readme_auto_update [INFERRED 0.85]
- **Trabajos en segundo plano leídos por canal en el bucle de la TUI** — intent_auto_actualizar_intent_revision_segundo_plano, intent_elegir_que_instalar_spec_comprobacion_c, intent_recetas_directas_intent_posible_origen, src_tui_mod_arranque [INFERRED 0.85]
- **rebake no ejecuta ni cambia nada sin una acción explícita de Denis** — intent_elegir_que_instalar_intent_abrir_no_ejecuta, intent_servidores_mcp_intent_mcp_manual, intent_recetas_directas_intent_posible_origen, intent_auto_actualizar_spec_tecla_u, intent_cookbook_remoto_intent_solo_clon_propio [INFERRED 0.75]
- **Filtro de secretos sobre datos externos (git, .claude.json)** — intent_cookbook_remoto_spec_sin_credenciales, intent_servidores_mcp_intent_solo_mcpservers, src_secretos_ocultar, src_secretos_sin_credenciales_url [INFERRED 0.85]

## Communities (39 total, 6 thin omitted)

### Community 0 - "Tests de la TUI"
Cohesion: 0.06
Nodes (80): abrir_instalacion_no_pide_comprobar(), al_terminar_la_instalacion_resume_y_va_a_la_checklist(), aprobar_pendiente_muestra_error_y_no_guarda(), c_no_lanza_otra_comprobacion_si_hay_una_corriendo(), c_pide_comprobar_y_muestra_el_estado_de_cada_fila(), cada_pestana_dice_para_que_es(), camino(), CODEX (+72 more)

### Community 1 - "Servidores MCP"
Cohesion: 0.06
Nodes (64): Intent: avisar de los servidores MCP, Servidor MCP como caso excepcional de pasos manuales, Serena (servidor MCP), Plan: avisar de los servidores MCP, Spec: avisar de los servidores MCP, Pasos a mano (rebake no los corre) en Pendientes, detectar(), fuente_del_lock() (+56 more)

### Community 2 - "Recetas directas y posible origen"
Cohesion: 0.06
Nodes (73): Intent: recetas directas, sin investigar, Posible origen (búsqueda web con confirmación), REBAKE_MODELO (Sonnet por defecto), Receta directa (instalador oficial, sin claude -p), skills.sh (npx skills add), Plan: recetas directas, Spec: recetas directas, sin investigar, Guardas al instalar: falta marketplace o npx (REQ-4) (+65 more)

### Community 3 - "Escaneo y fusión"
Cohesion: 0.07
Nodes (36): no_filtra_secretos(), valor(), entorno(), PacmanFalso, perfil(), escanear_laburo(), fuente_de_un_perfil_gana_a_la_pista_del_otro(), hallazgo() (+28 more)

### Community 4 - "Acciones sobre recetas"
Cohesion: 0.08
Nodes (41): Plan: aprobar y excluir con la misma tecla, Marcas nueva/cambió de la última traída, aceptar_posible(), alternar_aprobacion(), aprobar(), buscar(), descartar_posible(), excluir() (+33 more)

### Community 5 - "Home falso de tests"
Cohesion: 0.13
Nodes (44): HomeFalso, aislar(), aprobado_con(), carpeta(), clonado(), clonar_en_carpeta_vacia_clona_y_cuenta_recetas(), clonar_no_muestra_credenciales_de_la_url(), clonar_no_pisa_una_carpeta_con_contenido() (+36 more)

### Community 6 - "Evidencia y Serena"
Cohesion: 0.05
Nodes (44): Serena project config (.serena/project.yml), Serena rust language server, Evidence 3: install dry-run, Evidence 4: real install into an empty HOME, Independent final branch review (7 important fixes), Evidence 2: real claude -p research, Evidence 1: real scan (73 items, no secrets), Security fixes during implementation (five risks) (+36 more)

### Community 7 - "Perfiles y escaneo de archivos"
Cohesion: 0.07
Nodes (28): asegurar_perfiles(), fusionar(), detectar(), PerfilDetectado, archivo_por_defecto(), civil_desde_dias(), contraer(), dir_documentos() (+20 more)

### Community 8 - "Checklist y pasos a mano"
Cohesion: 0.10
Nodes (34): a_mano(), ajustes(), aplanar(), CUBIERTAS, detectar(), en_perfil(), lineas_de(), logins() (+26 more)

### Community 9 - "Release y alternar"
Cohesion: 0.09
Nodes (28): release.yml (GitHub Actions release workflow), Static musl build (x86_64-unknown-linux-musl), release job, Release tarball sha256 checksum, Intent: aprobar y excluir con la misma tecla, Alternar aprobación con espacio/a, Alternar exclusión con x, Spec: aprobar y excluir con la misma tecla (+20 more)

### Community 10 - "Tests de auto-actualización"
Cohesion: 0.12
Nodes (15): ASSET, atender(), carpeta_sin_permiso_no_instala(), checksum_malo_no_instala(), Copia, instalar_reemplaza_el_binario(), PROPIA, publicada_igual_no_avisa() (+7 more)

### Community 11 - "Instalador y comprobación"
Cohesion: 0.12
Nodes (29): Plan: elegir qué instalar desde la URL del cookbook, abrir(), abrir_log(), comando(), comprobar(), Evento, Fin, Inicio (+21 more)

### Community 12 - "Filtro de secretos"
Cohesion: 0.10
Nodes (24): OCULTO, CLAVE_EN_P, clave_sensible(), comilla_abierta(), es_esquema_de_auth(), ocultar(), ocultar_palabra(), ocultar_par() (+16 more)

### Community 13 - "Entrada de la CLI"
Cohesion: 0.11
Nodes (17): avisar(), Cli, clonar(), Comando, Escanear, Instalar, Investigar, RenombrarPerfil (+9 more)

### Community 14 - "Widgets de ratatui"
Cohesion: 0.08
Nodes (12): cabecera(), DIRECTA, PASOS, SPINNER, TECLAS_INSTALACION, TECLAS_PENDIENTES, TECLAS_RESULTADOS, TECLAS_SIN_PLAN (+4 more)

### Community 15 - "Cookbook desde un repo"
Cohesion: 0.21
Nodes (20): Plan: cookbook desde un repo, Spec: cookbook desde un repo, rebake clonar <url>, Sin manejo de credenciales (GIT_TERMINAL_PROMPT=0, URLs limpias), Traer al arrancar y publicar al terminar, Clonar, clonar(), clonar_en() (+12 more)

### Community 16 - "Archivo del cookbook"
Cohesion: 0.13
Nodes (10): ejemplo(), guardar_a_traves_de_symlink_conserva_el_enlace(), ida_y_vuelta_sin_perdidas(), ids_repetidos_son_invalidos(), de_punta_a_punta_con_claude_falso(), escanear_crea_recetario(), investigar_en_lote_salta_las_sin_origen_y_las_lista(), recetario() (+2 more)

### Community 17 - "Bucle de la TUI"
Cohesion: 0.20
Nodes (11): actualizar(), Arranque, bucle(), buscar(), comprobar(), editar(), ejecutar(), escanear() (+3 more)

### Community 19 - "Teclas de Recetas"
Cohesion: 0.20
Nodes (11): Efecto, Buscar, Comprobar, Editar, Escanear, Guardar, Instalar, Investigar (+3 more)

### Community 20 - "Procesos con tope"
Cohesion: 0.15
Nodes (5): ejecutar(), lanzar(), MARGEN_DRENADO, matar_grupo(), Salida

### Community 21 - "Posible origen y novedades"
Cohesion: 0.16
Nodes (9): aplicar_posible(), Posible, Novedad, Cambio, Nueva, MAX_SALIDA, Pestana, Instalacion (+1 more)

### Community 22 - "Auto-actualización"
Cohesion: 0.26
Nodes (8): Plan: revisión en segundo plano, BASE_POR_DEFECTO, en_segundo_plano(), instalar(), parsear(), reabrir(), si_es_nueva(), version_nueva()

### Community 23 - "install.sh y releases"
Cohesion: 0.23
Nodes (11): fail(), main(), install.sh script, Intent: rebake se actualiza solo, GitHub Releases, Revisión de versión en segundo plano, Verificación por checksum del binario nuevo, Spec: rebake se actualiza solo (+3 more)

### Community 24 - "Mensajes de commit"
Cohesion: 0.24
Nodes (7): aprobada(), con(), cuenta_nuevos_y_cambios_de_estado(), novedades_marca_nuevas_y_cambiadas(), plural_y_sin_comando_para_la_tui(), sin_nuevos_ni_estados_dice_que_actualiza(), sin_origen_lista_pendientes_sin_fuente_y_trabajos_sin_ids_las_excluye()

### Community 25 - "Vista clara"
Cohesion: 0.25
Nodes (10): Pie con la acción según el estado del ítem (REQ-6), Plan: una pantalla que se entienda, Spec: una pantalla que se entienda, Ayuda con ? (REQ-9), Detalle en palabras simples (REQ-5), Pie solo con teclas que hacen algo (REQ-7), Nombres cortados por ancho en pantalla con … (REQ-4), Ayuda (+2 more)

### Community 26 - "Filas de Pendientes"
Cohesion: 0.36
Nodes (6): agrupar_ajustes(), ClaseFila, Marcable, Plegable, Titulo, Fila

### Community 27 - "Tests de procesos"
Cohesion: 0.33
Nodes (6): hijo_en_segundo_plano_no_cuelga(), nieto_en_otra_sesion_no_cuelga(), pasa_variables_de_entorno(), sh(), stdin_cerrado_no_cuelga(), tope_mata_al_grupo()

### Community 28 - "Dibujo de pestañas"
Cohesion: 0.44
Nodes (8): ayuda(), checklist(), cortar(), dibujar(), instalacion(), lineas_detalle(), plan_instalacion(), recetas()

### Community 29 - "Estados y colores"
Cohesion: 0.20
Nodes (8): Estado, Aprobada, Excluida, Pendiente, PorRevisar, estilo_estado(), LEYENDA, nombre_estado()

### Community 30 - "Detalle de un ítem"
Cohesion: 0.28
Nodes (7): cabeza_de_archivo(), prompt_origen(), Item, linea_item(), marca(), que_falta(), spinner()

### Community 31 - "Modos de entrada"
Cohesion: 0.22
Nodes (9): Entrada, Filtro, Link, Motivo, Modo, ConfirmarInstalacion, ConfirmarSalida, Escribiendo (+1 more)

### Community 32 - "Pie y leyenda"
Cohesion: 0.29
Nodes (6): Pestaña Checklist renombrada a Pendientes (REQ-10), Checklist, leyenda(), lineas_pie(), lineas_proposito(), teclas()

### Community 33 - "Camino y siguiente paso"
Cohesion: 0.40
Nodes (6): Camino escanear › armar recetas › aprobar › instalar (REQ-2), Línea Siguiente: con tecla y cantidad (REQ-3), armable(), linea_camino(), paso_actual(), siguiente_paso()

## Ambiguous Edges - Review These
- `a_mano()` → `Pasos a mano (rebake no los corre) en Pendientes`  [AMBIGUOUS]
  intent/servidores-mcp/plan.md · relation: implements
- `MCP servers as manual steps in Pendientes` → `REQ-3 Never read credential files`  [AMBIGUOUS]
  README.md · relation: conceptually_related_to

## Knowledge Gaps
- **66 isolated node(s):** `rebake`, `Desconocido`, `Entorno<'a>`, `Ok`, `Salteado` (+61 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 211 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Recetario` connect `Acciones sobre recetas` to `Servidores MCP`, `Recetas directas y posible origen`, `Perfiles y escaneo de archivos`, `Checklist y pasos a mano`, `Instalador y comprobación`, `Entrada de la CLI`, `Cookbook desde un repo`, `Archivo del cookbook`, `Estado de la App`, `Posible origen y novedades`, `Mensajes de commit`, `Detalle de un ítem`?**
  _High betweenness centrality (0.132) - this node is a cross-community bridge._
- **What connects `rebake`, `Desconocido`, `Entorno<'a>` to the rest of the system?**
  _66 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Tests de la TUI` be split into smaller, more focused modules?**
  _Cohesion score 0.06388666132050254 - nodes in this community are weakly interconnected._
- **Why does `App` connect `Estado de la App` to `Pie y leyenda`, `Camino y siguiente paso`, `Tests de la TUI`, `Acciones sobre recetas`, `Instalador y comprobación`, `Widgets de ratatui`, `Bucle de la TUI`, `Teclas de Recetas`, `Posible origen y novedades`, `Filas de Pendientes`, `Dibujo de pestañas`, `Estados y colores`, `Detalle de un ítem`, `Modos de entrada`?**
  _High betweenness centrality (0.127) - this node is a cross-community bridge._
- **Should `Servidores MCP` be split into smaller, more focused modules?**
  _Cohesion score 0.05583308845136644 - nodes in this community are weakly interconnected._

### Low-confidence Hints
_AMBIGUOUS edges — the extractor was unsure. Verify before acting on these._

- **What is the exact relationship between `a_mano()` and `Pasos a mano (rebake no los corre) en Pendientes`?**
  _Edge tagged AMBIGUOUS (relation: implements) - confidence is low._
- **What is the exact relationship between `MCP servers as manual steps in Pendientes` and `REQ-3 Never read credential files`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._