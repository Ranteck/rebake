use recetario::acciones::*;
use recetario::modelo::*;

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
