#!/usr/bin/env bash
set -euo pipefail

# Resolve repository root
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$REPO_ROOT"

# Ensure cargo bin path available for wasm-bindgen
export PATH="$HOME/.cargo/bin:$PATH"

WWW_DIR="www"
PORT="${PORT:-8080}"
URL="http://127.0.0.1:${PORT}"

echo "==> Building with wasm-pack..."
wasm-pack build --target web --out-dir "${WWW_DIR}/pkg"

echo "==> Preparing web assets..."
mkdir -p "${WWW_DIR}"

echo "==> Serving ${WWW_DIR} at ${URL}"

# Try to open the browser on macOS
if command -v open >/dev/null 2>&1; then
  (sleep 1 && open "${URL}") >/dev/null 2>&1 &
fi

# Prefer Python 3 simple HTTP server; fall back to Python 2 if present;
# finally fall back to the native Rust server if neither is available.
if command -v python3 >/dev/null 2>&1; then
  (cd "${WWW_DIR}" && exec python3 -m http.server "${PORT}")
elif command -v python >/dev/null 2>&1; then
  (cd "${WWW_DIR}" && exec python -m SimpleHTTPServer "${PORT}")
else
  echo "python3/python not found; falling back to native Rust server..."
  echo "NOTE: Native server will serve '${WWW_DIR}' at ${URL}."
  exec cargo run --bin "${BIN_NAME}"
fi

