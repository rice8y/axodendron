#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
first=$(mktemp -d "${TMPDIR:-/tmp}/axodendron-wasm-a.XXXXXX")
second=$(mktemp -d "${TMPDIR:-/tmp}/axodendron-wasm-b.XXXXXX")
rust_host=$(rustc --version --verbose | sed -n 's/^host: //p')
canonical_host=x86_64-unknown-linux-gnu
restore_committed=0

cleanup() {
  if [ "$restore_committed" -eq 1 ] && [ -f "$first/committed.wasm" ]; then
    cp "$first/committed.wasm" "$repository_dir/package/plugin.wasm"
  fi
  rm -rf "$first" "$second"
}
trap cleanup EXIT HUP INT TERM

cd "$repository_dir"
cp package/plugin.wasm "$first/committed.wasm"
if [ "$rust_host" != "$canonical_host" ]; then
  restore_committed=1
fi
AXODENDRON_TARGET_DIR="$first" "$script_dir/build-plugin.sh"
cp package/plugin.wasm "$first/plugin.wasm"
AXODENDRON_TARGET_DIR="$second" "$script_dir/build-plugin.sh"
cmp "$first/plugin.wasm" package/plugin.wasm

if [ "$rust_host" = "$canonical_host" ]; then
  if ! cmp -s "$first/committed.wasm" "$first/plugin.wasm"; then
    printf 'committed plugin.wasm does not match the canonical %s build\n' "$canonical_host" >&2
    exit 1
  fi
fi

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
if [ "$rust_host" = "$canonical_host" ]; then
  printf 'reproducible committed plugin verified: %s bytes\n' "$size"
else
  printf 'reproducible plugin verified on %s: %s bytes; canonical artifact preserved\n' \
    "$rust_host" "$size"
fi
