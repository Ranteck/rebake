#[test]
fn no_arranca_con_recetario_invalido() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = dir.path().join("recetario.toml");
    std::fs::write(&ruta, "version = 1\n[[item]]\nid = \"claude\"\ntipo = \"claude\"\nestado = \"pendiente\"\ncolor = 1\n").unwrap();
    let err = recetario::tui::ejecutar(&ruta, dir.path())
        .unwrap_err()
        .to_string();
    assert!(err.contains("recetario.toml:"), "{err}");
    assert!(err.contains("color"), "{err}");
}
