#!/bin/sh
# Install dotai onto your PATH.
#
# Usage:
#   ./scripts/install.sh
# Custom directory (default ~/.local/bin):
#   DOTAI_INSTALL_DIR=/path ./scripts/install.sh
#
# Tip: on any machine with cargo, `cargo install --path .` is simpler —
# it installs to ~/.cargo/bin, which rustup already puts on your PATH.
# For a custom location: `cargo install --path . --root /somewhere` (-> /somewhere/bin).

set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
DEST_DIR=${DOTAI_INSTALL_DIR:-"$HOME/.local/bin"}

cargo build --release --manifest-path "$ROOT/Cargo.toml"
mkdir -p "$DEST_DIR"
install -m 0755 "$ROOT/target/release/dotai" "$DEST_DIR/dotai"
echo "Installed dotai to $DEST_DIR/dotai"

case ":$PATH:" in
    *":$DEST_DIR:"*) echo "dotai is on your PATH. Run \`dotai --help\`." ;;
    *) echo "Add $DEST_DIR to your PATH:  export PATH=\"$DEST_DIR:\$PATH\"" ;;
esac
