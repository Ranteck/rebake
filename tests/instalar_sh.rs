use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ASSET: &str = "rebake-x86_64-unknown-linux-musl.tar.gz";

/// Arma un release falso en `<base>/<ruta>/` con un `rebake` que imprime `version`.
fn publicar(base: &Path, ruta: &str, version: &str) -> PathBuf {
    let dir = base.join(ruta);
    fs::create_dir_all(&dir).unwrap();
    let staging = tempfile::tempdir().unwrap();
    let bin = staging.path().join("rebake");
    fs::write(&bin, format!("#!/bin/sh\necho \"rebake {version}\"\n")).unwrap();
    fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(staging.path().join("LICENSE"), "MIT\n").unwrap();
    let empaquetado = Command::new("tar")
        .arg("-czf")
        .arg(dir.join(ASSET))
        .arg("-C")
        .arg(staging.path())
        .args(["rebake", "LICENSE"])
        .status()
        .unwrap();
    assert!(empaquetado.success());
    let suma = Command::new("sha256sum")
        .arg(ASSET)
        .current_dir(&dir)
        .output()
        .unwrap();
    fs::write(dir.join(format!("{ASSET}.sha256")), suma.stdout).unwrap();
    dir
}

fn instalar(base: &Path, destino: &Path, version: Option<&str>) -> Output {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("install.sh");
    let mut c = Command::new("sh");
    c.arg(script)
        .env("REBAKE_BASE_URL", format!("file://{}", base.display()))
        .env("REBAKE_INSTALL_DIR", destino)
        .env_remove("REBAKE_VERSION");
    if let Some(v) = version {
        c.env("REBAKE_VERSION", v);
    }
    c.output().unwrap()
}

fn texto(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

#[test]
fn instala_el_binario_verificado() {
    let base = tempfile::tempdir().unwrap();
    let destino = tempfile::tempdir().unwrap();
    publicar(base.path(), "latest/download", "1.0.0");
    let o = instalar(base.path(), destino.path(), None);
    assert!(o.status.success(), "{}", texto(&o));
    let bin = destino.path().join("rebake");
    assert!(fs::metadata(&bin).unwrap().permissions().mode() & 0o111 != 0);
    assert!(texto(&o).contains("rebake 1.0.0"), "{}", texto(&o));
}

#[test]
fn checksum_incorrecto_no_instala() {
    let base = tempfile::tempdir().unwrap();
    let destino = tempfile::tempdir().unwrap();
    let dir = publicar(base.path(), "latest/download", "1.0.0");
    fs::write(
        dir.join(format!("{ASSET}.sha256")),
        format!("{}  {ASSET}\n", "0".repeat(64)),
    )
    .unwrap();
    let o = instalar(base.path(), destino.path(), None);
    assert!(!o.status.success());
    assert!(!destino.path().join("rebake").exists());
    assert!(texto(&o).contains("checksum"), "{}", texto(&o));
}

#[test]
fn version_fija_usa_su_release() {
    let base = tempfile::tempdir().unwrap();
    let destino = tempfile::tempdir().unwrap();
    publicar(base.path(), "download/v9.9.9", "9.9.9");
    let o = instalar(base.path(), destino.path(), Some("v9.9.9"));
    assert!(o.status.success(), "{}", texto(&o));
    assert!(texto(&o).contains("rebake 9.9.9"), "{}", texto(&o));
}

#[test]
fn reinstalar_reemplaza_el_binario() {
    let base = tempfile::tempdir().unwrap();
    let destino = tempfile::tempdir().unwrap();
    publicar(base.path(), "latest/download", "1.0.0");
    assert!(instalar(base.path(), destino.path(), None).status.success());
    publicar(base.path(), "latest/download", "2.0.0");
    let o = instalar(base.path(), destino.path(), None);
    assert!(o.status.success(), "{}", texto(&o));
    let version = Command::new(destino.path().join("rebake"))
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&version.stdout).trim(),
        "rebake 2.0.0"
    );
}
