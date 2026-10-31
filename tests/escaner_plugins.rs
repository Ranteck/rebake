mod comun;
use comun::*;
use rebake::escaner::escanear;
use rebake::modelo::Via;

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
    h.escribir(
        &format!("{dir}/plugins/marketplaces/openai-codex/.claude-plugin/marketplace.json"),
        CATALOGO,
    );
}

#[test]
fn plugin_en_dos_perfiles_es_un_item() {
    if !comun::aislado("plugin_en_dos_perfiles_es_un_item") {
        return;
    }
    let h = HomeFalso::nuevo();
    let settings =
        r#"{"enabledPlugins": {"codex@openai-codex": true, "canva@openai-codex": false}}"#;
    perfil_con_plugins(&h, ".claude", settings);
    perfil_con_plugins(&h, ".claude-personal", settings);
    let pacman = PacmanFalso(vec![]);
    let e = escanear(
        &entorno(&h, &pacman),
        &[
            perfil("laburo", "~/.claude"),
            perfil("personal", "~/.claude-personal"),
        ],
    );
    let codex: Vec<_> = e
        .hallazgos
        .iter()
        .filter(|x| x.id == "plugin:codex@openai-codex")
        .collect();
    assert_eq!(codex.len(), 1);
    assert_eq!(codex[0].perfiles, ["laburo", "personal"]);
    assert!(e.hallazgos.iter().all(|x| !x.id.contains("canva")));
}

#[test]
fn plugin_toma_repo_del_catalogo() {
    if !comun::aislado("plugin_toma_repo_del_catalogo") {
        return;
    }
    let h = HomeFalso::nuevo();
    perfil_con_plugins(
        &h,
        ".claude",
        r#"{"enabledPlugins": {"codex@openai-codex": true, "superpowers@openai-codex": true}}"#,
    );
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    let repo = |id: &str| {
        e.hallazgos
            .iter()
            .find(|x| x.id == id)
            .unwrap()
            .fuente
            .clone()
            .unwrap()
    };
    assert_eq!(
        repo("plugin:codex@openai-codex").repo,
        "https://github.com/openai/codex-plugin-cc"
    );
    assert_eq!(
        repo("plugin:superpowers@openai-codex").repo,
        "https://github.com/obra/superpowers"
    );
    assert_eq!(repo("marketplace:openai-codex").via, Via::Metadatos);
    let codex = e
        .hallazgos
        .iter()
        .find(|x| x.id == "plugin:codex@openai-codex")
        .unwrap();
    assert_eq!(codex.requiere, ["marketplace:openai-codex"]);
}

#[test]
fn settings_invalido_avisa_y_sigue() {
    if !comun::aislado("settings_invalido_avisa_y_sigue") {
        return;
    }
    let h = HomeFalso::nuevo();
    h.escribir(".claude/settings.json", "{esto no es json");
    perfil_con_plugins(
        &h,
        ".claude-personal",
        r#"{"enabledPlugins": {"codex@openai-codex": true}}"#,
    );
    let pacman = PacmanFalso(vec![]);
    let e = escanear(
        &entorno(&h, &pacman),
        &[
            perfil("laburo", "~/.claude"),
            perfil("personal", "~/.claude-personal"),
        ],
    );
    assert!(
        e.avisos
            .iter()
            .any(|a| a.contains("~/.claude/settings.json")),
        "{:?}",
        e.avisos
    );
    assert!(
        e.hallazgos
            .iter()
            .any(|x| x.id == "plugin:codex@openai-codex")
    );
}

#[test]
fn plugin_sincronizado_se_recuerda_sin_ocultar_el_homonimo_propio() {
    if !comun::aislado("plugin_sincronizado_se_recuerda_sin_ocultar_el_homonimo_propio") {
        return;
    }
    let h = HomeFalso::nuevo();
    for (dir, ruta) in [
        (".claude", ".claude/plugins/synced/cuenta/p"),
        (".claude-personal", ".claude-personal/plugins/cache/m/p/1"),
    ] {
        h.escribir(
            &format!("{dir}/settings.json"),
            r#"{"enabledPlugins":{"p@m":true}}"#,
        );
        let manifest = h.escribir(
            &format!("{ruta}/.claude-plugin/plugin.json"),
            r#"{"name":"p"}"#,
        );
        let instalacion = manifest.parent().unwrap().parent().unwrap();
        h.escribir(&format!("{dir}/plugins/installed_plugins.json"), &serde_json::json!({"version":2,"plugins":{"p@m":[{"scope":"user","installPath":instalacion,"version":"1","installedAt":"2026-10-29","lastUpdated":"2026-10-29","gitCommitSha":"abc"}]}}).to_string());
    }
    let perfiles = vec![
        perfil("laburo", "~/.claude"),
        perfil("personal", "~/.claude-personal"),
    ];
    let e = escanear(&entorno(&h, &PacmanFalso(vec![])), &perfiles);
    let mut r = rebake::modelo::Recetario::nuevo();
    r.perfiles = perfiles;
    rebake::fusion::fusionar(&mut r, e, &|_| false);
    let p = r.item("plugin:p@m").unwrap();
    assert_eq!(p.sincronizado_en, ["laburo"]);
    assert_eq!(
        p.perfiles_propios().cloned().collect::<Vec<_>>(),
        ["personal"]
    );
}

#[test]
fn plugins_y_marketplaces_usan_descripcion_del_autor_y_fallback() {
    let h = HomeFalso::nuevo();
    perfil_con_plugins(
        &h,
        ".claude",
        r#"{"enabledPlugins":{"codex@openai-codex":true,"superpowers@openai-codex":true}}"#,
    );
    h.escribir(".claude/plugins/installed_plugins.json", r#"{"plugins":{"codex@openai-codex":[{"installPath":"~/cache/codex"}],"superpowers@openai-codex":[{"installPath":"~/cache/superpowers"}]}}"#);
    h.escribir(
        "cache/codex/.claude-plugin/plugin.json",
        r#"{"description":"Plugin author's text."}"#,
    );
    h.escribir("cache/superpowers/.claude-plugin/plugin.json", "{}");
    h.escribir(".claude/plugins/marketplaces/openai-codex/.claude-plugin/marketplace.json", r#"{"plugins":[{"name":"codex","source":"./plugins/codex","description":"Do not override the manifest."},{"name":"superpowers","description":"Catalog author's text."}]}"#);
    let pacman = PacmanFalso(vec![]);
    let mut entorno = entorno(&h, &pacman);
    entorno.path.clear();
    for manifiesto in [
        r#"{"description":"Marketplace author's text."}"#,
        r#"{"metadata":{"description":"Marketplace author's text."}}"#,
    ] {
        h.escribir("x/.claude-plugin/marketplace.json", manifiesto);
        h.escribir(
            ".claude/plugins/known_marketplaces.json",
            &CONOCIDOS.replace("\"x\"", &format!("\"{}\"", h.ruta().join("x").display())),
        );
        let e = escanear(&entorno, &[perfil("laburo", "~/.claude")]);
        for (id, texto) in [
            ("plugin:codex@openai-codex", "Plugin author's text."),
            ("plugin:superpowers@openai-codex", "Catalog author's text."),
            ("marketplace:openai-codex", "Marketplace author's text."),
        ] {
            assert_eq!(
                e.hallazgos
                    .iter()
                    .find(|h| h.id == id)
                    .unwrap()
                    .descripcion
                    .as_deref(),
                Some(texto)
            );
        }
        assert!(e.avisos.is_empty(), "{:?}", e.avisos);
    }
}
