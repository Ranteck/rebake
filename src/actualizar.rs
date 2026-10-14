use anyhow::{Context, Result, bail};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;

const BASE_POR_DEFECTO: &str = "https://github.com/Ranteck/rebake/releases";

/// Misma variable que usa `install.sh`.
pub fn base() -> String {
    std::env::var("REBAKE_BASE_URL").unwrap_or_else(|_| BASE_POR_DEFECTO.to_string())
}

/// "X.Y.Z" con o sin "v" inicial; cualquier otra cosa no es una versión.
pub fn parsear(version: &str) -> Option<(u64, u64, u64)> {
    let mut partes = version.strip_prefix('v').unwrap_or(version).split('.');
    let leida = (
        partes.next()?.parse().ok()?,
        partes.next()?.parse().ok()?,
        partes.next()?.parse().ok()?,
    );
    partes.next().is_none().then_some(leida)
}

/// La publicada (sin "v") solo si es estrictamente más nueva que la propia.
pub fn si_es_nueva(propia: &str, publicada: &str) -> Option<String> {
    (parsear(publicada)? > parsear(propia)?)
        .then(|| publicada.strip_prefix('v').unwrap_or(publicada).to_string())
}

/// Versión publicada más nueva que la propia, o `None`. Ningún fallo (sin curl, sin red,
/// más de 2 s, una URL rara) se avisa: revisar nunca debe frenar el uso.
pub fn version_nueva(propia: &str, base: &str) -> Option<String> {
    let salida = Command::new("curl")
        // Sin seguir la redirección a `…/tag/vX.Y.Z`: bajar esa página casi duplica la demora.
        .args(["-fsS", "-o", "/dev/null", "--max-time", "2"])
        .args(["-w", "%{redirect_url}"])
        .arg(format!("{base}/latest"))
        .output()
        .ok()?;
    if !salida.status.success() {
        return None;
    }
    let url = String::from_utf8(salida.stdout).ok()?;
    si_es_nueva(propia, url.trim_end_matches('/').rsplit('/').next()?)
}

/// Lanza la revisión en un hilo y no espera: quien la lea con `try_recv` ve el resultado solo
/// si ya llegó.
pub fn en_segundo_plano(propia: String, base: String) -> Receiver<Option<String>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        // Falla solo si ya nadie espera la respuesta (el programa terminó o no la quiso).
        let _ = tx.send(version_nueva(&propia, &base));
    });
    rx
}

/// Corre el `install.sh` de la versión `version` sobre la carpeta de `binario`.
pub fn instalar(version: &str, base: &str, binario: &Path) -> Result<()> {
    // install.sh escribe siempre <carpeta>/rebake: con otro nombre instalaría al lado.
    if binario.file_name().is_none_or(|n| n != "rebake") {
        bail!(
            "{} no se llama `rebake`: actualizá a mano con el install.sh",
            binario.display()
        );
    }
    let dir = binario.parent().context("el binario no tiene carpeta")?;
    let tag = format!("v{version}");
    let url = format!("{base}/download/{tag}/install.sh");
    let script = Command::new("curl")
        .args(["-fsSL", "--max-time", "30", &url])
        .output()
        .context("no pude correr curl")?;
    if !script.status.success() {
        bail!(
            "no pude bajar {url}: {}",
            String::from_utf8_lossy(&script.stderr).trim()
        );
    }
    let instalacion = Command::new("sh")
        .arg("-c")
        .arg(String::from_utf8_lossy(&script.stdout).as_ref())
        .env("REBAKE_VERSION", &tag)
        .env("REBAKE_INSTALL_DIR", dir)
        .env("REBAKE_BASE_URL", base)
        .output()
        .context("no pude correr sh")?;
    if !instalacion.status.success() {
        bail!("{}", String::from_utf8_lossy(&instalacion.stderr).trim());
    }
    Ok(())
}

/// Lanza `binario` con los mismos argumentos que el proceso actual. Solo vuelve si falló.
pub fn reabrir(binario: &Path) -> anyhow::Error {
    let error = Command::new(binario)
        .args(std::env::args_os().skip(1))
        .exec();
    anyhow::Error::new(error).context(format!("no pude abrir {}", binario.display()))
}
