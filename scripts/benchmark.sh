#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
manifest="$repository_dir/wasm-plugin/Cargo.toml"

cd "$repository_dir"
cargo run --manifest-path "$manifest" --quiet --locked --release \
  -p axodendron-core --example benchmark -- 100000 2000
cargo run --manifest-path "$manifest" --quiet --locked --release \
  -p axodendron-svg --example benchmark -- 100000 2000

if [ "${AXODENDRON_BENCH_FULL:-0}" = "1" ]; then
  cargo run --manifest-path "$manifest" --quiet --locked --release \
    -p axodendron-core --example benchmark -- 250000 5000
fi
