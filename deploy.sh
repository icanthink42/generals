#!/usr/bin/env bash
set -euo pipefail

# Resolve repository root
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$REPO_ROOT"

# Ensure cargo bin path available for wasm-bindgen
export PATH="$HOME/.cargo/bin:$PATH"

# Target server
TARGET_SERVER="ec2-user@54.162.216.127"
REMOTE_DIR="/home/ec2-user/generals"

echo "==> Installing cross if not already installed..."
cargo install cross --git https://github.com/cross-rs/cross

echo "==> Building server binary in release mode for Linux..."
# Build for Linux using cross
cross build --release --target x86_64-unknown-linux-gnu

echo "==> Creating deployment package..."
# Create a temporary directory for deployment
DEPLOY_TMP="$(mktemp -d)"
mkdir -p "${DEPLOY_TMP}/www"

echo "==> Building client with wasm-pack in release mode..."
wasm-pack build --release --target web --out-dir www/pkg

# Create a script to start the client server
cat > "${DEPLOY_TMP}/start_client.sh" << 'EOSCRIPT'
#!/usr/bin/env bash
set -euo pipefail

WWW_DIR="www"
PORT="${PORT:-80}"

cd "$(dirname "$0")"

# Create index.html with environment variable
cat > "${WWW_DIR}/index.html" << 'EOHTML'
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Generals Game</title>
    <style>
        html, body {
            margin: 0;
            padding: 0;
            width: 100%;
            height: 100%;
            overflow: hidden;
            background-color: #1a1a1a;
        }
        canvas {
            display: block;
        }
    </style>
    <script>
        window.SERVER_URL = "ws://54.162.216.127:1812/ws";
    </script>
</head>
<body>
    <canvas id="canvas"></canvas>
    <script type="module">
        import init from './pkg/generals.js';
        init();
    </script>
</body>
</html>
EOHTML

echo "==> Serving ${WWW_DIR} at http://0.0.0.0:${PORT}"
sudo python3 -m http.server "${PORT}" --bind 0.0.0.0 --directory "${WWW_DIR}"
EOSCRIPT

chmod +x "${DEPLOY_TMP}/start_client.sh"

# Copy necessary files
cp target/x86_64-unknown-linux-gnu/release/server "${DEPLOY_TMP}/"
cp -r www/* "${DEPLOY_TMP}/www/"

echo "==> Deploying to ${TARGET_SERVER}..."
# Ensure remote directory exists
ssh "${TARGET_SERVER}" "mkdir -p ${REMOTE_DIR}"

# Check if config.toml exists on server
if ! ssh "${TARGET_SERVER}" "test -f ${REMOTE_DIR}/config.toml"; then
    echo "==> No config.toml found on server, copying local version..."
    cp config.toml "${DEPLOY_TMP}/"
else
    echo "==> Existing config.toml found on server, preserving it..."
fi

# Copy files to server
scp -r "${DEPLOY_TMP}"/* "${TARGET_SERVER}:${REMOTE_DIR}"

# Cleanup
rm -rf "${DEPLOY_TMP}"

echo "==> Deployment complete!"
echo "To start the server:"
echo "  ${REMOTE_DIR}/server"
echo
echo "To start the client web server (on port 8080):"
echo "  ${REMOTE_DIR}/start_client.sh"
echo
echo "Then visit http://$(hostname -f):8080 in your browser"