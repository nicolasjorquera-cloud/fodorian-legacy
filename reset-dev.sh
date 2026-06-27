#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
  cat <<'EOF'
Uso:
  ./reset-dev.sh           # Arranque normal (rapido)
  ./reset-dev.sh --clean   # Limpia Rust cache y arranca

Tip:
  Usa --clean solo cuando Tauri falle por cache/rutas viejas.
EOF
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

cd "$ROOT_DIR"

if [[ "${1:-}" == "--clean" ]]; then
  echo "[reset-dev] Limpiando cache de Rust..."
  (cd src-tauri && cargo clean)
fi

echo "[reset-dev] Iniciando Tauri dev..."
exec npm run tauri dev
