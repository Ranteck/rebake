use recetario::escaner::{Escaneo, Hallazgo};
use recetario::fusion::{asegurar_perfiles, fusionar};
use recetario::modelo::*;
use recetario::perfiles::PerfilDetectado;
use std::path::{Path, PathBuf};

fn base() -> Recetario {
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        Perfil { nombre: "laburo".into(), dir: "~/.claude".into() },
        Perfil { nombre: "personal".into(), dir: "~/.claude-personal".into() },
    ];
    r
}

fn hallazgo(id: &str, tipo: Tipo, perfiles: &[&str]) -> Hallazgo {
    let mut h = Hallazgo::nuevo(id, tipo, None);
    h.perfiles = perfiles.iter().map(|p| p.to_string()).collect();
    h
}

fn escaneo(hallazgos: Vec<Hallazgo>) -> Escaneo {
    Escaneo { hallazgos, ..Default::default() }
}

fn nunca(_: &str) -> bool { false }

#[test]
fn perfiles_nuevos_se_guardan_con_tilde() {
    let home = Path::new("/h");
    let detectados = vec![
        PerfilDetectado { nombre_sugerido: "claude".into(), dir: PathBuf::from("/h/.claude") },
        PerfilDetectado { nombre_sugerido: "personal".into(), dir: PathBuf::from("/h/.claude-personal") },
        PerfilDetectado { nombre_sugerido: "laburo".into(), dir: PathBuf::from("/h/.claude-laburo") },
    ];
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil { nombre: "laburo".into(), dir: "~/.claude".into() });
    asegurar_perfiles(&mut r, &detectados, home);
    asegurar_perfiles(&mut r, &detectados, home);
    let pares: Vec<_> = r.perfiles.iter().map(|p| (p.nombre.as_str(), p.dir.as_str())).collect();
    assert_eq!(pares, [("laburo", "~/.claude"), ("personal", "~/.claude-personal"), ("laburo-2", "~/.claude-laburo")]);
}

#[test]
fn aprobado_no_cambia_al_reescanear() {
    let mut r = base();
    let mut item = Item::nuevo("import:HOUSE-RULES.md", Tipo::Import);
    item.perfiles = vec!["laburo".into()];
    item.estado = Estado::Aprobada;
    item.fuente = Some(Fuente { repo: "https://github.com/ejemplo/house-rules".into(), via: Via::Symlink, doc: None });
    item.pasos = vec![Paso { cmd: "sh install.sh".into(), modo: Modo::Auto, por_perfil: true, cita: None, nota: None }];
    r.items.push(item.clone());

    let mut h = hallazgo("import:HOUSE-RULES.md", Tipo::Import, &["laburo", "personal"]);
    h.fuente = Some(Fuente { repo: "https://github.com/otro/repo".into(), via: Via::Metadatos, doc: None });
    fusionar(&mut r, escaneo(vec![h]), &nunca);

    let x = r.item("import:HOUSE-RULES.md").unwrap();
    assert_eq!(x.estado, Estado::Aprobada);
    assert_eq!(x.pasos, item.pasos);
    assert_eq!(x.fuente, item.fuente);
    assert_eq!(x.perfiles, ["laburo", "personal"]);
}

#[test]
fn desinstalado_queda_ausente() {
    let mut r = base();
    let mut viejo = Item::nuevo("plugin:viejo@x", Tipo::Plugin);
    viejo.estado = Estado::Aprobada;
    r.items.push(viejo);
    r.items.push(Item::nuevo("herramienta:codex", Tipo::Herramienta));
    fusionar(&mut r, escaneo(vec![]), &|nombre| nombre == "codex");
    assert!(r.item("plugin:viejo@x").unwrap().ausente);
    assert!(!r.item("herramienta:codex").unwrap().ausente);
}

#[test]
fn excluido_sigue_excluido() {
    let mut r = base();
    let mut x = Item::nuevo("skill:crawl4ai", Tipo::Skill);
    x.perfiles = vec!["personal".into()];
    x.estado = Estado::Excluida;
    x.motivo = Some("no me interesa".into());
    r.items.push(x);
    fusionar(&mut r, escaneo(vec![hallazgo("skill:crawl4ai", Tipo::Skill, &["personal"])]), &nunca);
    let x = r.item("skill:crawl4ai").unwrap();
    assert_eq!(x.estado, Estado::Excluida);
    assert_eq!(x.motivo.as_deref(), Some("no me interesa"));
}

#[test]
fn nuevo_entra_pendiente() {
    let mut r = base();
    r.checklist.sistema.push(PaqueteSistema { paquete: "jq".into(), para: "import:X.md".into() });
    let mut h = hallazgo("plugin:codex@openai-codex", Tipo::Plugin, &["laburo"]);
    h.requiere = vec!["marketplace:openai-codex".into()];
    h.pista = Some("plugin sin entrada en el catálogo".into());
    let mut e = escaneo(vec![h]);
    e.ajustes.push(Ajuste { perfil: "laburo".into(), clave: "theme".into(), valor: "dark".into() });
    fusionar(&mut r, e, &nunca);
    let x = r.item("plugin:codex@openai-codex").unwrap();
    assert_eq!(x.estado, Estado::Pendiente);
    assert_eq!(x.requiere, ["marketplace:openai-codex"]);
    assert_eq!(x.pista.as_deref(), Some("plugin sin entrada en el catálogo"));
    assert_eq!(r.checklist.ajustes.len(), 1);
    assert_eq!(r.checklist.sistema.len(), 1);
}
