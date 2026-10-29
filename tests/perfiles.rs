mod comun;
use comun::HomeFalso;
use rebake::perfiles::detectar;

#[test]
fn detecta_claude_y_claude_guion() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/settings.json", "{}");
    h.escribir(".claude-personal/settings.json", "{}");
    h.escribir(".claude-personal/.claude.json", "{}");
    h.escribir(".claude-vacio/otra-cosa.txt", "");
    h.escribir(".claude.json", "{}");
    let perfiles = detectar(h.ruta()).unwrap();
    let nombres: Vec<_> = perfiles
        .iter()
        .map(|p| p.nombre_sugerido.as_str())
        .collect();
    assert_eq!(nombres, ["claude", "personal"]);
    assert_eq!(perfiles[1].dir, h.ruta().join(".claude-personal"));
}

#[test]
fn settings_sin_marcador_de_claude_no_identifica_un_perfil() {
    let h = HomeFalso::nuevo();
    for nombre in [".claude", ".claude-mem", ".claude-cache", ".claude-laburo"] {
        h.escribir(&format!("{nombre}/settings.json"), "{}");
    }
    h.escribir(
        ".claude-laburo/.claude.json",
        "no hace falta leer el contenido",
    );
    let perfiles = detectar(h.ruta()).unwrap();
    let nombres: Vec<_> = perfiles
        .iter()
        .map(|p| p.nombre_sugerido.as_str())
        .collect();
    assert_eq!(nombres, ["laburo"]);
}

#[test]
fn el_marcador_debe_ser_archivo_y_settings_sigue_siendo_necesario() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/settings.json", "{}");
    h.escribir(".claude/.claude.json/otro", "");
    h.escribir(".claude-personal/.claude.json", "{}");
    assert!(detectar(h.ruta()).unwrap().is_empty());
}

#[test]
fn el_perfil_predeterminado_admite_el_marcador_dentro_de_su_carpeta() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/settings.json", "{}");
    h.escribir(".claude/.claude.json", "{}");
    assert_eq!(detectar(h.ruta()).unwrap()[0].nombre_sugerido, "claude");
}
