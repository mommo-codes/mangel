# Changelog

## Unreleased

The scaffold. No parsing rules, nothing published.

- Cargo workspace in barkod's shape: the core in `mangel/`, PyO3 bindings in
  `mangel-py/`, wasm-bindgen bindings and a TypeScript wrapper in
  `mangel-wasm/`.
- `Market`, with Sweden as its only variant. Every public function takes one,
  and there is no default.
- The vocabulary: TOML under `mangel/vocabulary/<market>/`, checked and
  compiled into the binary by `build.rs`. One table, `abbreviations`, holding
  one entry, `Laktosfri` → `LF`.
- `abbreviations(market)` in all three runtimes, so the compiled tables can be
  seen crossing both boundaries.
- CI: format, clippy with the purity lints in `mangel/clippy.toml`, tests in
  all three runtimes, the no-runtime-dependency assertion, and a check that
  Cargo and npm carry the same version. The release workflow is barkod's,
  renamed.
