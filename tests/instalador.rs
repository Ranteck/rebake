use recetario::instalador::*;
use recetario::modelo::*;
use std::fs;
use std::path::Path;
use std::time::Duration;

fn auto(cmd: &str, por_perfil: bool) -> Paso {
    Paso { cmd: cmd.into(), modo: Modo::Auto, por_perfil, cita: None, nota: None }
}

fn aprobado(id: &str, requiere: &[&str], pasos: Vec<Paso>) -> Item {
    let mut i = Item::nuevo(id, Tipo::Herramienta);
    i.estado = Estado::Aprobada;
    i.requiere = requiere.iter().map(|s| s.to_string()).collect();
    i.pasos = pasos;
    i
}

fn opciones(home: &Path, dry_run: bool) -> Opciones {
    Opciones { dry_run, tope: Duration::from_secs(10), home: home.to_path_buf(), log: Some(home.join("ultima.log")) }
}

fn correr(r: &Recetario, home: &Path, dry_run: bool) -> (Vec<(String, Resultado)>, Vec<Evento>) {
    let orden = planificar(r).unwrap();
    let mut eventos = Vec::new();
    let res = instalar(r, &orden, &opciones(home, dry_run), &mut |e| eventos.push(e));
    (res, eventos)
}

#[test]
fn solo_aprobadas_en_orden() {
    let dir = tempfile::tempdir().unwrap();
    let traza = dir.path().join("traza");
    let eco = |s: &str| auto(&format!("echo {s} >> '{}'", traza.display()), false);
    let mut r = Recetario::nuevo();
    r.items.push(aprobado("plugin:p@m", &["marketplace:m"], vec![eco("plugin")]));
    r.items.push(aprobado("marketplace:m", &["claude"], vec![eco("marketplace")]));
    r.items.push(aprobado("claude", &[], vec![eco("claude")]));
    let mut pendiente = aprobado("skill:x", &[], vec![eco("skill")]);
    pendiente.estado = Estado::Pendiente;
    r.items.push(pendiente);
    assert_eq!(planificar(&r).unwrap(), ["claude", "marketplace:m", "plugin:p@m"]);
    let (res, _) = correr(&r, dir.path(), false);
    assert!(res.iter().all(|(_, x)| *x == Resultado::Ok), "{res:?}");
    assert_eq!(fs::read_to_string(&traza).unwrap(), "claude\nmarketplace\nplugin\n");
}

#[test]
fn ciclo_no_ejecuta_nada() {
    let mut r = Recetario::nuevo();
    r.items.push(aprobado("a", &["b"], vec![]));
    r.items.push(aprobado("b", &["a"], vec![]));
    let err = planificar(&r).unwrap_err().to_string();
    assert!(err.contains("ciclo"), "{err}");
}

#[test]
fn verificar_por_perfil() {
    let dir = tempfile::tempdir().unwrap();
    let (p1, p2) = (dir.path().join("p1"), dir.path().join("p2"));
    fs::create_dir_all(&p1).unwrap();
    fs::create_dir_all(&p2).unwrap();
    fs::write(p1.join("listo"), "").unwrap();
    let traza = dir.path().join("traza");
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        Perfil { nombre: "uno".into(), dir: p1.display().to_string() },
        Perfil { nombre: "dos".into(), dir: p2.display().to_string() },
    ];
    let mut item = aprobado("import:X.md", &[], vec![
        auto(&format!("echo global >> '{}'", traza.display()), false),
        auto("touch \"$CLAUDE_CONFIG_DIR/instalado\"", true),
    ]);
    item.perfiles = vec!["uno".into(), "dos".into()];
    item.verificar = Some("test -f \"$CLAUDE_CONFIG_DIR/listo\" || test -f \"$CLAUDE_CONFIG_DIR/instalado\"".into());
    r.items.push(item);

    let (res, _) = correr(&r, dir.path(), false);
    assert_eq!(res[0].1, Resultado::Ok);
    assert!(!p1.join("instalado").exists());
    assert!(p2.join("instalado").exists());
    assert_eq!(fs::read_to_string(&traza).unwrap(), "global\n");

    let (res, _) = correr(&r, dir.path(), false);
    assert_eq!(res[0].1, Resultado::Salteado);
}

#[test]
fn falla_bloquea_dependientes() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = Recetario::nuevo();
    r.items.push(aprobado("a", &[], vec![auto("exit 7", false)]));
    r.items.push(aprobado("b", &["a"], vec![auto("true", false)]));
    r.items.push(aprobado("c", &[], vec![auto("true", false)]));
    let (res, _) = correr(&r, dir.path(), false);
    let de = |id: &str| res.iter().find(|(x, _)| x == id).unwrap().1.clone();
    assert!(matches!(de("a"), Resultado::Fallo(m) if m.contains('7')));
    assert!(matches!(de("b"), Resultado::Bloqueado(_)));
    assert_eq!(de("c"), Resultado::Ok);
    assert!(fs::read_to_string(dir.path().join("ultima.log")).unwrap().contains("exit 7"));
}

#[test]
fn dry_run_no_ejecuta() {
    let dir = tempfile::tempdir().unwrap();
    let marca = dir.path().join("marca");
    let mut r = Recetario::nuevo();
    let mut item = aprobado("a", &[], vec![auto(&format!("touch '{}'", marca.display()), false)]);
    item.verificar = Some(format!("touch '{}.verificado'", marca.display()));
    r.items.push(item);
    let (res, eventos) = correr(&r, dir.path(), true);
    assert_eq!(res[0].1, Resultado::Simulado);
    assert!(!marca.exists());
    assert!(!dir.path().join("marca.verificado").exists());
    assert!(eventos.iter().any(|e| matches!(e, Evento::Linea(_, t) if t.contains("touch"))));
}
