#!/usr/bin/env bash
# Regenerate bootstrap JSON artifacts from IRIS source.
# Delegates to: iris-stage0 rebuild
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
exec "$SCRIPT_DIR/iris-stage0" rebuild "$(cd "$SCRIPT_DIR/.." && pwd)"
