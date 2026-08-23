#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
typst_bin=${TYPST_BIN:-typst}
temporary_dir=$(mktemp -d "${TMPDIR:-/tmp}/axodendron-readme.XXXXXX")
trap 'rm -rf "$temporary_dir"' EXIT HUP INT TERM
package_version=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$repository_dir/package/typst.toml")
package_root="$temporary_dir/typst/packages/preview/axodendron/$package_version"

for command_name in "$typst_bin" qpdf pdftoppm; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    printf 'required command not found: %s\n' "$command_name" >&2
    exit 2
  fi
done

mkdir -p "$package_root/src"
cp "$repository_dir/package/typst.toml" "$package_root/typst.toml"
cp "$repository_dir/package/lib.typ" "$package_root/lib.typ"
cp "$repository_dir"/package/src/*.typ "$package_root/src/"
cp "$repository_dir/package/plugin.wasm" "$package_root/plugin.wasm"

render_image() {
  source_name=$1
  output_name=$2
  pdf="$temporary_dir/$output_name.pdf"
  png_prefix="$temporary_dir/$output_name"
  TYPST_PACKAGE_PATH="$temporary_dir/typst/packages" "$typst_bin" compile \
    --root "$repository_dir" \
    "$repository_dir/package/examples/$source_name.typ" \
    "$pdf"
  page_count=$(qpdf --show-npages "$pdf")
  if [ "$page_count" != "1" ]; then
    printf '%s must compile to exactly one page; found %s\n' "$source_name" "$page_count" >&2
    exit 1
  fi
  pdftoppm -png -r 180 -singlefile "$pdf" "$png_prefix"
  cp "$png_prefix.png" "$repository_dir/package/images/$output_name.png"
}

render_image overview readme-overview
render_image readme readme-example
render_image analysis readme-analysis
render_image rendering readme-rendering
render_image cetz readme-cetz
printf 'README images written to package/images/\n'
