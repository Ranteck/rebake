use std::path::Path;

#[test]
fn no_versiona_datos_personales() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ignorados = std::fs::read_to_string(raiz.join(".gitignore")).unwrap();
    assert!(ignorados.lines().any(|l| l.trim() == "recetario.toml"));
    for dir in ["src", "tests"] {
        for entrada in walk(&raiz.join(dir)) {
            let texto = std::fs::read_to_string(&entrada).unwrap();
            // Partidos para que este mismo archivo no los contenga literalmente.
            for prohibido in ["/home/".to_string() + "denis", "flock".to_string() + "it"] {
                assert!(!texto.contains(&prohibido), "{} contiene {prohibido}", entrada.display());
            }
        }
    }
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() { out.extend(walk(&p)) } else { out.push(p) }
    }
    out
}
