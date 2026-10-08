# Evidencia: recetario (desde plan.md 2026-10-07)
Fecha: 2026-10-07 · Máquina: la de Denis (CachyOS, perfiles `~/.claude` y `~/.claude-personal`)

Calidad del código al cierre: `cargo fmt --check` y `cargo clippy --all-targets -- -D warnings`
limpios; `cargo test` 78/78. La TUI se probó con un arnés pty (14/14 chequeos del Step 6 de la
Task 14).

## Evidencia 1 — Escaneo real

**Pregunta:** ¿el escaneo encuentra todo lo instalado en los dos perfiles, con su origen, sin
guardar secretos? (REQ-1, REQ-2, REQ-3, REQ-4, REQ-11)

**Procedimiento:**
```bash
recetario --archivo /tmp/recetario-evidencia/recetario.toml escanear
grep -nE 'sk-ant|oauth|refresh_token|ghp_' /tmp/recetario-evidencia/recetario.toml
```

**Resultado:** 73 ítems (38 plugins, 22 skills, 4 marketplaces, 3 imports, 3 hooks, 1
statusline, 1 herramienta, `claude`), dos perfiles (`claude` → `~/.claude`, `personal` →
`~/.claude-personal`), sin avisos y sin secretos.
- `plugin:codex@openai-codex` → `https://github.com/openai/codex-plugin-cc` (metadatos).
- `import:HOUSE-RULES.md` → `Ranteck/house-rules` e `import:AI-NATIVE-SDLC.md` →
  `Ranteck/ai-native-sdlc` (symlink).
- `herramienta:rtk` → `https://github.com/rtk-ai/rtk` (URL dentro del binario).
- `statusline:statusline.sh` → pendiente, "archivo sin origen conocido".
- Checklist: 68 ajustes, entre ellos `modelSettings.claude-opus-5-5.effortLevel = xhigh` en los
  dos perfiles, y los títulos del texto propio de cada `CLAUDE.md`.

**Desvíos encontrados y corregidos:**
- El hook `SessionStart` de ai-native-sdlc (`sh '…/load-intent.sh'`) quedaba sin fuente: REQ-2
  pide resolver hooks "a un binario o un archivo" y solo se resolvía el binario. Además el id
  guardaba la ruta absoluta del HOME. Corregido en `4604ebb`; ahora va a `Ranteck/ai-native-sdlc`
  y el id usa `~`.
- `skill:graph-engineer` mostraba fuente y, a la vez, la pista "sin origen" de la copia del
  perfil personal. Corregido en `4604ebb`.

**Conclusión:** REQ-1, REQ-2, REQ-3, REQ-4 y REQ-11 se cumplen sobre datos reales.

## Evidencia 2 — Investigación real

**Pregunta:** ¿`claude -p` devuelve la instalación que documenta cada repo, con citas
verificables? (REQ-5, REQ-6)

**Procedimiento:**
```bash
recetario --archivo /tmp/recetario-evidencia/recetario.toml investigar \
  import:HOUSE-RULES.md import:AI-NATIVE-SDLC.md plugin:codex@openai-codex marketplace:openai-codex
```

**Resultado:** `4 por revisar, 0 con error`. Recetas:
- house-rules: `curl -fsSL …/house-rules/main/install.sh | CLAUDE_CONFIG_DIR="$CLAUDE_CONFIG_DIR" sh`
  por perfil; cita del README ("…| CLAUDE_CONFIG_DIR=/path/to/profile sh").
- ai-native-sdlc: `[ -d ~/Proyectos/ai-native-sdlc ] || git clone …` una vez e `install.sh` por
  perfil; citas del README.
- codex: marketplace y plugin con guardas, `codex login` y `/codex:setup` como pasos manuales;
  citas `/plugin marketplace add openai/codex-plugin-cc`, `/plugin install codex@openai-codex`.
- Paquetes del sistema a la checklist: `nodejs`, `npm`, `curl`, `git`, `jq`.

Las citas coinciden con los READMEs revisados al inicio del proyecto.

**Desvíos encontrados y corregidos:**
- La primera corrida devolvió recetas vacías marcadas como exitosas: en `claude -p`, `--tools`
  habilita WebSearch/WebFetch pero los usos se rechazan sin `--allowedTools`. Corregido en
  `0974d75`: se preaprueban y un permiso rechazado o un `doc` sin URL es un error.
- Requisitos con aclaraciones en el nombre (`openai-codex (openai/codex-plugin-cc)`) creaban
  ítems duplicados. Corregido en `d162a03`: se usa solo el identificador.

**Conclusión:** REQ-5 y REQ-6 se cumplen. Queda como menor que la investigación propone
`herramienta:claude`, que duplica conceptualmente al ítem `claude` (se excluye con un motivo).

## Evidencia 3 — Dry-run

**Pregunta:** ¿la instalación respeta el orden de requisitos y no ejecuta nada en modo
simulado? (REQ-10)

**Procedimiento:** aprobar los 4 ítems desde la TUI real (filtro + `a`) y correr
`recetario --archivo … instalar --dry-run`.

**Resultado:** los 4 ítems en orden (`marketplace:openai-codex` antes que el plugin), los pasos
por perfil con `[claude]`/`[personal]`, el `git clone` una sola vez, y los 4 `simulado`.

**Conclusión:** REQ-10 (orden, por perfil, dry-run) se cumple.

## Evidencia 4 — Instalación real en un HOME vacío

**Pregunta:** ¿las recetas aprobadas dejan el setup instalado en una máquina "limpia", se
pueden repetir y no tocan el setup real? (REQ-10, REQ-11)

**Procedimiento:**
```bash
H=$(mktemp -d) && mkdir -p "$H/.claude" "$H/.claude-personal"
HOME=$H recetario --archivo /tmp/recetario-evidencia/recetario.toml instalar --si   # dos veces
```

**Resultado:**
- Primera corrida: los 4 `instalado`. En los dos perfiles: `HOUSE-RULES.md`, symlinks
  `AI-NATIVE-SDLC.md` y `skills/ai-native-sdlc` al repo clonado en `$H/Proyectos`, `CLAUDE.md`
  con `@HOUSE-RULES.md` y `@AI-NATIVE-SDLC.md`, el hook `SessionStart` en `settings.json` y
  `codex@openai-codex` en `installed_plugins.json`.
- Segunda corrida: los 4 `salteado (ya estaba)`.
- Checklist impresa: logins por perfil, `gh auth login`, `codex login`, `/codex:setup` y los
  ajustes.
- `~/.claude` y `~/.claude-personal` reales sin cambios (fecha de `settings.json` y `CLAUDE.md`
  idéntica antes y después).

**Conclusión:** el resultado esperado del intent se cumple para estos ítems: con la herramienta
y el recetario, un comando deja el setup en los dos perfiles y lista lo que falta a mano.

## Correcciones de seguridad durante la implementación
Una revisión automática de los commits marcó cinco riesgos; todos se corrigieron con un test que
falló primero:
1. URLs de remotos con credenciales (`https://usuario:token@…`) se guardaban tal cual.
2. Comandos de hooks con secretos (`VAR=token cmd`, `Bearer …`) iban al id del ítem; la
   checklist solo ocultaba cuatro nombres de clave.
3. El archivo temporal de edición tenía un nombre predecible en `/tmp`.
4. Las primeras líneas de archivos enviadas a `claude -p` podían llevar secretos (riesgo de
   exfiltración si una página inyecta instrucciones).
5. "Ejecución de comandos generados por un LLM": es el diseño aprobado (concern 7 del spec);
   se mitiga con la aprobación obligatoria y la cita, sin cambio de código.
