#!/usr/bin/env sh
set -eu

repository_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
normalized=$(mktemp "${TMPDIR:-/tmp}/axodendron-readme-sync.XXXXXX")
trap 'rm -f "$normalized"' EXIT HUP INT TERM

sed \
  -e 's|package/images/|images/|g' \
  -e 's|package/examples/|examples/|g' \
  -e 's|package/docs/|docs/|g' \
  -e 's|package/THIRD_PARTY_NOTICES.md|THIRD_PARTY_NOTICES.md|g' \
  "$repository_dir/README.md" > "$normalized"

if ! cmp -s "$normalized" "$repository_dir/package/README.md"; then
  printf '%s\n' 'README.md and package/README.md differ beyond package-relative paths' >&2
  diff -u "$repository_dir/package/README.md" "$normalized" >&2 || true
  exit 1
fi
