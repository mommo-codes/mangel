# @mommo-codes/mangel

Product data normalisation: irregular fields in, regular fields out. Written
in Rust, compiled to WebAssembly — the same implementation the Rust crate and
the Python package run, vocabulary included, so all three give the same
answers.

Scoped from the start. npm refused barkod's bare name as too similar to an
existing package, and `mangel` is two transposed letters away from the
existing `mangle`. The crate and the Python package are unscoped.

**Not yet published, and no parsing rules exist yet.** What the package
exposes today is the vocabulary compiled into the library.

```ts
import * as mangel from "@mommo-codes/mangel";

await mangel.init(); // loads the wasm once, at start-up

mangel.abbreviations("se").get("Laktosfri"); // "LF"
mangel.abbreviations("SE");                  // throws — market codes are exact
```

Every function throws until `init()` resolves, rather than returning a
placeholder. Every call takes a market, and there is no default.

Full documentation: <https://github.com/mommo-codes/mangel>

MIT licensed.
