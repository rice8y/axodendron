#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
manifest="$repository_dir/wasm-plugin/Cargo.toml"
cargo_bin=$(command -v cargo)
cargo_home=$(CDPATH= cd -- "$(dirname -- "$cargo_bin")/.." && pwd)
target_dir=${AXODENDRON_TARGET_DIR:-"$repository_dir/target"}

remap_flags="--remap-path-prefix=$repository_dir=/workspace --remap-path-prefix=$cargo_home=/cargo"
if [ -n "${RUSTFLAGS:-}" ]; then
  remap_flags="$RUSTFLAGS $remap_flags"
fi

cd "$repository_dir"
CARGO_INCREMENTAL=0 \
CARGO_TARGET_DIR="$target_dir" \
SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-1} \
RUSTFLAGS="$remap_flags" \
cargo build --manifest-path "$manifest" --locked --release --target wasm32-unknown-unknown \
  -p axodendron-typst-plugin

artifact="$target_dir/wasm32-unknown-unknown/release/axodendron_typst_plugin.wasm"
cp "$artifact" "$repository_dir/package/plugin.wasm"
