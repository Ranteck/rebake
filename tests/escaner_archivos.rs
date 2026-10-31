mod comun;
use comun::*;
use rebake::escaner::{Escaneo, escanear};
use rebake::modelo::Via;

fn escanear_laburo(h: &HomeFalso) -> Escaneo {
    h.escribir(".claude/settings.json", "{}");
    let pacman = PacmanFalso(vec![]);
    escanear(&entorno(h, &pacman), &[perfil("laburo", "~/.claude")])
}

fn hallazgo<'a>(e: &'a Escaneo, id: &str) -> &'a rebake::escaner::Hallazgo {
    e.hallazgos
        .iter()
        .find(|x| x.id == id)
        .unwrap_or_else(|| panic!("falta {id}: {:?}", e.hallazgos))
}

#[test]
fn skill_symlink_a_repo_usa_remoto() {
    if !comun::aislado("skill_symlink_a_repo_usa_remoto") {
        return;
    }
    let h = HomeFalso::nuevo();
    let repo = h.repo_git(
        "Proyectos/graph-engineer",
        Some("https://github.com/ejemplo/graph-engineer.git"),
    );
    h.escribir(
        "Proyectos/graph-engineer/skills/graph-engineer/SKILL.md",
        "---\nname: graph-engineer\n---\n",
    );
    h.enlazar(
        ".claude/skills/graph-engineer",
        &repo.join("skills/graph-engineer"),
    );
    let e = escanear_laburo(&h);
    let f = hallazgo(&e, "skill:graph-engineer").fuente.clone().unwrap();
    assert_eq!(f.repo, "https://github.com/ejemplo/graph-engineer");
    assert_eq!(f.via, Via::Symlink);
}

#[test]
fn repo_sin_remoto_queda_con_pista() {
    if !comun::aislado("repo_sin_remoto_queda_con_pista") {
        return;
    }
    let h = HomeFalso::nuevo();
    let repo = h.repo_git("Proyectos/ai-native-sdlc", None);
    h.escribir("Proyectos/ai-native-sdlc/AI-NATIVE-SDLC.md", "# reglas\n");
    h.enlazar(".claude/AI-NATIVE-SDLC.md", &repo.join("AI-NATIVE-SDLC.md"));
    h.escribir(".claude/CLAUDE.md", "@AI-NATIVE-SDLC.md\n");
    let e = escanear_laburo(&h);
    let x = hallazgo(&e, "import:AI-NATIVE-SDLC.md");
    assert!(x.fuente.is_none());
    assert_eq!(
        x.pista.as_deref(),
        Some("repo local sin remoto: ~/Proyectos/ai-native-sdlc")
    );
}

#[test]
fn skill_del_lock_usa_su_fuente() {
    if !comun::aislado("skill_del_lock_usa_su_fuente") {
        return;
    }
    let h = HomeFalso::nuevo();
    let skill = h.escribir(
        ".agents/skills/pragmatic-programmer/SKILL.md",
        "---\nname: pragmatic-programmer\n---\n",
    );
    h.enlazar(
        ".claude/skills/pragmatic-programmer",
        skill.parent().unwrap(),
    );
    h.escribir(
        ".local/state/skills/.skill-lock.json",
        r#"{"version": 3, "skills": {"pragmatic-programmer": {"source": "ejemplo/skills", "sourceType": "github", "sourceUrl": "https://github.com/ejemplo/skills.git"}}}"#,
    );
    let e = escanear_laburo(&h);
    let f = hallazgo(&e, "skill:pragmatic-programmer")
        .fuente
        .clone()
        .unwrap();
    assert_eq!(f.repo, "https://github.com/ejemplo/skills");
    assert_eq!(f.via, Via::Metadatos);
}

#[test]
fn synced_no_aparece() {
    if !comun::aislado("synced_no_aparece") {
        return;
    }
    let h = HomeFalso::nuevo();
    h.escribir(".claude/skills/synced/algo/SKILL.md", "x");
    h.escribir(".claude/skills/propia/SKILL.md", "x");
    let e = escanear_laburo(&h);
    assert!(e.hallazgos.iter().all(|x| !x.id.contains("synced")));
    assert_eq!(
        hallazgo(&e, "skill:propia").pista.as_deref(),
        Some("skill sin origen conocido: ~/.claude/skills/propia")
    );
}

#[test]
fn las_copias_de_backup_no_aparecen() {
    if !comun::aislado("las_copias_de_backup_no_aparecen") {
        return;
    }
    let h = HomeFalso::nuevo();
    h.escribir(
        ".claude/skills/graph-engineer.backup-20260730-171339/SKILL.md",
        "x",
    );
    h.escribir(".claude/skills/vieja.bak/SKILL.md", "x");
    h.escribir(".claude/skills/graph-engineer/SKILL.md", "x");
    let e = escanear_laburo(&h);
    let skills: Vec<&str> = e
        .hallazgos
        .iter()
        .map(|x| x.id.as_str())
        .filter(|id| id.starts_with("skill:"))
        .collect();
    assert_eq!(skills, ["skill:graph-engineer"]);
}

#[test]
fn import_symlink_resuelve_repo() {
    if !comun::aislado("import_symlink_resuelve_repo") {
        return;
    }
    let h = HomeFalso::nuevo();
    let repo = h.repo_git(
        "Proyectos/house-rules",
        Some("https://github.com/ejemplo/house-rules"),
    );
    h.escribir("Proyectos/house-rules/HOUSE-RULES.md", "# reglas\n");
    h.enlazar(".claude/HOUSE-RULES.md", &repo.join("HOUSE-RULES.md"));
    h.escribir(".claude/RTK.md", "# rtk\n");
    h.escribir(
        ".claude/CLAUDE.md",
        "@RTK.md\n@HOUSE-RULES.md\n\n## Propio\ntexto\n",
    );
    let e = escanear_laburo(&h);
    assert_eq!(
        hallazgo(&e, "import:HOUSE-RULES.md")
            .fuente
            .clone()
            .unwrap()
            .via,
        Via::Symlink
    );
    assert_eq!(
        hallazgo(&e, "import:RTK.md").pista.as_deref(),
        Some("archivo sin origen conocido: ~/.claude/RTK.md")
    );
    assert!(hallazgo(&e, "import:RTK.md").requiere.is_empty());
}

#[test]
fn remoto_con_credenciales_no_se_guarda() {
    if !comun::aislado("remoto_con_credenciales_no_se_guarda") {
        return;
    }
    let h = HomeFalso::nuevo();
    let repo = h.repo_git(
        "Proyectos/privado",
        Some("https://usuario:ghp_SECRETO123@github.com/ejemplo/privado.git"),
    );
    h.escribir(
        "Proyectos/privado/skills/privado/SKILL.md",
        "---\nname: privado\n---\n",
    );
    h.enlazar(".claude/skills/privado", &repo.join("skills/privado"));
    let e = escanear_laburo(&h);
    assert_eq!(
        hallazgo(&e, "skill:privado").fuente.clone().unwrap().repo,
        "https://github.com/ejemplo/privado"
    );
    assert!(!format!("{e:?}").contains("SECRETO"));
}

#[test]
fn normalizar_url_saca_credenciales() {
    use rebake::escaner::normalizar_url;
    assert_eq!(
        normalizar_url("https://ghp_x@github.com/a/b.git/"),
        "https://github.com/a/b"
    );
    assert_eq!(
        normalizar_url("https://github.com/a/b"),
        "https://github.com/a/b"
    );
    assert_eq!(
        normalizar_url("git@github.com:a/b.git"),
        "git@github.com:a/b"
    );
}

#[test]
fn normalizar_url_saca_query() {
    assert_eq!(
        rebake::escaner::normalizar_url("https://github.com/a/b.git?token=x"),
        "https://github.com/a/b"
    );
}

#[test]
fn fuente_de_un_perfil_gana_a_la_pista_del_otro() {
    if !comun::aislado("fuente_de_un_perfil_gana_a_la_pista_del_otro") {
        return;
    }
    let h = HomeFalso::nuevo();
    let repo = h.repo_git(
        "Proyectos/graph-engineer",
        Some("https://github.com/ejemplo/graph-engineer"),
    );
    h.escribir(
        "Proyectos/graph-engineer/skills/graph-engineer/SKILL.md",
        "x",
    );
    h.enlazar(
        ".claude/skills/graph-engineer",
        &repo.join("skills/graph-engineer"),
    );
    h.escribir(".claude-personal/skills/graph-engineer/SKILL.md", "copia");
    h.escribir(".claude/settings.json", "{}");
    h.escribir(".claude-personal/settings.json", "{}");
    let pacman = PacmanFalso(vec![]);
    let e = escanear(
        &entorno(&h, &pacman),
        &[
            perfil("laburo", "~/.claude"),
            perfil("personal", "~/.claude-personal"),
        ],
    );
    let x = hallazgo(&e, "skill:graph-engineer");
    assert!(x.fuente.is_some());
    assert!(x.pista.is_none(), "{:?}", x.pista);
}

#[test]
fn sincronizadas_y_propias_homonimas_se_distinguen_por_destino_real() {
    if !comun::aislado("sincronizadas_y_propias_homonimas_se_distinguen_por_destino_real") {
        return;
    }
    let h = HomeFalso::nuevo();
    let synced = h.escribir(".agents/skills/synced/cuenta/pdf/SKILL.md", "sync");
    h.enlazar(".claude/skills/pdf", synced.parent().unwrap());
    h.escribir(".claude-personal/skills/pdf/SKILL.md", "propia");
    let propia = h.escribir(".agents/skills/synced-propias/otra/SKILL.md", "propia");
    h.enlazar(".claude/skills/otra", propia.parent().unwrap());
    let perfiles = vec![
        perfil("laburo", "~/.claude"),
        perfil("personal", "~/.claude-personal"),
    ];
    let e = escanear(&entorno(&h, &PacmanFalso(vec![])), &perfiles);
    let mut r = rebake::modelo::Recetario::nuevo();
    r.perfiles = perfiles;
    rebake::fusion::fusionar(&mut r, e, &|_| false);
    let pdf = r.item("skill:pdf").unwrap();
    assert_eq!(pdf.sincronizado_en, ["laburo"]);
    assert_eq!(
        pdf.perfiles_propios().cloned().collect::<Vec<_>>(),
        ["personal"]
    );
    assert!(!pdf.es_sincronizado());
    assert!(!r.item("skill:otra").unwrap().es_sincronizado());
}

#[test]
fn git_del_escaneo_ignora_configuracion_invalida_en_home_aislado() {
    if !comun::aislado("git_del_escaneo_ignora_configuracion_invalida_en_home_aislado") {
        return;
    }
    assert!(std::env::var_os("CLAUDE_CONFIG_DIR").is_none());
    assert!(std::env::var_os("GIT_CONFIG_COUNT").is_none());
    let h = HomeFalso::nuevo();
    let repo = h.repo_git("repo", Some("https://example.com/propia"));
    h.escribir("repo/SKILL.md", "propia");
    h.enlazar(".claude/skills/propia", &repo);
    let e = escanear_laburo(&h);
    assert_eq!(
        hallazgo(&e, "skill:propia").fuente.as_ref().unwrap().repo,
        "https://example.com/propia"
    );
    let env_home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
    assert!(
        std::fs::read_to_string(env_home.join("git-aislado"))
            .unwrap()
            .lines()
            .count()
            >= 3
    );
}

#[test]
fn destino_ausente_conserva_marca_y_destino_propio_verificado_la_quita() {
    if !comun::aislado("destino_ausente_conserva_marca_y_destino_propio_verificado_la_quita") {
        return;
    }
    let h = HomeFalso::nuevo();
    let raiz = h.escribir(".agents/skills/synced/cuenta/pdf/SKILL.md", "sync");
    h.enlazar(".claude/skills/pdf", raiz.parent().unwrap());
    let mut r = rebake::modelo::Recetario::nuevo();
    r.perfiles.push(perfil("laburo", "~/.claude"));
    rebake::fusion::fusionar(&mut r, escanear_laburo(&h), &|_| false);
    assert!(r.items[0].es_sincronizado());
    std::fs::remove_file(h.ruta().join(".claude/skills/pdf")).unwrap();
    rebake::fusion::fusionar(&mut r, escanear_laburo(&h), &|_| false);
    assert!(r.items[0].es_sincronizado());
    h.escribir(".claude/skills/pdf/SKILL.md", "ahora propia");
    rebake::fusion::fusionar(&mut r, escanear_laburo(&h), &|_| false);
    assert!(r.items[0].sincronizado_en.is_empty());
    assert_eq!(
        r.items[0].perfiles_propios().cloned().collect::<Vec<_>>(),
        ["laburo"]
    );
}

#[test]
fn skills_leen_descripcion_completa_en_los_cuatro_formatos() {
    let h = HomeFalso::nuevo();
    for (nombre, valor, esperado) in [
        (
            "plana",
            "Author text, untouched.",
            "Author text, untouched.",
        ),
        (
            "dobles",
            "\"Author says: \\\"hello\\\".\"",
            "Author says: \"hello\".",
        ),
        (
            "simples",
            "'Author''s original text.'",
            "Author's original text.",
        ),
        (
            "plegado",
            ">\n  First line.\n  Second line.",
            "First line. Second line.",
        ),
        (
            "literal",
            "|\n  First line.\n  Second line.",
            "First line. Second line.",
        ),
    ] {
        h.escribir(&format!(".claude/skills/{nombre}/SKILL.md"), &format!("---\nname: {nombre}\ndescription: {valor}\nother: ignored\n---\nBody description: ignored"));
        let e = escanear_laburo(&h);
        assert_eq!(
            hallazgo(&e, &format!("skill:{nombre}"))
                .descripcion
                .as_deref(),
            Some(esperado)
        );
    }
    for (nombre, texto) in [
        ("falta", "---\nname: falta\n---\ndescription: body"),
        ("rota", "---\ndescription: \"unclosed\n---"),
        ("anidada", "---\nmetadata:\n  description: nested\n---"),
        ("sin_cierre", "---\ndescription: no closing delimiter"),
        ("objeto", "---\ndescription: {invalid: object}\n---"),
    ] {
        h.escribir(&format!(".claude/skills/{nombre}/SKILL.md"), texto);
        let e = escanear_laburo(&h);
        assert_eq!(hallazgo(&e, &format!("skill:{nombre}")).descripcion, None);
        assert!(e.avisos.is_empty(), "{:?}", e.avisos);
    }
    h.escribir(".claude/skills/ilegible/SKILL.md/hijo", "directorio");
    let e = escanear_laburo(&h);
    assert_eq!(hallazgo(&e, "skill:ilegible").descripcion, None);
    assert_eq!(e.avisos.len(), 1);
}

#[test]
fn escaneo_sin_path_trae_descripcion_y_agrupa_la_primera() {
    let h = HomeFalso::nuevo();
    for (dir, texto) in [
        (".claude", "First author's text"),
        (".claude-personal", "Second author's text"),
    ] {
        h.escribir(&format!("{dir}/settings.json"), "{}");
        h.escribir(
            &format!("{dir}/skills/demo/SKILL.md"),
            &format!("---\ndescription: {texto}\n---"),
        );
    }
    let pacman = PacmanFalso(vec![]);
    let mut entorno = entorno(&h, &pacman);
    entorno.path.clear();
    let e = escanear(
        &entorno,
        &[
            perfil("laburo", "~/.claude"),
            perfil("personal", "~/.claude-personal"),
        ],
    );
    assert_eq!(
        hallazgo(&e, "skill:demo").descripcion.as_deref(),
        Some("First author's text")
    );
    assert_eq!(hallazgo(&e, "skill:demo").perfiles.len(), 2);
    assert!(e.avisos.is_empty(), "{:?}", e.avisos);
}

#[test]
fn descripcion_oculta_token_y_home_antes_de_guardar() {
    let h = HomeFalso::nuevo();
    let token = "sk-ant-abcdefghijklmnopqrstuvwxyz0123456789";
    h.escribir(
        ".claude/skills/privada/SKILL.md",
        &format!(
            "---\ndescription: TOKEN={token} path={}/private/file\n---",
            h.ruta().display()
        ),
    );
    let mut r = rebake::modelo::Recetario::nuevo();
    r.perfiles.push(perfil("laburo", "~/.claude"));
    rebake::fusion::fusionar(&mut r, escanear_laburo(&h), &|_| false);
    let ruta = h.ruta().join("cookbook.toml");
    rebake::archivo::guardar(&ruta, &r).unwrap();
    let texto = std::fs::read_to_string(ruta).unwrap();
    assert!(!texto.contains(token), "token expuesto");
    assert!(!texto.contains(h.ruta().to_str().unwrap()), "home expuesto");
    assert!(texto.contains("~/private/file"));
    assert!(texto.contains(rebake::modelo::OCULTO));
}
