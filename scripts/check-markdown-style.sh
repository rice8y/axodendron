#!/usr/bin/env sh
set -eu

repository_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
status=0
file_list=$(mktemp "${TMPDIR:-/tmp}/axodendron-markdown.XXXXXX")
trap 'rm -f "$file_list"' EXIT HUP INT TERM

find "$repository_dir" \( -path '*/.git' -o -path '*/target' \) -prune -o \
  -type f -name '*.md' -print > "$file_list"
while IFS= read -r file; do
  if ! awk '
    function structural(line) {
      return line ~ /^(#|[-*+] |[0-9]+\. |\||>|    |<)/
    }
    /^```/ {
      fenced = !fenced
      previous = ""
      next
    }
    fenced { next }
    /^[[:space:]]*$/ {
      previous = ""
      next
    }
    {
      if (previous != "" && !structural(previous) && !structural($0)) {
        printf "%s:%d: prose paragraphs must occupy one physical line\n", FILENAME, NR
        invalid = 1
      }
      previous = $0
    }
    END { exit invalid }
  ' "$file"; then
    status=1
  fi
done < "$file_list"

exit "$status"
