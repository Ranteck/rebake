# Spec: recetario (desde intent.md 2026-10-07)
Estado: aceptado

## Requisitos
- REQ-1 MUST: detecta como perfiles `~/.claude` y cada `~/.claude-*` con `settings.json`;
  Denis nombra cada perfil una vez y las rutas se guardan con `~`. Origen: Usuarios — "sus
  perfiles `~/.claude` (laburo) y `~/.claude-personal` (personal)".
- REQ-2 MUST: escanea sin modificar nada: marketplaces, plugins habilitados, skills (salvo
  `synced/`), imports `@X.md` de `CLAUDE.md`, hooks, statusline, herramientas que no son de
  pacman y Claude Code. Un ítem presente en varios perfiles es uno solo. Origen: Resultado 1 —
  "Mira qué tiene instalado esta PC en los dos perfiles"; Fuera de alcance — "pacman viene de
  arch".
- REQ-3 MUST: nunca lee `.credentials.json`, `~/.codex/auth.json` ni `.claude.json`. Origen:
  Restricciones — "Sin secretos".
- REQ-4 MUST: resuelve el repo de origen de cada ítem con esta prioridad: link manual >
  metadatos locales (catálogo del marketplace, symlink → remoto git, lock de skills.sh, URL de
  GitHub dentro del binario) > búsqueda web. Origen: Resultado 2 — "busca 'el repositorio de
  donde viene'".
- REQ-5 MUST: investiga cada ítem sin receta con `claude -p`, sin capacidad de ejecutar ni
  escribir, y obtiene la instalación que documenta su repo: pasos (comando, automático o
  manual, una vez o por perfil, cita literal de la doc), requisitos y una verificación
  opcional. Los pasos se pueden repetir sin fallar. Origen: Resultado 2 — "cómo es el sistema
  de instalación", "con `claude -p`, una vez por cosa".
- REQ-6 MUST: los requisitos que descubre la investigación se suman como ítems; los paquetes de
  pacman van a la checklist. Origen: Resultado 2; Fuera de alcance — pacman.
- REQ-7 MUST: Denis puede pegar un link, editar una receta, aprobarla o excluirla con motivo.
  Lo excluido no se investiga, no se instala ni se vuelve a proponer. Origen: Resultado 2 —
  "acepta un link o una receta escrita a mano. Lo excluido no se vuelve a investigar".
- REQ-8 MUST: el recetario es un solo archivo TOML, en la ruta de `--archivo` o en
  `~/.config/recetario/recetario.toml`; se valida estricto y se escribe de forma atómica.
  Origen: Resultado 3 — "un solo archivo versionado"; Restricciones — "el recetario vive
  aparte".
- REQ-9 MUST: volver a escanear agrega lo nuevo sin pisar estado, pasos, fuente ni motivo de lo
  revisado; lo que ya no está instalado se marca `ausente` y no se borra. Origen: Resultado 3 —
  "revisable en el diff".
- REQ-10 MUST: instala solo lo aprobado, con confirmación previa y en el orden de sus
  requisitos. Verifica por perfil y saltea donde ya está. Corre los pasos por perfil con
  `CLAUDE_CONFIG_DIR`, sin entrada por teclado y con tope de tiempo. Una falla bloquea solo a
  sus dependientes. `--dry-run` muestra los comandos sin ejecutarlos. Origen: Resultado 4 —
  "que lo instale de esa manera".
- REQ-11 MUST: la checklist se guarda en el recetario y lista logins, pasos manuales, ajustes
  por perfil (sin lo que cubren los instaladores; de `env` solo los nombres; claves sensibles
  ocultas), paquetes del sistema y los títulos del texto propio de cada `CLAUDE.md`. Origen:
  Resultado 5 — "checklist de ajustes personales y logins"; Fuera de alcance — "van a la
  checklist".
- REQ-12 MUST: TUI con pestañas Recetas, Checklist e Instalación, y subcomandos `escanear`,
  `investigar` e `instalar` sin pantalla. Origen: Restricciones — "Esto tiene que estar hecho
  en la CLI"; "se podría usar tui sino rust".
- REQ-13 MUST: el repo de la herramienta no contiene datos de Denis. Origen: Restricciones —
  "la herramienta queda pública".

## Capacidades y escenarios

### Escanear (REQ-1, REQ-2, REQ-3, REQ-9)
- GIVEN `codex@openai-codex` habilitado en los dos perfiles
- WHEN escaneo
- THEN hay un solo ítem `plugin:codex@openai-codex` con perfiles laburo y personal

- GIVEN `jq` instalado por pacman y usado por la statusline
- WHEN escaneo
- THEN `jq` no aparece como ítem

- GIVEN un ítem `aprobada`
- WHEN escaneo de nuevo
- THEN su estado, pasos y fuente no cambian

- GIVEN un plugin aprobado que ya no está instalado
- WHEN escaneo
- THEN queda marcado `ausente` y sigue en el recetario

- GIVEN un perfil con `.credentials.json`
- WHEN escaneo
- THEN ese archivo nunca se abre

### Resolver la fuente e investigar (REQ-4, REQ-5, REQ-6)
- GIVEN `@AI-NATIVE-SDLC.md` es un symlink a un repo con remoto `Ranteck/ai-native-sdlc`
- WHEN escaneo
- THEN la fuente es ese repo, con `via = "symlink"`

- GIVEN un ítem con fuente
- WHEN `claude -p` devuelve una receta válida
- THEN el ítem pasa a `por_revisar` con pasos, cita y URL de la doc

- GIVEN `claude -p` devuelve un JSON que no cumple el schema, o tarda más de 5 minutos
- WHEN investigo
- THEN el ítem queda `pendiente` con el error visible (rechazo)

- GIVEN la receta de `clangd-lsp` pide el paquete `clang`
- WHEN se guarda
- THEN `clang` aparece en la checklist y no como ítem a instalar

### Revisión manual (REQ-7)
- GIVEN `statusline.sh` sin origen conocido
- WHEN pego `https://github.com/nilbuild/claude-statusline`
- THEN la fuente queda con `via = "manual"` y se investiga ese repo

- GIVEN `skill:crawl4ai`
- WHEN la excluyo con motivo "no me interesa"
- THEN no se investiga, no se instala y no vuelve a proponerse

- GIVEN un ítem `pendiente` sin pasos
- WHEN intento aprobarlo
- THEN se rechaza (rechazo)

- GIVEN edito una receta y dejo un campo inválido
- WHEN guardo
- THEN veo el error y el archivo no cambia (rechazo)

### Archivo (REQ-8)
- GIVEN no existe recetario
- WHEN escaneo
- THEN se crea en `~/.config/recetario/recetario.toml`

- GIVEN un recetario con un campo desconocido
- WHEN abro `recetario`
- THEN no arranca y muestra línea y columna del error (rechazo)

### Instalar (REQ-10)
- GIVEN `codex@openai-codex` requiere `marketplace:openai-codex`
- WHEN instalo
- THEN el marketplace se instala antes que el plugin

- GIVEN la verificación de un ítem pasa en laburo y falla en personal
- WHEN instalo
- THEN sus pasos por perfil corren solo en personal

- GIVEN un paso que falla
- WHEN instalo
- THEN sus dependientes quedan bloqueados y el resto sigue

- GIVEN un paso que espera una respuesta por teclado
- WHEN instalo
- THEN falla sin colgar la instalación (rechazo)

- GIVEN un ciclo en `requiere`
- WHEN instalo
- THEN no se ejecuta nada y se informa el ciclo (rechazo)

- GIVEN `--dry-run`
- WHEN instalo
- THEN se muestran los comandos y no se ejecuta ninguno

### Checklist (REQ-11)
- GIVEN un `settings.json` con `effortLevel = "xhigh"`, `enabledPlugins` y `env.API_KEY`
- WHEN escaneo
- THEN la checklist muestra `effortLevel = xhigh`, no muestra `enabledPlugins` y muestra
  `env.API_KEY` sin valor

- GIVEN un `CLAUDE.md` con `@RTK.md` y la sección `## Correcciones de inglés`
- WHEN escaneo
- THEN la checklist lista el título de la sección y no el import

### Interfaz (REQ-12)
- GIVEN un recetario con ítems en los cuatro estados
- WHEN abro `recetario`
- THEN la pestaña Recetas los muestra agrupados por tipo, con su estado y el detalle del ítem
  elegido

### Herramienta pública (REQ-13)
- GIVEN el repo de la herramienta
- WHEN busco un `recetario.toml` o datos de Denis
- THEN no hay ninguno y `.gitignore` excluye `recetario.toml`

## Diseño

### Módulos

| Módulo | Responsabilidad |
|---|---|
| `perfiles` | Detecta los perfiles y expande rutas (REQ-1). |
| `escaner` | Un detector por tipo de ítem; devuelve hallazgos con pista (REQ-2, REQ-3). |
| `fuentes` | Resuelve el repo de cada hallazgo por prioridad (REQ-4). |
| `investigador` | Arma el prompt, llama a `claude -p` y valida la respuesta (REQ-5, REQ-6). |
| `recetario` | Modelo, lectura con validación, escritura atómica y fusión (REQ-7, REQ-8, REQ-9). |
| `instalador` | Orden por grafo, verificación por perfil y ejecución de pasos (REQ-10). |
| `checklist` | Ajustes, logins, pasos manuales, sistema y títulos de `CLAUDE.md` (REQ-11). |
| `tui` | Pantallas y teclas; solo usa los módulos de arriba (REQ-12). |
| `main` | Argumentos (`clap`), logging y traducción de errores a mensajes. |

### Flujo de datos

```
esta PC ──escaner──▶ hallazgos ──fuentes──▶ recetario.toml (pendiente)
                                               │
                     investigador (claude -p) ◀┘ ──▶ por_revisar
                                               │
              Denis en la TUI: aprobar / excluir / pegar link / editar
                                               │
         git diff + commit en el repo privado de Denis (fuera de la herramienta)
                                               │
PC nueva ──instalador──▶ recetas aprobadas, en orden ──▶ checklist
```

### Formato del recetario

```toml
version = 1

[[perfil]]
nombre = "laburo"
dir = "~/.claude"

[[perfil]]
nombre = "personal"
dir = "~/.claude-personal"

[[item]]
id = "import:AI-NATIVE-SDLC.md"   # tipo:nombre, estable entre escaneos
tipo = "import"   # claude | marketplace | plugin | skill | import | hook | statusline | herramienta
perfiles = ["laburo", "personal"]
estado = "aprobada"               # pendiente | por_revisar | aprobada | excluida
fuente = { repo = "https://github.com/Ranteck/ai-native-sdlc", via = "symlink", doc = "https://github.com/Ranteck/ai-native-sdlc#instalar" }
verificar = 'test -L "$CLAUDE_CONFIG_DIR/AI-NATIVE-SDLC.md"'   # corre una vez por perfil
investigado = "2026-10-07"

  [[item.paso]]
  cmd = "[ -d ~/Proyectos/ai-native-sdlc ] || git clone https://github.com/Ranteck/ai-native-sdlc.git ~/Proyectos/ai-native-sdlc"
  modo = "auto"                   # auto: lo ejecuta el instalador | manual: va a la checklist
  por_perfil = false              # una sola vez
  cita = "git clone https://github.com/Ranteck/ai-native-sdlc.git ~/Proyectos/ai-native-sdlc"

  [[item.paso]]
  cmd = "~/Proyectos/ai-native-sdlc/install.sh"
  modo = "auto"
  por_perfil = true               # una vez por perfil, con CLAUDE_CONFIG_DIR
  cita = "~/Proyectos/ai-native-sdlc/install.sh"

[[item]]
id = "plugin:codex@openai-codex"
tipo = "plugin"
perfiles = ["laburo", "personal"]
estado = "aprobada"
fuente = { repo = "https://github.com/openai/codex-plugin-cc", via = "metadatos", doc = "https://github.com/openai/codex-plugin-cc#install" }
requiere = ["marketplace:openai-codex", "herramienta:codex"]

  [[item.paso]]
  cmd = "claude plugin install codex@openai-codex"
  modo = "auto"
  por_perfil = true
  cita = "/plugin install codex@openai-codex"

  [[item.paso]]
  cmd = "/codex:setup"
  modo = "manual"
  cita = "Checks whether Codex is installed and authenticated."
  nota = "Dentro de Claude"

[[item]]
id = "statusline:statusline.sh"
tipo = "statusline"
perfiles = ["laburo", "personal"]
estado = "pendiente"
pista = "archivo sin origen conocido: ~/.claude/statusline.sh"

[[item]]
id = "skill:crawl4ai"
tipo = "skill"
perfiles = ["personal"]
estado = "excluida"
motivo = "no me interesa"

[[checklist.ajuste]]
perfil = "laburo"
clave = "modelSettings.claude-opus-5-5.effortLevel"
valor = "xhigh"

[[checklist.sistema]]
paquete = "jq"
para = "import:AI-NATIVE-SDLC.md"
```

Campos opcionales de `item`: `fuente`, `pista`, `requiere`, `verificar`, `investigado`,
`motivo` (solo `excluida`), `error` (último error de investigación), `ausente`, `paso`.
`CLAUDE_CONFIG_DIR` está definido siempre que corren `verificar` y los pasos por perfil.

### Reglas de instalación
- El escáner agrega las dependencias de estructura: todo lo de Claude requiere `claude` y cada
  plugin requiere su marketplace. La investigación agrega el resto.
- Los ítems sin perfiles (`claude` y las herramientas, como `rtk`) verifican y corren una sola
  vez.
- Por ítem con perfiles: `verificar` corre en cada perfil del ítem. Si pasa en todos, el ítem se saltea. Si
  no, los pasos `por_perfil = false` corren una vez y los `por_perfil = true` corren solo en los
  perfiles donde falló.
- Cada paso corre con `sh -c`, stdin cerrado y tope de 10 minutos. Salida completa en
  `~/.local/state/recetario/ultima-instalacion.log`.

### TUI

```
 recetario · laburo + personal          ✓ 38 aprobadas  ? 4 por revisar  ○ 2 pendientes  ✗ 9 excluidas
┌ Recetas ──────────────────────────────┐┌ plugin:codex@openai-codex ─────────────────────────┐
│ Claude Code                           ││ Estado:   ✓ aprobada        Perfiles: laburo, personal
│  ✓ claude                             ││ Fuente:   github.com/openai/codex-plugin-cc  (metadatos)
│ Marketplaces                          ││ Requiere: marketplace:openai-codex, herramienta:codex
│  ✓ openai-codex   ✓ trailofbits       ││
│ Plugins                               ││ Pasos
│ ▶✓ codex@openai-codex                 ││  1 auto  por perfil  claude plugin install codex@openai-codex
│  ? superpowers@claude-plugins-official││    cita: /plugin install codex@openai-codex
│  ⟳ remember@claude-plugins-official   ││  2 manual            /codex:setup
│ Imports CLAUDE.md                     ││
│  ✓ AI-NATIVE-SDLC.md                  ││
│ Statusline                            ││
│  ○ statusline.sh  sin origen          ││
│ Skills                                ││
│  ✗ crawl4ai  no me interesa           ││
└───────────────────────────────────────┘└────────────────────────────────────────────────────┘
 [1]Recetas [2]Checklist [3]Instalación   s escanear  i investigar  I investigar todo  a aprobar
 x excluir  l pegar link  e editar  r reinvestigar  P instalar  / filtrar  q salir
```

- La investigación corre en segundo plano (hasta 4 a la vez) y avisa a la TUI por un canal.
- Editar suspende la TUI mientras corre `$EDITOR` y la restaura al volver.
- Un ítem `ausente` se muestra atenuado; Denis decide si lo excluye o lo deja.
- Instalación: ítems en orden con `⏳ ✓ ⤼ ✗ ⊘` y un panel con la salida en vivo. Al terminar,
  resumen y salto a la Checklist, donde `espacio` marca como hecho solo durante la sesión.
- Subcomandos: resultados a stdout; progreso y errores a stderr.

### Investigación
`claude -p` con `--json-schema`, solo WebSearch y WebFetch, sin sesión persistente y sin los
hooks ni plugins del usuario. El prompt lleva id, tipo y pista del ítem, y las primeras 20
líneas cuando el ítem es un archivo (p. ej. `statusline.sh`). Pide la instalación que documenta
el repo, con prerrequisitos y pasos posteriores, flags no interactivos, pasos que se puedan
repetir sin fallar y la `cita` de cada comando.

### Errores
- Los archivos de Claude se leen tomando solo los campos necesarios; si uno no se puede leer,
  ese detector avisa (warn) y los demás siguen.
- `recetario.toml` es propio: validación estricta y error con posición.
- Los errores se traducen a mensajes en un solo lugar: la barra de estado de la TUI o `main`.
- Logs con `tracing` a `~/.local/state/recetario/recetario.log`, un evento por acción con campos
  elegidos, nunca valores de `settings.json`.

### Dependencias
`clap`, `ratatui` + `crossterm`, `serde` + `toml` + `serde_json`, `anyhow`, `tracing` +
`tracing-subscriber` + `tracing-appender`; para tests, `tempfile`. Sin cliente HTTP.

### Uso en una PC nueva
```bash
sudo pacman -S rust
cargo install --git https://github.com/Ranteck/recetario
mkdir -p ~/.config/recetario
ln -s ~/Proyectos/hyprland-config/claude/recetario.toml ~/.config/recetario/recetario.toml
recetario            # P → instalar, después la checklist
```

## Concerns
1. **La statusline puede no respetar `CLAUDE_CONFIG_DIR`:** su README dice que escribe en
   `~/.claude/statusline.sh`. Si la investigación lo confirma, el perfil personal necesita un
   paso manual. Puede pasar con otros instaladores; la investigación lo anota en `nota`.
2. **skills.sh conoce `~/.claude` pero no `~/.claude-personal`:** las skills enlazadas en los
   dos perfiles pueden necesitar un paso más para el personal.
3. **Modo autor contra modo README en tus repos:** house-rules instala una copia con `curl` y
   daily-flock se clona dentro de `skills/`, pero hoy los usás por symlink a `~/Proyectos/<repo>`
   porque los desarrollás. La receta sigue el README; si querés el modo autor en alguno, editás
   esa receta. ai-native-sdlc ya instala en modo autor.
4. **Costo de la primera investigación:** unos 50 ítems con `claude -p`, con la cuota de tu
   cuenta. Es una sola vez; después solo lo nuevo.
5. **El escáner no distingue las skills que trae la app de Claude** (son carpetas comunes en
   `~/.claude-personal/skills`). Las excluís una vez con motivo.
6. **La checklist guarda tus reglas `autoMode`,** que mencionan dónde hay credenciales en otros
   repos. El recetario tiene que vivir en un repo privado; la herramienta no lo controla.
7. **Riesgo residual de la investigación:** la `cita` ayuda a revisar, pero un comando
   equivocado que apruebes se ejecuta. La barrera es tu revisión.
8. **La evidencia de punta a punta** (instalación real en un HOME temporal) descarga plugins y
   herramientas: tarda y necesita red.
