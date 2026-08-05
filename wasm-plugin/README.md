# Axodendron WASM plugin

This directory is the independent Rust workspace behind the Axodendron Typst package. `axodendron-core` implements SWC parsing, topology, scientific metrics, and pure transformations; `axodendron-svg` implements deterministic orthographic SVG rendering; `axodendron-typst-plugin` is the bounded, versioned CBOR adapter exported through Typst's minimal WASM protocol.

The crates are implementation components and set `publish = false`; nothing in this workspace is released to crates.io. The only shipped binary is `package/plugin.wasm`, built with the exact Rust version in `rust-toolchain.toml`; its committed bytes are verified on the canonical `x86_64-unknown-linux-gnu` release host, while other hosts verify same-host reproducibility without replacing the canonical artifact during the full check.

Run all Rust tests with:

```sh
cargo test --manifest-path wasm-plugin/Cargo.toml --locked --workspace --all-targets
```

The checksum-pinned NeuroMorpho test corpus is downloaded only into the ignored repository `target/` cache and is never copied into this workspace or the Typst package, except for the four explicitly attributed README examples in `package/examples/`.
