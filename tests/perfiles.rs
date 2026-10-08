mod comun;
use comun::HomeFalso;
use rebake::perfiles::detectar;

#[test]
fn detecta_claude_y_claude_guion() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/settings.json", "{}");
    h.escribir(".claude-personal/settings.json", "{}");
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
