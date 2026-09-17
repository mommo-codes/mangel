# mangel ships vocabulary, and barkod does not

barkod's [`docs/no-data.md`](https://github.com/mommo-codes/barkod/blob/main/docs/no-data.md)
argues that data does not belong in a library: lookups go stale, the
questions are joins, and a correction should not need a release. mangel
compiles a vocabulary into its binary anyway. This is why the two are
consistent, what it costs, and where the line sits.

## What the vocabulary is

**Spelling conventions.** How the golden standard writes a word or a unit in
one market: `Laktosfri` is written `LF`; a litre is written the way the
standard writes it. The vocabulary says how something is spelled. It never
says what something is, who owns it or which row it belongs to.

That is a different kind of data from the registries barkod excludes:

| | A registry (barkod keeps out) | Vocabulary (mangel compiles in) |
|---|---|---|
| Example | which company owns a GS1 prefix | `Laktosfri` → `LF` |
| Maintained by | an outside body, with publication dates | this team |
| Answers | a join over stored rows | how to write one field |
| Needed offline, mid-keystroke | no | yes |

## Why it is in the library at all

**Because the browser needs it.** Name Scrubbing normalises while someone
types. That runs in the browser, synchronously, with no round trip, so the
word lists have to be in the bundle. barkod's own note left exactly this case
open: data a frontend needs offline cannot come from a server.

**Because two runtimes must agree.** mangel exists so the backend and the
browser give the same answer to the same field. If the word lists were loaded
separately from the code, "same library version" would stop meaning "same
answer" — the two could run identical code against different lists. Compiling
the vocabulary in makes the version number cover both.

## Why it is compiled in rather than read

- **No I/O.** The core reads nothing at run time, vocabulary included. There
  is no file to be missing in production and no path to configure.
- **A broken entry fails the build, not a request.** `build.rs` parses and
  checks every table while compiling. The binary contains static slices;
  there is no parse left to fail.
- **One artefact.** The wheel, the wasm and the crate each carry the whole
  vocabulary. Nothing ships beside them.

The TOML is read by the [`toml`](https://crates.io/crates/toml) crate, as a
**build dependency**. Rust's standard library has no TOML parser, so that is
one real dependency. It runs on the machine compiling mangel and is not linked
into the library, so `cargo tree -p mangel --edges normal` is still one line,
and CI still asserts that.

## Why TOML

Decided 2026-09-17, over:

- **JSON** — no comments, so nothing can record why a word is deliberately
  left out. One stray comma breaks the file for someone who does not write
  code. Most parsers quietly keep the last copy of a duplicated key.
- **TSV** — editable in a spreadsheet, but Excel in a Swedish locale saves
  "CSV" with semicolons and often not in UTF-8, which corrupts å, ä and ö on
  the way back in. No comments either.
- **YAML** — reads well, but guesses types: `no` becomes `false`, which bites
  the moment a Norwegian market arrives. The standard Rust YAML crate is
  archived.

TOML takes comments, refuses a duplicated key outright, and one
`"Word" = "ABBR"` line reads cleanly to someone who has never seen the format.

## The cost, accepted

**Every vocabulary change is a release.** Adding `"Fettfri" = "FF"` reaches
production only through the same path as a code change: merge, tag, publish to
crates.io, PyPI and npm, move CatalogOS onto the new version.

That was the trade, made knowingly. What compiling it in buys is that the
file is readable and editable by someone who does not write Rust — not that it
skips the release. If release friction turns out to be the real problem, that
is a different fix, and it will be visible by then.

## Where the line is

The vocabulary holds **exact spellings and nothing else**. How a messy field
is matched against them — case, spacing, inflection, word boundaries — is a
parsing rule, and parsing rules live in code.

For anything proposed as vocabulary, ask:

1. **Is it a condition?** "Abbreviate this unless…" is logic. It belongs in
   Rust, where it is tested.
2. **Is it a join?** Matching a normalised name to a brand row, a product or
   a supplier happens in CatalogOS, against its database. mangel can make
   `coca cola` read `Coca-Cola`; which brand row that is, is not its question.
3. **Does an outside body publish it with a date?** Then it is a registry, and
   barkod's reasoning applies unchanged: it belongs in a database.

A spelling that passes all three belongs in a table. A spelling written into a
`match` arm in Rust is in the wrong place, and so is a condition written into
a TOML comment.

## Markets

Sweden is the only market, but nothing assumes it is the only one. Every
table sits under a market directory, every public function takes a market,
and there is no default: a default right for Sweden would be silently wrong
for the next market.

A market is added in code first — a variant of `Market` — because a market
brings its own rules, not only its own words. The build refuses a vocabulary
directory for a market that does not exist in code.

**Not settled:** a market with more than one language. Finland writes product
data in Finnish and Swedish. Whether that is one market with two sets of
tables, or two markets, is recorded here and decided when a real case arrives.
