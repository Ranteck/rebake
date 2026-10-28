# Spec: hooks que instala su herramienta (desde intent.md 2026-10-27)
Estado: aceptado

## Requisitos
- REQ-1 MUST: vincular por ejecutable (primera palabra no asignación) coincidente con un
  ítem del cookbook, o por ruta ejecutada dentro de su carpeta solo si el nombre del
  ejecutable coincide con el del ítem. Sin ítem, una carpeta
  propia `~/.<nombre>/` identifica la herramienta externa `<nombre>`. Condicionales cuentan
  si ejecutan la ruta. Intérpretes, menciones, requisitos y directorios de perfil no prueban
  autoría; ante dudas, como hoy.
  Origen: Resultado esperado 1–4, «queda ligado» y «siguen como hoy»; Preguntas abiertas,
  «¿Cómo reconoce rebake de qué herramienta es un hook?», respondida aquí.
- REQ-2 MUST: el hook sigue como ítem del cookbook, vinculado al responsable y perfiles,
  marcado «lo instala <herramienta>» en TUI. Sin receta propia, no se investiga, aprueba
  ni ejecuta, aunque antes tuviera receta.
  Origen: Resultado esperado 1, «no tiene receta propia».
- REQ-3 MUST: al investigar con Claude una herramienta con hooks vinculados, pedir el
  registro documentado por su repo. La receta lo agrega después de instalarla; si el
  instalador ya crea hooks, no repetirlo. Sin comando documentado, queda pendiente y avisa,
  sin inventar. Mantener aprobación y alcance por perfil documentado.
  Origen: Resultado esperado 2, «incluye el comando que crea el hook»;
  Restricciones, «el comando sale del repo».
- REQ-4 MUST: el hook vinculado conserva comprobación sin aprobación, por presencia en la
  configuración de cada perfil. Herramienta instalada y hook faltante son resultados distintos;
  el binario no permite saltear el registro.
  Origen: Resultado esperado 1–2, «queda ligado» y «el hook aparece solo».
- REQ-5 MUST: ampliar la checklist de CLI y TUI, que hoy excluye hooks: una línea por
  herramienta externa con sus hooks, perfiles y aviso «aparecen al instalar o abrir
  <herramienta>». No instalar herramienta ni hooks.
  Origen: Resultado esperado 3, «la checklist dice que aparece al instalar o abrir esa
  herramienta»; Fuera de alcance.
- REQ-6 MUST: scripts propios y hooks no atribuibles conservan su flujo actual, aunque
  compartan carpeta o repo con un ítem.
  Origen: Resultado esperado 4, «siguen como hoy».
- REQ-7 MUST: no exponer secretos en cookbook, pantalla, logs ni investigación; no copiar
  hooks a la configuración.
  Origen: Restricciones, «Sin secretos» y «Nunca se copia el hook a mano».

## Capacidades y escenarios
### Reconocer y vincular (REQ-1, REQ-2)
- GIVEN RTK en el cookbook y un hook cuyo ejecutable, después de asignaciones, coincide
- WHEN se escanea
- THEN conserva el ítem hook, perfiles, comprobación y marca «lo instala RTK», sin receta.

- GIVEN impeccable en el cookbook y hooks con ejecutable `skills/impeccable/scripts/impeccable`
- WHEN se reescanean, incluso con recetas previas
- THEN quedan vinculados; no se investigan, aprueban ni ejecutan aparte.

### Registrar y comprobar (REQ-3, REQ-4)
- GIVEN RTK con hooks vinculados y registro documentado mediante `rtk init -g`
- WHEN Claude prepara su receta
- THEN incluye instalar y después registrar; si el instalador ya los crea, no duplica.

- GIVEN RTK instalado, pero falta el hook en un perfil
- WHEN se comprueba y luego instala
- THEN señala ese perfil y no saltea el registro por existir el binario.

- GIVEN registro sin documentación
- WHEN se investiga
- THEN queda pendiente, sin comandos inventados.

### Herramienta externa (REQ-1, REQ-2, REQ-5)
- GIVEN quince hooks que ejecutan scripts bajo `~/.orca/`, sin ítem Orca, mediante condicionales
- WHEN se escanea e instala
- THEN una línea de checklist agrupa Orca, hooks, perfiles y aviso; no se instalan.

### Compatibilidad y privacidad (REQ-1, REQ-6, REQ-7)
- GIVEN la skill `ai-native-sdlc` en el cookbook y el hook `skills/ai-native-sdlc/scripts/load-intent.sh`
- WHEN se escanea
- THEN conserva receta propia: `load-intent.sh` no coincide con `ai-native-sdlc`.

- GIVEN un hook con un secreto
- WHEN se escanea, investiga o instala
- THEN no expone el secreto ni reconstruye el hook copiándolo.

## Concerns
- Ante varios responsables posibles, conservar el flujo actual.
- Informar si el registro oficial cubre solo un perfil.
