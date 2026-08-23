#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
typst_bin=${TYPST_BIN:-typst}
python_bin=${PYTHON_BIN:-python3}
baseline="$repository_dir/tests/typst/visual-regression.sha256"
actual=${AXODENDRON_VISUAL_OUTPUT:-"${TMPDIR:-/tmp}/axodendron-visual-regression.png"}

"$python_bin" "$script_dir/test_png_pixel_sha256.py"

"$typst_bin" compile \
  --root "$repository_dir" \
  --format png \
  --ppi 144 \
  "$repository_dir/tests/typst/visual-regression.typ" \
  "$actual"

observed=$("$python_bin" "$script_dir/png-pixel-sha256.py" "$actual")
if [ "${AXODENDRON_UPDATE_VISUAL_BASELINE:-0}" = "1" ]; then
  printf '%s\n' "$observed" > "$baseline"
  printf 'visual baseline updated: %s\n' "$observed"
  exit 0
fi

expected=$(sed -n '1p' "$baseline")
if [ -z "$expected" ] || [ "$observed" != "$expected" ]; then
  printf 'visual regression mismatch\nexpected: %s\nobserved: %s\nimage: %s\n' \
    "$expected" "$observed" "$actual" >&2
  exit 1
fi
printf 'visual regression verified: %s\n' "$observed"
