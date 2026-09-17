# mangel

Irregular product data in, regular product data out. One implementation,
callable from **Rust**, **Python** and **TypeScript**.

*Mangel* is Swedish for a mangle — the roller press that squeezed water out of
washing and flattened everything to the same shape.

mangel takes a messy field and returns what it means:

| Field | Means |
|---|---|
| `3x45m` | a multipack: 3 units of 45 metres |
| `45l` | 45 litres, in the golden standard's spelling |
| `Laktosfri` | `LF` |
| `coca cola` | `Coca-Cola` — casing and spacing only; matching it to a brand row is CatalogOS's job |

> **Status: scaffold.** None of the parsing above exists yet; it is being
> built one rule at a time. What exists is the structure it goes into: the
> three runtimes, the market every answer is given for, and the vocabulary
> compiled into the binary. Nothing is published.

## Deterministic and pure

No network, no database, no credentials, no clock, no randomness, nothing read
at run time. The same input gives the same output, every time, offline. It
runs in two places — over OCR output in the backend, and in the browser while
someone types — and it has to give both the same answer.

That is the design constraint, and it is enforced rather than intended:

- **The core has no runtime dependencies.** CI asserts `cargo tree` is one line.
- **Impurity does not compile.** `mangel/clippy.toml` refuses files, sockets,
  clocks, environment variables, and hash maps whose iteration order changes
  between runs.
- **The vocabulary is compiled in**, not loaded.

## Logic in code, vocabulary in data

Abbreviations, unit spellings and the like live in TOML, one directory per
market, so someone who does not write Rust can add to them:

```toml
# mangel/vocabulary/se/abbreviations.toml
"Laktosfri" = "LF"
```

`build.rs` reads and checks every table at compile time and bakes it into the
binary. A malformed entry fails the build with the file and line; the published
library never parses TOML. Every vocabulary change is therefore a release,
deliberately — see [docs/vocabulary.md](docs/vocabulary.md).

How to edit the tables: [mangel/vocabulary/README.md](mangel/vocabulary/README.md).

## Layout

| Crate | What it is |
|---|---|
| [`mangel/`](mangel) | The core. No runtime dependencies, no `unsafe`, no I/O. Holds `vocabulary/` and the `build.rs` that compiles it. |
| [`mangel-py/`](mangel-py) | PyO3/maturin bindings. |
| [`mangel-wasm/`](mangel-wasm) | wasm-bindgen bindings plus the TypeScript wrapper, published as `@mommo-codes/mangel`. |

The core does not depend on PyO3 or wasm-bindgen. The bindings decide
nothing: every rule is written once, in the core.

## Testing

```sh
cargo test --workspace                 # core, vocabulary pipeline, build-time checks
cargo clippy --workspace --all-targets -- -D warnings
( cd mangel-py && maturin develop && pytest tests/ )
( cd mangel-wasm/npm && npm test )     # build + typecheck + node
```

Each guard has been watched failing on purpose: a padded, decomposed and
mistyped vocabulary entry, a misspelled table file, a `HashMap` and an
environment read in the core, a runtime dependency, and a code generator that
silently emitted no entries. A green run means something only if you have seen
the red one.

MIT licensed.
