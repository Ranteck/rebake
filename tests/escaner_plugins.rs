mod comun;
use comun::*;
use recetario::escaner::escanear;
use recetario::modelo::Via;

const CONOCIDOS: &str = r#"{
  "openai-codex": {"source": {"source": "github", "repo": "openai/codex-plugin-cc"}, "installLocation": "x"}
}"#;
const CATALOGO: &str = r#"{"name": "openai-codex", "plugins": [
  {"name": "codex", "source": "./plugins/codex"},
  {"name": "superpowers", "source": {"source": "url", "url": "https://github.com/obra/superpowers.git"}}
]}"#;

fn perfil_con_plugins(h: &HomeFalso, dir: &str, settings: &str) {
    h.escribir(&format!("{dir}/settings.json"), settings);
    h.escribir(&format!("{dir}/plugins/known_marketplaces.json"), CONOCIDOS);
    h.escribir(&format!("{dir}/plugins/marketplaces/openai-codex/.claude-plugin/marketplace.json"), CATALOGO);
}

#[test]
fn plugin_en_dos_perfiles_es_un_item() {
    let h = HomeFalso::nuevo();
    let settings = r#"{"enabledPlugins": {"codex@openai-codex": true, "canva@openai-codex": false}}"#;
    perfil_con_plugins(&h, ".claude", settings);
    perfil_con_plugins(&h, ".claude-personal", settings);
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude"), perfil("personal", "~/.claude-personal")]);
    let codex: Vec<_> = e.hallazgos.iter().filter(|x| x.id == "plugin:codex@openai-codex").collect();
    assert_eq!(codex.len(), 1);
    assert_eq!(codex[0].perfiles, ["laburo", "personal"]);
    assert!(e.hallazgos.iter().all(|x| !x.id.contains("canva")));
}

#[test]
fn plugin_toma_repo_del_catalogo() {
    let h = HomeFalso::nuevo();
    perfil_con_plugins(&h, ".claude", r#"{"enabledPlugins": {"codex@openai-codex": true, "superpowers@openai-codex": true}}"#);
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    let repo = |id: &str| e.hallazgos.iter().find(|x| x.id == id).unwrap().fuente.clone().unwrap();
    assert_eq!(repo("plugin:codex@openai-codex").repo, "https://github.com/openai/codex-plugin-cc");
    assert_eq!(repo("plugin:superpowers@openai-codex").repo, "https://github.com/obra/superpowers");
    assert_eq!(repo("marketplace:openai-codex").via, Via::Metadatos);
    let codex = e.hallazgos.iter().find(|x| x.id == "plugin:codex@openai-codex").unwrap();
    assert_eq!(codex.requiere, ["marketplace:openai-codex"]);
}

#[test]
fn settings_invalido_avisa_y_sigue() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/settings.json", "{esto no es json");
    perfil_con_plugins(&h, ".claude-personal", r#"{"enabledPlugins": {"codex@openai-codex": true}}"#);
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude"), perfil("personal", "~/.claude-personal")]);
    assert!(e.avisos.iter().any(|a| a.contains("~/.claude/settings.json")), "{:?}", e.avisos);
    assert!(e.hallazgos.iter().any(|x| x.id == "plugin:codex@openai-codex"));
}
