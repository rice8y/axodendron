#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
manifest="$repository_dir/wasm-plugin/test-data/neuromorpho-cases.tsv"
workspace_manifest="$repository_dir/wasm-plugin/Cargo.toml"
cache_dir=${AXODENDRON_NEUROMORPHO_DIR:-"$repository_dir/target/neuromorpho"}
gallery_dir=${AXODENDRON_GALLERY_DIR:-"$repository_dir/target/neuromorpho-gallery"}

for command_name in cargo rsvg-convert magick; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    printf 'required command not found: %s\n' "$command_name" >&2
    exit 2
  fi
done

if [ ! -d "$cache_dir" ]; then
  printf 'fixture cache not found; run ./scripts/fetch-neuromorpho-fixtures.sh first\n' >&2
  exit 2
fi

mkdir -p "$gallery_dir/xy" "$gallery_dir/projections" "$gallery_dir/styles" \
  "$gallery_dir/radius-policies" "$gallery_dir/soma-modes"
XDG_CACHE_HOME="$gallery_dir/.cache"
export XDG_CACHE_HOME
mkdir -p "$XDG_CACHE_HOME"
cd "$repository_dir"
CARGO_TARGET_DIR="$repository_dir/target" cargo build --manifest-path "$workspace_manifest" \
  --quiet --locked -p axodendron-svg --example render
renderer="$repository_dir/target/debug/examples/render"

render_png() {
  input=$1
  output=$2
  view=$3
  geometry=$4
  color=$5
  radius_mode=${6:-readable}
  soma_mode=${7:-equivalent}
  svg=${output%.png}.svg
  "$renderer" "$input" "$svg" "$view" "$geometry" "$color" \
    "$radius_mode" "$soma_mode"
  rsvg-convert --width 1200 --height 1200 "$svg" --output "$output"
}

tab=$(printf '\t')
tail -n +2 "$manifest" | while IFS="$tab" read -r nmo_id _rest; do
  [ -n "$nmo_id" ] || continue
  input="$cache_dir/$nmo_id.swc"
  if [ ! -f "$input" ]; then
    printf 'missing fixture: %s\n' "$input" >&2
    exit 2
  fi
  render_png "$input" "$gallery_dir/xy/$nmo_id.png" xy tapered type
done

for nmo_id in NMO_00001 NMO_37500 NMO_80000 NMO_120000 NMO_200000; do
  for radius_mode in readable physical; do
    render_png "$cache_dir/$nmo_id.swc" \
      "$gallery_dir/radius-policies/${nmo_id}-${radius_mode}.png" \
      xy tapered type "$radius_mode" equivalent
  done
  for soma_mode in equivalent encoded raw; do
    render_png "$cache_dir/$nmo_id.swc" \
      "$gallery_dir/soma-modes/${nmo_id}-${soma_mode}.png" \
      xy tapered type readable "$soma_mode"
  done
done

for nmo_id in NMO_00001 NMO_01000 NMO_80000 NMO_200000; do
  for view in xy xz yz iso; do
    render_png "$cache_dir/$nmo_id.swc" \
      "$gallery_dir/projections/${nmo_id}-${view}.png" "$view" tapered type
  done
  render_png "$cache_dir/$nmo_id.swc" \
    "$gallery_dir/styles/${nmo_id}-tapered-mono.png" xy tapered mono
  render_png "$cache_dir/$nmo_id.swc" \
    "$gallery_dir/styles/${nmo_id}-skeleton-type.png" xy skeleton type
done

magick montage "$gallery_dir"/xy/*.png \
  -thumbnail 360x360 -tile 4x -geometry +16+28 \
  -background '#f8fafc' -fill '#111827' \
  -set label '%t' "$gallery_dir/xy-contact.png"
magick montage "$gallery_dir"/projections/*.png \
  -thumbnail 360x360 -tile 4x -geometry +16+28 \
  -background '#f8fafc' -fill '#111827' \
  -set label '%t' "$gallery_dir/projections-contact.png"
magick montage "$gallery_dir"/styles/*.png \
  -thumbnail 360x360 -tile 4x -geometry +16+28 \
  -background '#f8fafc' -fill '#111827' \
  -set label '%t' "$gallery_dir/styles-contact.png"
magick montage "$gallery_dir"/radius-policies/*.png \
  -thumbnail 360x360 -tile 2x -geometry +16+28 \
  -background '#f8fafc' -fill '#111827' \
  -set label '%t' "$gallery_dir/radius-policies-contact.png"
magick montage "$gallery_dir"/soma-modes/*.png \
  -thumbnail 360x360 -tile 3x -geometry +16+28 \
  -background '#f8fafc' -fill '#111827' \
  -set label '%t' "$gallery_dir/soma-modes-contact.png"

printf 'local-only image gallery written to %s\n' "$gallery_dir"
