#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
manifest="$repository_dir/wasm-plugin/Cargo.toml"
cargo_bin=$(command -v cargo)
cargo_home=$(CDPATH= cd -- "$(dirname -- "$cargo_bin")/.." && pwd)
target_dir=${AXODENDRON_TARGET_DIR:-"$repository_dir/target"}
expected_rust=$(sed -n 's/^channel = "\([^"]*\)"$/\1/p' "$repository_dir/rust-toolchain.toml")
actual_rust=$(rustc --version | awk '{ print $2 }')
expected_binaryen=132
wasm_opt=$(command -v wasm-opt || true)

if [ -z "$expected_rust" ] || [ "$actual_rust" != "$expected_rust" ]; then
  printf 'plugin.wasm requires Rust %s, found %s\n' "$expected_rust" "$actual_rust" >&2
  exit 1
fi

if [ -z "$wasm_opt" ]; then
  printf 'plugin.wasm requires wasm-opt from Binaryen %s\n' "$expected_binaryen" >&2
  exit 1
fi
actual_binaryen=$($wasm_opt --version | sed -n 's/^wasm-opt version \([0-9][0-9]*\).*$/\1/p')
if [ "$actual_binaryen" != "$expected_binaryen" ]; then
  printf 'plugin.wasm requires Binaryen %s, found %s\n' \
    "$expected_binaryen" "${actual_binaryen:-unknown}" >&2
  exit 1
fi

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
optimized="$target_dir/wasm32-unknown-unknown/release/axodendron_typst_plugin.opt.wasm"
"$wasm_opt" -Os \
  --enable-bulk-memory \
  --enable-bulk-memory-opt \
  --enable-nontrapping-float-to-int \
  "$artifact" \
  -o "$optimized"
cp "$optimized" "$repository_dir/package/plugin.wasm"
