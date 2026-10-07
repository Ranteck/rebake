mod comun;
use comun::*;
use recetario::escaner::{escanear, Escaneo};
use recetario::modelo::Via;

fn escanear_laburo(h: &HomeFalso) -> Escaneo {
    h.escribir(".claude/settings.json", "{}");
    let pacman = PacmanFalso(vec![]);
    escanear(&entorno(h, &pacman), &[perfil("laburo", "~/.claude")])
}

fn hallazgo<'a>(e: &'a Escaneo, id: &str) -> &'a recetario::escaner::Hallazgo {
    e.hallazgos.iter().find(|x| x.id == id).unwrap_or_else(|| panic!("falta {id}: {:?}", e.hallazgos))
}

#[test]
fn skill_symlink_a_repo_usa_remoto() {
    let h = HomeFalso::nuevo();
    let repo = h.repo_git("Proyectos/graph-engineer", Some("https://github.com/ejemplo/graph-engineer.git"));
    h.escribir("Proyectos/graph-engineer/skills/graph-engineer/SKILL.md", "---\nname: graph-engineer\n---\n");
    h.enlazar(".claude/skills/graph-engineer", &repo.join("skills/graph-engineer"));
    let e = escanear_laburo(&h);
    let f = hallazgo(&e, "skill:graph-engineer").fuente.clone().unwrap();
    assert_eq!(f.repo, "https://github.com/ejemplo/graph-engineer");
    assert_eq!(f.via, Via::Symlink);
}

#[test]
fn repo_sin_remoto_queda_con_pista() {
    let h = HomeFalso::nuevo();
    let repo = h.repo_git("Proyectos/ai-native-sdlc", None);
    h.escribir("Proyectos/ai-native-sdlc/AI-NATIVE-SDLC.md", "# reglas\n");
    h.enlazar(".claude/AI-NATIVE-SDLC.md", &repo.join("AI-NATIVE-SDLC.md"));
    h.escribir(".claude/CLAUDE.md", "@AI-NATIVE-SDLC.md\n");
    let e = escanear_laburo(&h);
    let x = hallazgo(&e, "import:AI-NATIVE-SDLC.md");
    assert!(x.fuente.is_none());
    assert_eq!(x.pista.as_deref(), Some("repo local sin remoto: ~/Proyectos/ai-native-sdlc"));
}

#[test]
fn skill_del_lock_usa_su_fuente() {
    let h = HomeFalso::nuevo();
    let skill = h.escribir(".agents/skills/pragmatic-programmer/SKILL.md", "---\nname: pragmatic-programmer\n---\n");
    h.enlazar(".claude/skills/pragmatic-programmer", skill.parent().unwrap());
    h.escribir(
        ".local/state/skills/.skill-lock.json",
        r#"{"version": 3, "skills": {"pragmatic-programmer": {"source": "ejemplo/skills", "sourceType": "github", "sourceUrl": "https://github.com/ejemplo/skills.git"}}}"#,
    );
    let e = escanear_laburo(&h);
    let f = hallazgo(&e, "skill:pragmatic-programmer").fuente.clone().unwrap();
    assert_eq!(f.repo, "https://github.com/ejemplo/skills");
    assert_eq!(f.via, Via::Metadatos);
}

#[test]
fn synced_no_aparece() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/skills/synced/algo/SKILL.md", "x");
    h.escribir(".claude/skills/propia/SKILL.md", "x");
    let e = escanear_laburo(&h);
    assert!(e.hallazgos.iter().all(|x| !x.id.contains("synced")));
    assert_eq!(hallazgo(&e, "skill:propia").pista.as_deref(), Some("skill sin origen conocido: ~/.claude/skills/propia"));
}

#[test]
fn import_symlink_resuelve_repo() {
    let h = HomeFalso::nuevo();
    let repo = h.repo_git("Proyectos/house-rules", Some("https://github.com/ejemplo/house-rules"));
    h.escribir("Proyectos/house-rules/HOUSE-RULES.md", "# reglas\n");
    h.enlazar(".claude/HOUSE-RULES.md", &repo.join("HOUSE-RULES.md"));
    h.escribir(".claude/RTK.md", "# rtk\n");
    h.escribir(".claude/CLAUDE.md", "@RTK.md\n@HOUSE-RULES.md\n\n## Propio\ntexto\n");
    let e = escanear_laburo(&h);
    assert_eq!(hallazgo(&e, "import:HOUSE-RULES.md").fuente.clone().unwrap().via, Via::Symlink);
    assert_eq!(hallazgo(&e, "import:RTK.md").pista.as_deref(), Some("archivo sin origen conocido: ~/.claude/RTK.md"));
    assert_eq!(hallazgo(&e, "import:RTK.md").requiere, ["claude"]);
}

#[test]
fn remoto_con_credenciales_no_se_guarda() {
    let h = HomeFalso::nuevo();
    let repo = h.repo_git("Proyectos/privado", Some("https://usuario:ghp_SECRETO123@github.com/ejemplo/privado.git"));
    h.escribir("Proyectos/privado/skills/privado/SKILL.md", "---\nname: privado\n---\n");
    h.enlazar(".claude/skills/privado", &repo.join("skills/privado"));
    let e = escanear_laburo(&h);
    assert_eq!(hallazgo(&e, "skill:privado").fuente.clone().unwrap().repo, "https://github.com/ejemplo/privado");
    assert!(!format!("{e:?}").contains("SECRETO"));
}

#[test]
fn normalizar_url_saca_credenciales() {
    use recetario::escaner::normalizar_url;
    assert_eq!(normalizar_url("https://ghp_x@github.com/a/b.git/"), "https://github.com/a/b");
    assert_eq!(normalizar_url("https://github.com/a/b"), "https://github.com/a/b");
    assert_eq!(normalizar_url("git@github.com:a/b.git"), "git@github.com:a/b");
}

#[test]
fn normalizar_url_saca_query() {
    assert_eq!(recetario::escaner::normalizar_url("https://github.com/a/b.git?token=x"), "https://github.com/a/b");
}
