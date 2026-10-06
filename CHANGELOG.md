# Changelog

## Unreleased

- `size` declines an amount that reads as two numbers: one to three digits
  (not starting with 0), a separator, and exactly three digits. `1.000 g`
  was read as 1 g; it is now declined as "could be 1 g or 1000 g". A point
  declines in every unit. A comma declines in g, ml, cl, dl and st and stays
  a decimal in kg and L, where three decimals are whole grams and
  millilitres (`1,048kg`, `1,000 kg`). `0,750 L` and `1,25 kg` read as
  before. The rule recorded in #2; the bug is #7.

## 0.1.0

The first rules — the ones Name Scrubbing's cleaning sheet reads with. Every
rule answers for one market and returns what a field means, or declines with
a sentence a person can act on. Nothing is guessed.

- `size`: `56kg` is 56 and `kg`; `0,5 l` is 0.5 and `L`. The unit is matched
  without regard to case against the new `units` table and returned in the
  golden spelling. No unit, no amount, a multipack or an unknown unit is
  declined.
- `cleaned_name`: trimmed, spacing collapsed, and the first word must start
  with a capital. Checked, not fixed.
- `category`: found by its name, by a group holding only it, or by a keyword
  that starts or ends a word of exactly one category's name (`pizza` is Fryst
  Pizza; `kaffe` is in seven and is declined with them). The Swedish list —
  180 categories in 32 groups — is the new `categories` table.
- `vat`: a category's rate, from the new `category_vat` table. Typed only for
  the two mixed-rate categories (Jul & Nyår, Påsk); a typed rate that
  contradicts the category is declined, not corrected.
- `deposit`: 2 or 3 in Sweden, or none.
- `Market::vat_rates` and `Market::deposits`.
- All of it in Python (`mangel.Declined`, a `ValueError`) and TypeScript (an
  `Error` named `"Declined"`, and `isDeclined`).

## 0.0.0

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
