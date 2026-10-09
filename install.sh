#!/bin/sh
# Instala o actualiza rebake desde GitHub Releases. Se puede correr las veces que haga falta.

# Todo corre dentro de main: si la red corta la descarga del script, no se ejecuta la mitad.
main() {
  set -eu
  repo="Ranteck/rebake"
  asset="rebake-x86_64-unknown-linux-musl.tar.gz"
  version="${REBAKE_VERSION:-latest}"
  dir="${REBAKE_INSTALL_DIR:-$HOME/.local/bin}"
  base="${REBAKE_BASE_URL:-https://github.com/$repo/releases}"

  if [ "$(uname -s)" != Linux ] || [ "$(uname -m)" != x86_64 ]; then
    fail "por ahora solo hay binarios para Linux x86_64; con Rust instalado: cargo install --git https://github.com/$repo"
  fi
  if [ "$version" = latest ]; then
    url="$base/latest/download"
  else
    url="$base/download/$version"
  fi

  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT
  curl -fsSL "$url/$asset" -o "$tmp/$asset" || fail "no pude bajar $url/$asset"
  curl -fsSL "$url/$asset.sha256" -o "$tmp/$asset.sha256" || fail "no pude bajar el checksum de $asset"
  # Sin checksum válido no se instala nada: protege de descargas cortadas o alteradas.
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$tmp" && sha256sum -c --status "$asset.sha256") || fail "el checksum no coincide; no se instaló nada"
  else
    (cd "$tmp" && shasum -a 256 -c -s "$asset.sha256") || fail "el checksum no coincide; no se instaló nada"
  fi
  tar -xzf "$tmp/$asset" -C "$tmp" rebake

  mkdir -p "$dir"
  # Copiar al lado del destino y renombrar: nunca queda un binario a medio copiar.
  cp "$tmp/rebake" "$dir/.rebake.tmp"
  chmod 755 "$dir/.rebake.tmp"
  mv "$dir/.rebake.tmp" "$dir/rebake"

  "$dir/rebake" --version
  case ":$PATH:" in
    *":$dir:"*) ;;
    *) echo "rebake: agregá $dir a tu PATH para usarlo como 'rebake'." ;;
  esac
}

fail() {
  echo "rebake: $*" >&2
  exit 1
}

main "$@"
