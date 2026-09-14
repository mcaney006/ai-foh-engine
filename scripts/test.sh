#!/usr/bin/env bash
# Checkout may live on a FUSE/noexec volume. Build artifacts go to a native fs.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/ai-foh-target}"
cd "$ROOT"
cargo test --workspace "$@"
cargo run -p foh-engine --quiet
cmake -S cpp/foh_dsp -B /tmp/foh_dsp_build >/dev/null
cmake --build /tmp/foh_dsp_build
/tmp/foh_dsp_build/foh_dsp_selftest
