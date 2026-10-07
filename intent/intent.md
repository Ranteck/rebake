# Intención: recetario
Autor: Denis · Estado: aceptado · Fecha: 2026-10-07

## Problema
En cada formateo Denis rehace a mano su setup de Claude Code: "plugin que encontraba, plugin
que copiaba el comando de instalación". Va tres veces y cada vez creció: hoy son dos perfiles y
unos 40 plugins. Copiar carpetas no sirve: "pueden quedar truncados o
arruinados porque por ahí eso mismo tuvo actualización o dejaron de funcionar".

## Resultado esperado
Una CLI que funciona como recetario de instaladores:
1. Mira qué tiene instalado esta PC en los dos perfiles.
2. Para cada cosa busca "el repositorio de donde viene" y "cómo es el sistema de instalación",
   incluidas las reglas de la casa, con `claude -p`, una vez por cosa. Sin origen, acepta un
   link o una receta escrita a mano. Lo excluido no se vuelve a investigar.
3. Guarda el recetario en un solo archivo versionado, revisable en el diff.
4. En una PC recién formateada ejecuta las recetas: "que lo instale de esa manera".
5. Muestra una checklist de ajustes personales y logins.

Éxito: en una PC nueva, con la herramienta y el recetario, un comando deja los mismos plugins,
skills, house-rules y hooks en ambos perfiles y lista lo pendiente.

## Usuarios y sistemas afectados
Denis y sus perfiles `~/.claude` (laburo) y `~/.claude-personal` (personal). Claude Code,
skills.sh, los repos de origen y rtk, codex y node.

## Restricciones
- CLI con TUI: "Esto tiene que estar hecho en la CLI"; "se podría usar tui sino rust".
- Solo instaladores: la receta sale del repo de origen, no se inventa.
- Sin secretos: las credenciales se vuelven a loguear.
- Herramienta pública; el recetario vive aparte: "se podría guardar ese solo archivo en otro
  lado y la herramienta queda pública".
- AI-Native SDLC, sin sobreingeniería.

## Fuera de alcance
- Memorias, `~/.remember`, transcripts, planes e historial.
- Aplicar ajustes, texto propio de `CLAUDE.md` o credenciales: van a la checklist.
- Escritorio y paquetes de pacman: "pacman viene de arch, así que eso no hay que importar".

## Preguntas abiertas
Ninguna; las decisiones están en `spec.md`.

## Cambios
- 2026-10-07: herramienta pública y recetario aparte (antes: "clonar el repo y correr un
  comando"), porque el recetario guarda datos de Denis. Decisión de Denis.
- 2026-10-07: pacman fuera de alcance y TUI en Rust (decisiones de Denis); recorte por la
  plantilla de ai-native-sdlc. Aprobado.
