use rebake::instalador::*;
use rebake::modelo::*;
use std::fs;
use std::path::Path;
use std::time::Duration;

fn auto(cmd: &str, por_perfil: bool) -> Paso {
    Paso {
        cmd: cmd.into(),
        modo: Modo::Auto,
        por_perfil,
        cita: None,
        nota: None,
    }
}

fn aprobado(id: &str, requiere: &[&str], pasos: Vec<Paso>) -> Item {
    let mut i = Item::nuevo(id, Tipo::Herramienta);
    i.estado = Estado::Aprobada;
    i.requiere = requiere.iter().map(|s| s.to_string()).collect();
    i.pasos = pasos;
    i
}

fn opciones(home: &Path, dry_run: bool) -> Opciones {
    Opciones {
        dry_run,
        tope: Duration::from_secs(10),
        home: home.to_path_buf(),
        log: Some(home.join("ultima.log")),
    }
}

fn correr(r: &Recetario, home: &Path, dry_run: bool) -> (Vec<(String, Resultado)>, Vec<Evento>) {
    let orden = planificar(r).unwrap();
    let mut eventos = Vec::new();
    let res = instalar(r, &orden, &opciones(home, dry_run), &mut |e| {
        eventos.push(e)
    });
    (res, eventos)
}

#[test]
fn solo_aprobadas_en_orden() {
    let dir = tempfile::tempdir().unwrap();
    let traza = dir.path().join("traza");
    let eco = |s: &str| auto(&format!("echo {s} >> '{}'", traza.display()), false);
    let mut r = Recetario::nuevo();
    r.items.push(aprobado(
        "plugin:p@m",
        &["marketplace:m"],
        vec![eco("plugin")],
    ));
    r.items.push(aprobado(
        "marketplace:m",
        &["claude"],
        vec![eco("marketplace")],
    ));
    r.items.push(aprobado("claude", &[], vec![eco("claude")]));
    let mut pendiente = aprobado("skill:x", &[], vec![eco("skill")]);
    pendiente.estado = Estado::Pendiente;
    r.items.push(pendiente);
    assert_eq!(
        planificar(&r).unwrap(),
        ["claude", "marketplace:m", "plugin:p@m"]
    );
    let (res, _) = correr(&r, dir.path(), false);
    assert!(res.iter().all(|(_, x)| *x == Resultado::Ok), "{res:?}");
    assert_eq!(
        fs::read_to_string(&traza).unwrap(),
        "claude\nmarketplace\nplugin\n"
    );
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
        Perfil {
            nombre: "uno".into(),
            dir: p1.display().to_string(),
        },
        Perfil {
            nombre: "dos".into(),
            dir: p2.display().to_string(),
        },
    ];
    let mut item = aprobado(
        "import:X.md",
        &[],
        vec![
            auto(&format!("echo global >> '{}'", traza.display()), false),
            auto("touch \"$CLAUDE_CONFIG_DIR/instalado\"", true),
        ],
    );
    item.perfiles = vec!["uno".into(), "dos".into()];
    item.verificar = Some(
        "test -f \"$CLAUDE_CONFIG_DIR/listo\" || test -f \"$CLAUDE_CONFIG_DIR/instalado\"".into(),
    );
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
    r.items
        .push(aprobado("a", &[], vec![auto("exit 7", false)]));
    r.items
        .push(aprobado("b", &["a"], vec![auto("true", false)]));
    r.items.push(aprobado("c", &[], vec![auto("true", false)]));
    let (res, _) = correr(&r, dir.path(), false);
    let de = |id: &str| res.iter().find(|(x, _)| x == id).unwrap().1.clone();
    assert!(matches!(de("a"), Resultado::Fallo(m) if m.contains('7')));
    assert!(matches!(de("b"), Resultado::Bloqueado(_)));
    assert_eq!(de("c"), Resultado::Ok);
    assert!(
        fs::read_to_string(dir.path().join("ultima.log"))
            .unwrap()
            .contains("exit 7")
    );
}

#[test]
fn dry_run_no_ejecuta() {
    let dir = tempfile::tempdir().unwrap();
    let marca = dir.path().join("marca");
    let mut r = Recetario::nuevo();
    let mut item = aprobado(
        "a",
        &[],
        vec![auto(&format!("touch '{}'", marca.display()), false)],
    );
    item.verificar = Some(format!("touch '{}.verificado'", marca.display()));
    r.items.push(item);
    let (res, eventos) = correr(&r, dir.path(), true);
    assert_eq!(res[0].1, Resultado::Simulado);
    assert!(!marca.exists());
    assert!(!dir.path().join("marca.verificado").exists());
    assert!(
        eventos
            .iter()
            .any(|e| matches!(e, Evento::Linea(_, t) if t.contains("touch")))
    );
}

fn presencias(r: &Recetario, home: &Path, tope: Duration) -> Vec<(String, Presencia)> {
    let ids = planificar(r).unwrap();
    let mut salida = Vec::new();
    comprobar(r, &ids, home, tope, &mut |id, p| salida.push((id, p)));
    salida
}

#[test]
fn comprobar_dice_en_que_perfil_falta_y_no_instala() {
    let dir = tempfile::tempdir().unwrap();
    let (p1, p2) = (dir.path().join("uno"), dir.path().join("dos"));
    fs::create_dir_all(&p1).unwrap();
    fs::create_dir_all(&p2).unwrap();
    fs::write(p1.join("listo"), "").unwrap();
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        Perfil {
            nombre: "uno".into(),
            dir: p1.display().to_string(),
        },
        Perfil {
            nombre: "dos".into(),
            dir: p2.display().to_string(),
        },
    ];
    let mut item = aprobado(
        "x",
        &[],
        vec![auto("touch \"$CLAUDE_CONFIG_DIR/instalado\"", true)],
    );
    item.perfiles = vec!["uno".into(), "dos".into()];
    item.verificar = Some("test -f \"$CLAUDE_CONFIG_DIR/listo\"".into());
    r.items.push(item);

    let p = presencias(&r, dir.path(), Duration::from_secs(10));
    assert_eq!(
        p,
        vec![("x".to_string(), Presencia::Falta(vec!["dos".into()]))]
    );
    assert!(!p1.join("instalado").exists() && !p2.join("instalado").exists());
}

#[test]
fn comprobar_instalada_y_sin_comprobacion() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = Recetario::nuevo();
    let mut a = aprobado("a", &[], vec![auto("true", false)]);
    a.verificar = Some("true".into());
    let b = aprobado("b", &[], vec![auto("true", false)]);
    let mut c = aprobado("c", &[], vec![auto("true", false)]);
    c.verificar = Some("false".into());
    r.items = vec![a, b, c];

    let p = presencias(&r, dir.path(), Duration::from_secs(10));
    assert_eq!(
        p,
        vec![
            ("a".to_string(), Presencia::Instalada),
            ("b".to_string(), Presencia::SinComprobacion),
            ("c".to_string(), Presencia::Falta(vec![])),
        ]
    );
}

#[test]
fn una_comprobacion_colgada_no_frena_a_las_demas() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = Recetario::nuevo();
    let mut a = aprobado("a", &[], vec![auto("true", false)]);
    a.verificar = Some("sleep 30".into());
    let mut b = aprobado("b", &[], vec![auto("true", false)]);
    b.verificar = Some("true".into());
    r.items = vec![a, b];

    let p = presencias(&r, dir.path(), Duration::from_secs(1));
    assert!(matches!(p[0].1, Presencia::NoSePudo(_)), "{:?}", p[0]);
    assert_eq!(p[1], ("b".to_string(), Presencia::Instalada));
}

#[test]
fn el_perfil_por_defecto_no_exporta_claude_config_dir() {
    // Claude Code usa `~/.claude.json` sin la variable y `~/.claude/.claude.json` con ella.
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path();
    let traza = home.join("traza");
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        Perfil {
            nombre: "claude".into(),
            dir: "~/.claude".into(),
        },
        Perfil {
            nombre: "personal".into(),
            dir: "~/.claude-personal".into(),
        },
    ];
    let cmd = format!(
        "echo \"$CLAUDE_CONFIG_DIR|$(sh -c 'echo $CLAUDE_CONFIG_DIR')\" >> '{}'",
        traza.display()
    );
    let mut item = aprobado("herramienta:x", &[], vec![auto(&cmd, true)]);
    item.perfiles = vec!["claude".into(), "personal".into()];
    item.verificar = Some(format!(
        "[ \"$CLAUDE_CONFIG_DIR\" = '{h}/.claude' ] && [ -z \"$(sh -c 'echo $CLAUDE_CONFIG_DIR')\" ] \
         || [ \"$(sh -c 'echo $CLAUDE_CONFIG_DIR')\" = '{h}/.claude-personal' ]",
        h = home.display()
    ));
    r.items.push(item);

    let mut presencia = None;
    comprobar(
        &r,
        &["herramienta:x".into()],
        home,
        Duration::from_secs(10),
        &mut |_, p| presencia = Some(p),
    );
    assert_eq!(presencia, Some(Presencia::Instalada));

    // La comprobación dio instalada: se saltea. Sin ella, corre el paso en los dos perfiles.
    r.items[0].verificar = None;
    let (res, _) = correr(&r, home, false);
    assert_eq!(res[0].1, Resultado::Ok);
    let h = home.display();
    assert_eq!(
        fs::read_to_string(&traza).unwrap(),
        format!("{h}/.claude|\n{h}/.claude-personal|{h}/.claude-personal\n")
    );
}
