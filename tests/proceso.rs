use recetario::proceso::ejecutar;
use std::process::Command;
use std::time::{Duration, Instant};

fn sh(script: &str) -> Command {
    let mut c = Command::new("sh");
    c.arg("-c").arg(script);
    c
}

#[test]
fn stdin_cerrado_no_cuelga() {
    let inicio = Instant::now();
    let s = ejecutar(&mut sh("if read x; then echo leyo; else echo eof; fi"), Duration::from_secs(5), &mut |_| {}).unwrap();
    assert!(s.exito());
    assert_eq!(s.texto.trim(), "eof");
    assert!(inicio.elapsed() < Duration::from_secs(4));
}

#[test]
fn tope_mata_al_grupo() {
    let inicio = Instant::now();
    let s = ejecutar(&mut sh("sleep 30 & sleep 30"), Duration::from_millis(300), &mut |_| {}).unwrap();
    assert!(s.vencido);
    assert!(!s.exito());
    assert!(inicio.elapsed() < Duration::from_secs(5));
}

#[test]
fn hijo_en_segundo_plano_no_cuelga() {
    let inicio = Instant::now();
    let s = ejecutar(&mut sh("sleep 30 & echo listo"), Duration::from_secs(10), &mut |_| {}).unwrap();
    assert!(s.exito());
    assert!(s.texto.contains("listo"));
    assert!(inicio.elapsed() < Duration::from_secs(5));
}

#[test]
fn pasa_variables_de_entorno() {
    let mut cmd = sh("echo \"$CLAUDE_CONFIG_DIR\"; echo error >&2; exit 3");
    cmd.env("CLAUDE_CONFIG_DIR", "/tmp/perfil-x");
    let mut lineas = Vec::new();
    let s = ejecutar(&mut cmd, Duration::from_secs(5), &mut |l| lineas.push(l.to_string())).unwrap();
    assert_eq!(s.codigo, Some(3));
    assert!(lineas.contains(&"/tmp/perfil-x".to_string()));
    assert!(lineas.contains(&"error".to_string()));
}

#[test]
fn ejecutable_ocupado_un_instante_se_reintenta() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("script");
    std::fs::write(&script, "#!/bin/sh\necho corrio\n").unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    // Un descriptor de escritura abierto hace que exec falle con ETXTBSY hasta que se cierra.
    let ocupado = std::fs::OpenOptions::new().write(true).open(&script).unwrap();
    let liberar = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(60));
        drop(ocupado);
    });
    let s = ejecutar(&mut Command::new(&script), Duration::from_secs(5), &mut |_| {}).unwrap();
    liberar.join().unwrap();
    assert!(s.exito());
    assert_eq!(s.texto.trim(), "corrio");
}
