#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
first=$(mktemp -d "${TMPDIR:-/tmp}/axodendron-wasm-a.XXXXXX")
second=$(mktemp -d "${TMPDIR:-/tmp}/axodendron-wasm-b.XXXXXX")
trap 'rm -rf "$first" "$second"' EXIT HUP INT TERM

cd "$repository_dir"
AXODENDRON_TARGET_DIR="$first" "$script_dir/build-plugin.sh"
cp package/plugin.wasm "$first/plugin.wasm"
AXODENDRON_TARGET_DIR="$second" "$script_dir/build-plugin.sh"
cmp "$first/plugin.wasm" package/plugin.wasm

wasm-tools validate package/plugin.wasm
if strings package/plugin.wasm | grep -E '/Users/|/home/|\.cargo/registry' >/dev/null; then
  printf 'plugin.wasm contains a host-specific absolute path\n' >&2
  exit 1
fi

size=$(wc -c < package/plugin.wasm | tr -d ' ')
if [ "$size" -gt 1048576 ]; then
  printf 'plugin.wasm exceeds the 1 MiB release budget: %s bytes\n' "$size" >&2
  exit 1
fi

wasm-tools print package/plugin.wasm | grep 'import "typst_env"' >/dev/null
printf 'reproducible plugin verified: %s bytes\n' "$size"
