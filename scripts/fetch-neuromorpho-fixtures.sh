#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
manifest="$repository_dir/wasm-plugin/test-data/neuromorpho-cases.tsv"
cache_dir=${AXODENDRON_NEUROMORPHO_DIR:-"$repository_dir/target/neuromorpho"}
user_agent='Axodendron quality tests/0.1.1 (+https://github.com/rice8y/axodendron)'

mkdir -p "$cache_dir"
cache_dir=$(CDPATH= cd -- "$cache_dir" && pwd)
case "$cache_dir" in
  "$repository_dir"|"$repository_dir"/*)
    case "$cache_dir" in
      "$repository_dir/target"|"$repository_dir/target"/*) ;;
      *)
        printf 'refusing to store NeuroMorpho.Org SWCs in a publishable repository path: %s\n' \
          "$cache_dir" >&2
        exit 2
        ;;
    esac
    ;;
esac

checksum() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

tab=$(printf '\t')
tail -n +2 "$manifest" | while IFS="$tab" read -r nmo_id file_name swc_url expected_hash _rest; do
  if [ -z "$nmo_id" ]; then
    continue
  fi
  if ! printf '%s\n' "$nmo_id" | grep -Eq '^NMO_[0-9]+$'; then
    printf 'invalid NeuroMorpho.Org identifier in manifest: %s\n' "$nmo_id" >&2
    exit 2
  fi
  case "$swc_url" in
    https://neuromorpho.org/dableFiles/*) ;;
    *)
      printf 'refusing undocumented morphology URL: %s\n' "$swc_url" >&2
      exit 2
      ;;
  esac
  destination="$cache_dir/$nmo_id.swc"
  if [ -f "$destination" ] && [ "$(checksum "$destination")" = "$expected_hash" ]; then
    printf 'verified %s (%s)\n' "$nmo_id" "$file_name"
    continue
  fi

  temporary=$(mktemp "$cache_dir/.axodendron-download.XXXXXX")
  if ! curl --fail --location --silent --show-error --retry 3 \
    --user-agent "$user_agent" "$swc_url" --output "$temporary"; then
    rm -f "$temporary"
    exit 1
  fi
  actual_hash=$(checksum "$temporary")
  if [ "$actual_hash" != "$expected_hash" ]; then
    printf 'checksum mismatch for %s: expected %s, found %s\n' \
      "$nmo_id" "$expected_hash" "$actual_hash" >&2
    rm -f "$temporary"
    exit 1
  fi
  mv "$temporary" "$destination"
  printf 'downloaded %s (%s)\n' "$nmo_id" "$file_name"
  sleep 0.2
done
