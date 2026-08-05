#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
manifest="$repository_dir/wasm-plugin/Cargo.toml"
output_dir=$(mktemp -d "${TMPDIR:-/tmp}/axodendron-check.XXXXXX")
typst_bin=${TYPST_BIN:-typst}
trap 'rm -rf "$output_dir"' EXIT HUP INT TERM

cd "$repository_dir"
swc_files=$(find . \( -path './.git' -o -path './target' \) -prune -o \
  -type f -name '*.swc' -print | sort)
expected_swc_files='./package/examples/data/AA0109.CNG.swc
./package/examples/data/Nr5a1-Cre_Ai14-187777-05-02-01_491392821_m.kp1.swc
./package/examples/data/Sst-IRES-Cre_Ai14-188740-03-02-01_491119369_m.kp12.swc
./package/examples/data/Vipr2-IRES2-Cre_Ai14-310513-05-02-01_637021223_m.CNG.swc'
if [ "$swc_files" != "$expected_swc_files" ]; then
  printf 'unexpected repository SWC set:\n%s\n' "$swc_files" >&2
  exit 1
fi
checksum() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print $1 }'
  else
    shasum -a 256 "$1" | awk '{ print $1 }'
  fi
}
for example in package/examples/data/AA0109.CNG.swc \
  package/examples/data/Nr5a1-Cre_Ai14-187777-05-02-01_491392821_m.kp1.swc \
  package/examples/data/Sst-IRES-Cre_Ai14-188740-03-02-01_491119369_m.kp12.swc \
  package/examples/data/Vipr2-IRES2-Cre_Ai14-310513-05-02-01_637021223_m.CNG.swc; do
  hash=$(checksum "$example")
  if ! grep -Fq "$hash" package/THIRD_PARTY_NOTICES.md; then
    printf 'missing or stale example checksum notice: %s\n' "$example" >&2
    exit 1
  fi
done

"$script_dir/check-markdown-style.sh"
"$script_dir/check-readme-sync.sh"
cargo fmt --manifest-path "$manifest" --all -- --check
cargo clippy --manifest-path "$manifest" --locked --workspace --all-targets -- -D warnings
cargo test --manifest-path "$manifest" --locked --workspace --all-targets
cargo test --manifest-path "$manifest" --locked -p axodendron-core --test limits -- --ignored
RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path "$manifest" --locked --workspace --no-deps
"$script_dir/benchmark.sh"
"$script_dir/verify-plugin.sh"
TYPST_BIN="$typst_bin" "$script_dir/check-package.sh"
"$typst_bin" compile --root . package/tests/smoke.typ "$output_dir/smoke.pdf"

if [ "${AXODENDRON_WITH_NEUROMORPHO:-0}" = "1" ]; then
  "$script_dir/fetch-neuromorpho-fixtures.sh"
  cargo test --manifest-path "$manifest" --locked \
    -p axodendron-core --test neuromorpho -- --ignored
  cargo test --manifest-path "$manifest" --locked \
    -p axodendron-svg --test neuromorpho_render -- --ignored
  if [ -z "${AXODENDRON_NEUROMORPHO_DIR:-}" ]; then
    "$typst_bin" compile --root . package/tests/neuromorpho.typ \
      "$output_dir/neuromorpho.pdf"
  fi
fi
