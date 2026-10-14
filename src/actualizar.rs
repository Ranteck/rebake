use anyhow::{Context, Result, bail};
use std::os::unix::process::CommandExt;
use std::process::Command;

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
        .args(["-fsSL", "-o", "/dev/null", "--max-time", "2"])
        .args(["-w", "%{url_effective}"])
        .arg(format!("{base}/latest"))
        .output()
        .ok()?;
    if !salida.status.success() {
        return None;
    }
    let url = String::from_utf8(salida.stdout).ok()?;
    si_es_nueva(propia, url.trim_end_matches('/').rsplit('/').next()?)
}

/// Corre el `install.sh` de la versión `version` sobre el binario en uso y lo vuelve a lanzar
/// con los mismos argumentos. Solo vuelve si algo falló.
pub fn actualizar(version: &str, base: &str) -> Result<()> {
    let exe = std::env::current_exe().context("no pude ver cuál es el binario en uso")?;
    // install.sh escribe siempre <carpeta>/rebake: con otro nombre instalaría al lado.
    if exe.file_name().is_none_or(|n| n != "rebake") {
        bail!(
            "{} no se llama `rebake`: actualizá a mano con el install.sh",
            exe.display()
        );
    }
    let dir = exe.parent().context("el binario no tiene carpeta")?;
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
    let error = Command::new(&exe).args(std::env::args_os().skip(1)).exec();
    Err(error).with_context(|| format!("no pude abrir {}", exe.display()))
}
