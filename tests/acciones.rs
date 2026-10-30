use rebake::acciones::*;
use rebake::modelo::*;

fn recetario_con(estado: Estado) -> Recetario {
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil {
        nombre: "claude".into(),
        dir: "~/.claude".into(),
    });
    let mut item = Item::nuevo("statusline:statusline.sh", Tipo::Statusline);
    item.perfiles = vec!["claude".into()];
    item.estado = estado;
    r.items.push(item);
    r.checklist.ajustes.push(Ajuste {
        perfil: "claude".into(),
        clave: "theme".into(),
        valor: "dark".into(),
    });
    r
}

const ID: &str = "statusline:statusline.sh";

#[test]
fn un_hook_vinculado_no_se_aprueba_ni_alterna() {
    for estado in [Estado::Pendiente, Estado::PorRevisar, Estado::Aprobada] {
        let mut r = Recetario::nuevo();
        let mut hook = Item::nuevo("hook:tool", Tipo::Hook);
        hook.estado = estado;
        hook.instalado_por = Some(InstaladorHook::Externo("tool".into()));
        r.items.push(hook);
        assert!(aprobar(&mut r, "hook:tool").is_err());
        assert!(alternar_aprobacion(&mut r, "hook:tool").is_err());
        assert_eq!(r.items[0].estado, estado);
    }
}

#[test]
fn aprobar_pendiente_se_rechaza() {
    let mut r = recetario_con(Estado::Pendiente);
    assert!(aprobar(&mut r, ID).is_err());
    assert_eq!(r.item(ID).unwrap().estado, Estado::Pendiente);
    let mut r = recetario_con(Estado::PorRevisar);
    aprobar(&mut r, ID).unwrap();
    assert_eq!(r.item(ID).unwrap().estado, Estado::Aprobada);
}

#[test]
fn excluir_exige_motivo() {
    let mut r = recetario_con(Estado::Pendiente);
    assert!(excluir(&mut r, ID, "   ").is_err());
    excluir(&mut r, ID, "no me interesa").unwrap();
    let x = r.item(ID).unwrap();
    assert_eq!(x.estado, Estado::Excluida);
    assert_eq!(x.motivo.as_deref(), Some("no me interesa"));
}

#[test]
fn pegar_link_deja_fuente_manual() {
    let mut r = recetario_con(Estado::Pendiente);
    assert!(pegar_link(&mut r, ID, "no-es-una-url").is_err());
    pegar_link(&mut r, ID, "https://github.com/nilbuild/claude-statusline/").unwrap();
    let f = r.item(ID).unwrap().fuente.clone().unwrap();
    assert_eq!(f.repo, "https://github.com/nilbuild/claude-statusline");
    assert_eq!(f.via, Via::Manual);
    assert_eq!(r.item(ID).unwrap().estado, Estado::Pendiente);
}

#[test]
fn edicion_invalida_no_cambia() {
    let mut r = recetario_con(Estado::Pendiente);
    let antes = r.clone();
    let texto = item_como_toml(r.item(ID).unwrap()).unwrap() + "color = \"rojo\"\n";
    assert!(reemplazar_desde_toml(&mut r, ID, &texto).is_err());
    let otro_id = item_como_toml(r.item(ID).unwrap())
        .unwrap()
        .replace(ID, "statusline:otra.sh");
    assert!(reemplazar_desde_toml(&mut r, ID, &otro_id).is_err());
    assert_eq!(r, antes);
}

#[test]
fn edicion_valida_queda_por_revisar() {
    let mut r = recetario_con(Estado::Pendiente);
    let texto = item_como_toml(r.item(ID).unwrap())
        .unwrap()
        .replace("estado = \"pendiente\"", "estado = \"aprobada\"")
        + "\n[[paso]]\ncmd = \"npx @kamranahmedse/claude-statusline\"\nmodo = \"auto\"\npor_perfil = false\n";
    reemplazar_desde_toml(&mut r, ID, &texto).unwrap();
    let x = r.item(ID).unwrap();
    assert_eq!(x.estado, Estado::PorRevisar);
    assert_eq!(x.pasos.len(), 1);
}

#[test]
fn renombrar_perfil_actualiza_items() {
    let mut r = recetario_con(Estado::Pendiente);
    renombrar_perfil(&mut r, "claude", "laburo").unwrap();
    assert_eq!(r.perfiles[0].nombre, "laburo");
    assert_eq!(r.item(ID).unwrap().perfiles, ["laburo"]);
    assert_eq!(r.checklist.ajustes[0].perfil, "laburo");
    r.perfiles.push(Perfil {
        nombre: "personal".into(),
        dir: "~/.claude-personal".into(),
    });
    assert!(renombrar_perfil(&mut r, "laburo", "personal").is_err());
}

#[test]
fn alternar_aprobacion_va_y_vuelve() {
    let mut r = recetario_con(Estado::PorRevisar);
    assert_eq!(alternar_aprobacion(&mut r, ID).unwrap(), Estado::Aprobada);
    assert_eq!(alternar_aprobacion(&mut r, ID).unwrap(), Estado::PorRevisar);
    assert_eq!(r.item(ID).unwrap().estado, Estado::PorRevisar);
    for estado in [Estado::Pendiente, Estado::Excluida] {
        let mut r = recetario_con(estado);
        assert!(alternar_aprobacion(&mut r, ID).is_err());
        assert_eq!(r.item(ID).unwrap().estado, estado);
    }
}

#[test]
fn incluir_vuelve_segun_tenga_pasos_y_borra_el_motivo() {
    let mut r = recetario_con(Estado::Pendiente);
    excluir(&mut r, ID, "no lo uso").unwrap();
    assert_eq!(incluir(&mut r, ID).unwrap(), Estado::Pendiente);
    assert_eq!(r.item(ID).unwrap().motivo, None);

    let mut r = recetario_con(Estado::PorRevisar);
    r.item_mut(ID).unwrap().pasos = vec![Paso {
        cmd: "echo hola".into(),
        modo: Modo::Auto,
        por_perfil: false,
        cita: None,
        nota: None,
    }];
    excluir(&mut r, ID, "duplicado").unwrap();
    assert_eq!(incluir(&mut r, ID).unwrap(), Estado::PorRevisar);
    assert!(
        incluir(&mut r, ID).is_err(),
        "incluir algo no excluido es un error"
    );
}

fn con_posible(repo: Option<&str>) -> Recetario {
    let mut r = recetario_con(Estado::Pendiente);
    r.item_mut(ID).unwrap().posible = Some(Posible {
        repo: repo.map(String::from),
        motivo: "el README lo nombra".into(),
        descartado: false,
    });
    r
}

#[test]
fn aceptar_el_posible_lo_vuelve_origen_buscado() {
    let mut r = con_posible(Some("https://github.com/ejemplo/statusline"));
    aceptar_posible(&mut r, ID).unwrap();
    let x = r.item(ID).unwrap();
    assert!(x.posible.is_none());
    assert_eq!(
        x.fuente,
        Some(Fuente {
            repo: "https://github.com/ejemplo/statusline".into(),
            via: Via::Busqueda,
            doc: None,
        })
    );
    assert_eq!(x.estado, Estado::Pendiente);
}

#[test]
fn descartar_lo_anota_y_ya_no_se_puede_aceptar() {
    let mut r = con_posible(Some("https://github.com/ejemplo/statusline"));
    descartar_posible(&mut r, ID).unwrap();
    let x = r.item(ID).unwrap();
    assert!(x.fuente.is_none());
    assert!(x.posible.as_ref().unwrap().descartado);
    assert!(aceptar_posible(&mut r, ID).is_err());
    assert!(r.item(ID).unwrap().fuente.is_none());
}

#[test]
fn sin_repo_encontrado_no_hay_nada_que_aceptar() {
    let mut r = con_posible(None);
    assert!(aceptar_posible(&mut r, ID).is_err());
    assert!(descartar_posible(&mut r, ID).is_err());
}

#[test]
fn pegar_el_link_reemplaza_al_posible() {
    let mut r = con_posible(Some("https://github.com/ejemplo/statusline"));
    pegar_link(&mut r, ID, "https://github.com/otro/repo").unwrap();
    let x = r.item(ID).unwrap();
    assert!(x.posible.is_none());
    assert_eq!(x.fuente.as_ref().unwrap().via, Via::Manual);
}

#[test]
fn renombrar_perfil_conserva_su_clasificacion_sincronizada() {
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil {
        nombre: "antes".into(),
        dir: "~/.claude".into(),
    });
    let mut item = Item::nuevo("skill:pdf", Tipo::Skill);
    item.perfiles = vec!["antes".into()];
    item.sincronizado_en = vec!["antes".into()];
    r.items.push(item);
    rebake::acciones::renombrar_perfil(&mut r, "antes", "despues").unwrap();
    assert_eq!(r.items[0].sincronizado_en, ["despues"]);
    assert!(r.items[0].es_sincronizado());
}
