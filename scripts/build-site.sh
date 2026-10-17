#!/usr/bin/env bash
# Build the landing site with its live demo into target/site (deployed by pages.yml):
#   site/*                          static pages
#   pkg/                            the WebAssembly package (wasm-pack --target web)
#   ephemeris/de440s-1920-2070.bsp  a DE440s excerpt for the demo (~16 MB, covers 150 years)
# Needs wasm-pack and the wasm32-unknown-unknown target. Preview locally with
#   python3 -m http.server -d target/site
set -euo pipefail
cd "$(dirname "$0")/.."
OUT="$PWD/target/site"
# 1920-01-01 to 2070-01-01, TDB Julian dates. Keep in sync with site/demo.js.
START_JD=2422320.5
END_JD=2477103.5

rm -rf "$OUT"
mkdir -p "$OUT/ephemeris"
cp -R site/. "$OUT/"

scripts/fetch-kernels.sh >/dev/null
cargo run --release --quiet -p astroceleste-engine --example excerpt -- \
  kernels/de440s.bsp "$OUT/ephemeris/de440s-1920-2070.bsp" "$START_JD" "$END_JD"

wasm-pack build --release --target web --out-dir "$OUT/pkg" crates/astroceleste-engine-wasm
# Also sync to crates/astroceleste-engine-wasm/pkg for local tests and adapter consumers
mkdir -p crates/astroceleste-engine-wasm/pkg
cp "$OUT/pkg"/astroceleste_engine_wasm_bg.wasm crates/astroceleste-engine-wasm/pkg/
cp "$OUT/pkg"/astroceleste_engine_wasm.js crates/astroceleste-engine-wasm/pkg/
cp "$OUT/pkg"/astroceleste_engine_wasm.d.ts crates/astroceleste-engine-wasm/pkg/
cp "$OUT/pkg"/astroceleste_engine_wasm_bg.wasm.d.ts crates/astroceleste-engine-wasm/pkg/
# Only the module and its bindings are served.
rm -f "$OUT/pkg/.gitignore" "$OUT/pkg/package.json" "$OUT/pkg/README.md" \
  "$OUT/pkg"/LICENSE-* "$OUT/pkg/NOTICE"

du -h "$OUT"/ephemeris/*.bsp "$OUT"/pkg/*.wasm
du -sh "$OUT"
