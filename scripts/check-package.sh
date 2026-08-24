#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
source_dir="$repository_dir/package"
package_data=$(mktemp -d "${TMPDIR:-/tmp}/axodendron-package.XXXXXX")
output_dir=$(mktemp -d "${TMPDIR:-/tmp}/axodendron-package-output.XXXXXX")
trap 'rm -rf "$package_data" "$output_dir"' EXIT HUP INT TERM
package_version=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$source_dir/typst.toml")
package_dir="$package_data/typst/packages/preview/axodendron/$package_version"
typst_bin=${TYPST_BIN:-typst}

mkdir -p "$package_dir/docs" "$package_dir/examples/data" "$package_dir/images" "$package_dir/src"
cp "$source_dir/typst.toml" "$package_dir/typst.toml"
cp "$source_dir/lib.typ" "$package_dir/lib.typ"
cp "$source_dir"/src/*.typ "$package_dir/src/"
cp "$source_dir/plugin.wasm" "$package_dir/plugin.wasm"
cp "$source_dir/README.md" "$package_dir/README.md"
cp "$source_dir/LICENSE" "$package_dir/LICENSE"
cp "$source_dir/THIRD_PARTY_NOTICES.md" "$package_dir/THIRD_PARTY_NOTICES.md"
cp "$source_dir/docs/documentation.typ" "$package_dir/docs/documentation.typ"
cp "$source_dir"/examples/*.typ "$package_dir/examples/"
cp "$source_dir"/examples/data/*.swc "$package_dir/examples/data/"
cp "$source_dir"/images/*.png "$package_dir/images/"

TYPST_PACKAGE_PATH="$package_data/typst/packages" "$typst_bin" compile \
  --root "$package_dir" \
  "$package_dir/examples/basic.typ" \
  "$output_dir/basic.pdf"
TYPST_PACKAGE_PATH="$package_data/typst/packages" "$typst_bin" compile \
  --root "$package_dir" \
  "$package_dir/examples/readme.typ" \
  "$output_dir/readme.pdf"
TYPST_PACKAGE_PATH="$package_data/typst/packages" "$typst_bin" compile \
  --root "$package_dir" \
  "$package_dir/examples/overview.typ" \
  "$output_dir/overview.pdf"
TYPST_PACKAGE_PATH="$package_data/typst/packages" "$typst_bin" compile \
  --root "$package_dir" \
  "$package_dir/examples/analysis.typ" \
  "$output_dir/analysis.pdf"
TYPST_PACKAGE_PATH="$package_data/typst/packages" "$typst_bin" compile \
  --root "$package_dir" \
  "$package_dir/examples/rendering.typ" \
  "$output_dir/rendering.pdf"
TYPST_PACKAGE_PATH="$package_data/typst/packages" "$typst_bin" compile \
  --root "$package_dir" \
  "$package_dir/examples/cetz.typ" \
  "$output_dir/cetz.pdf"
for example_name in morphometrics topology tmd population; do
  TYPST_PACKAGE_PATH="$package_data/typst/packages" "$typst_bin" compile \
    --root "$package_dir" \
    "$package_dir/examples/$example_name.typ" \
    "$output_dir/$example_name.pdf"
done

files=$(find "$package_dir" -type f | sed "s|$package_dir/||" | sort)
expected='LICENSE
README.md
THIRD_PARTY_NOTICES.md
docs/documentation.typ
examples/analysis.typ
examples/basic.typ
examples/cetz.typ
examples/data/AA0109.CNG.swc
examples/data/Nr5a1-Cre_Ai14-187777-05-02-01_491392821_m.kp1.swc
examples/data/Sst-IRES-Cre_Ai14-188740-03-02-01_491119369_m.kp12.swc
examples/data/Vipr2-IRES2-Cre_Ai14-310513-05-02-01_637021223_m.CNG.swc
examples/morphometrics.typ
examples/overview.typ
examples/population.typ
examples/readme.typ
examples/rendering.typ
examples/tmd.typ
examples/topology.typ
images/readme-analysis.png
images/readme-cetz.png
images/readme-example.png
images/readme-overview.png
images/readme-rendering.png
lib.typ
plugin.wasm
src/analysis.typ
src/annotations.typ
src/cetz.typ
src/population.typ
src/protocol.typ
src/rendering.typ
src/tmd.typ
src/transforms.typ
typst.toml'
if [ "$files" != "$expected" ]; then
  printf 'unexpected package bundle:\n%s\n' "$files" >&2
  exit 1
fi

size=$(du -sk "$package_dir" | awk '{print $1}')
if [ "$size" -gt 4096 ]; then
  printf 'package bundle exceeds the 4 MiB budget: %s KiB\n' "$size" >&2
  exit 1
fi
printf 'package import verified: %s KiB\n' "$size"
