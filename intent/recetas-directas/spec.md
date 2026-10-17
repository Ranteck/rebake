# Spec: recetas directas, sin investigar (desde recetas-directas/intent.md 2026-10-14)
Estado: aceptado

## Requisitos
Investigar escribe, sin `claude -p`, un paso por perfil (Origen de REQ-1 a REQ-3: Resultado 1):
- REQ-1 MUST: marketplace pendiente con repo: `claude plugin marketplace add <repo>`.
- REQ-2 MUST: plugin `<nombre>@<marketplace>`: `claude plugin install <nombre>@<marketplace>`;
  sigue requiriendo su marketplace.
- REQ-3 MUST: skill con origen en el lock de skills.sh:
  `npx -y skills add <repo> --skill <nombre> -g -a claude-code -y`.
- REQ-4 MUST: al instalar, si falta el marketplace del plugin o falta `npx`, esa receta falla
  diciendo qué falta y cómo resolverlo. Origen: Resultado 1 — "la receta lo avisa al instalar".
- REQ-5 MUST: la receta directa queda "por revisar"; aprobar e instalar no cambian. Origen:
  Resultado 2; Restricciones — "No cambia el paso de aprobación ni la instalación".
- REQ-6 MUST: la receta directa dice que es directa y trae comprobación por perfil: figura en el
  listado de `claude plugin`, o la skill tiene su `SKILL.md`. Origen: Resultado 4.
- REQ-7 MUST: lo demás con origen conocido se investiga con `claude -p` como hoy, con Sonnet;
  `REBAKE_MODELO` cambia el modelo. Origen: Resultados 3 y 5.
- REQ-8 MUST: una receta sin origen conocido no se manda a `claude -p` sola. En la pantalla, `i`
  sobre ella pide el link: con un link se investiga desde ese repo; con Enter vacío se busca el
  origen en la web; con Esc no cambia nada. Origen: Resultado 6.
- REQ-9 MUST: la receta cuyo origen se buscó lo dice en su detalle ("origen buscado en la web").
  Origen: Resultado 6 — "la receta dice que su origen fue buscado".
- REQ-10 MUST: `I` y `rebake investigar` sin ids investigan solo pendientes con origen, avisan
  cuántas quedaron sin origen y no tocan las demás. Origen: Resultado 6; Restricciones — "Una
  receta ya escrita o aprobada no se pisa".

## Capacidades y escenarios
### Recetas directas (REQ-1 a REQ-3, REQ-5, REQ-6)
- GIVEN `plugin:amd-skills@claude-plugins-official` pendiente — WHEN investigo — THEN su paso es
  `claude plugin install amd-skills@claude-plugins-official`, su comprobación busca ese id, queda
  por revisar y Claude no corrió
- GIVEN `skill:pragmatic-programmer` con origen del lock — WHEN investigo — THEN su paso es el
  `npx skills add` de `https://github.com/wondelai/skills` con `--skill pragmatic-programmer`
- GIVEN una receta directa aprobada en una PC donde ya está — WHEN `c` — THEN dice "instalada"

### Avisos al instalar (REQ-4)
- GIVEN un plugin aprobado y su marketplace sin registrar en el perfil — WHEN instalo — THEN esa
  receta falla con "falta el marketplace claude-plugins-official: aprobá
  marketplace:claude-plugins-official" y las demás siguen
- GIVEN una skill directa aprobada y sin `npx` en el PATH — WHEN instalo — THEN falla con "falta
  npx: instalá Node.js" (rechazo)

### Lo que sigue yendo a Claude (REQ-7)
- GIVEN `skill:ai-native-sdlc` con origen por symlink — WHEN investigo — THEN corre `claude -p`
  con `--model sonnet`, u `opus` con `REBAKE_MODELO=opus`

### Sin origen (REQ-8 a REQ-10)
- GIVEN `statusline:statusline.sh` sin origen — WHEN `i` — THEN la pantalla pide el link y Claude
  no corrió
- GIVEN ese pedido — WHEN Esc — THEN la receta sigue pendiente y no cambió
- GIVEN ese pedido — WHEN Enter vacío — THEN se busca en la web y el detalle dice "origen buscado
  en la web"
- GIVEN 3 pendientes, 1 sin origen — WHEN `I` o `rebake investigar` — THEN se investigan 2 y avisa
  "1 sin origen: investigala con `i`"
- GIVEN una receta aprobada y otra por revisar — WHEN `I` — THEN las dos quedan igual

## Concerns
- `npx` necesita Node, de pacman (fuera de alcance): rebake avisa, no lo instala.
- Verificado: `npx skills add -g` respeta `CLAUDE_CONFIG_DIR`.
- Enter vacío lanza una búsqueda paga, aunque el resultado queda por revisar.
- El spec raíz describe la búsqueda web automática: va una nota fechada, como en v0.4.0.

## Cambios
- 2026-10-14 (v0.6.2): REQ-10 también se cumple en la cabecera, que muestra fijo
  "○ N pendientes (M sin origen)"; el aviso de `I` queda corto y ya no se repite al cerrar el
  lote. Aprobado.
- 2026-10-17 (aprobado por Denis): "posible origen". REQ-8 y REQ-10 quedan reemplazados por:
  - REQ-11 MUST: `i` sobre un ítem sin origen no pide el link: lanza una búsqueda corta en la
    web (`claude -p`, solo lectura) que devuelve un repo y el motivo, sin receta. Origen:
    Resultado 6, "primero busca en la web de dónde podría venir".
  - REQ-12 MUST: `t` hace lo mismo para todos los pendientes sin origen, además de investigar
    los que tienen origen. Origen: Resultado 6, "también al investigar todas".
  - REQ-13 MUST: el posible origen queda guardado en el cookbook hasta que Denis responda, y el
    panel de la derecha lo muestra con su motivo y las teclas: Enter lo acepta, `n` lo descarta,
    `i` pide otro link. Origen: Resultado 6, "lo acepte o no… en el panel de la derecha".
  - REQ-14 MUST: aceptado, pasa a ser el origen ("origen buscado en la web", REQ-9) y sigue el
    camino normal: receta directa si corresponde, si no `claude -p`. Descartado o no
    encontrado, el panel ofrece pegar el link. Nunca se investiga ni instala con un posible sin
    aceptar. Origen: Resultado 6, "Sin origen aceptado no se investiga y no se instala".
  - REQ-15 MUST: mientras busca, la fila y el panel muestran un spinner animado y "buscando de
    dónde viene…". Origen: Resultado 6, "lo indica con un spinner".
  - REQ-16 SHOULD: `rebake investigar` sin ids sigue salteando los sin origen, como hoy (la
    confirmación solo existe en la pantalla). Origen: Resultado 6, "Denis lo acepte o no".
  - Escenarios: GIVEN `mcp:serena` sin origen — WHEN `i` — THEN spinner, y luego el panel dice
    "Posible origen: https://github.com/oraios/serena (motivo) · Enter acepta · n descarta ·
    i otro link". GIVEN ese posible — WHEN Enter — THEN se investiga desde ese repo. GIVEN ese
    posible — WHEN `n` — THEN queda sin origen y no se investigó. GIVEN 3 sin origen — WHEN `t`
    — THEN las 3 muestran spinner y después su posible; ninguna se investiga sola. GIVEN un
    posible sin responder — WHEN salgo y vuelvo a abrir — THEN sigue ahí.
  - REQ-17 MUST: `t` busca una sola vez por ítem: saltea los que tienen un posible sin
    responder y los ya buscados (no encontrado o descartado). Solo `i` vuelve a buscar. Origen:
    Resultado 6; cuidar la cuota (nota del 2026-10-17).
  - Concern: el primer `t` gasta una búsqueda por cada ítem sin origen (hoy 18).
