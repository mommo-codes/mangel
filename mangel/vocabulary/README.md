# vocabulary/

The words mangel knows: one directory per market, and one per language. You
do not need to write Rust to change anything in here.

```
vocabulary/
├── se/                     Sweden: how its register writes things
│   ├── abbreviations.toml  "Laktosfri" = "LF"
│   ├── units.toml          "l" = "L"              (unit → the register's spelling)
│   ├── categories.toml     "Frukt" = "Frukt & Bär"   (category → its group)
│   └── category_vat.toml   "Frukt" = "12"         (category → VAT %, or "manual")
└── language/               how a label in each language is read
    └── sv/                 Swedish
        └── units.toml      "styck" = "piece"      (word, lowercase → unit)
```

**A market and a language are different things.** A market's tables say how
its register writes a value: Sweden writes a litre `L`. A language's tables
say how a label printed in that language is read: Swedish calls a piece
`styck`. A pack sold in Sweden can be printed in Danish, so the two are kept
apart.

**Units are named by code**: `g`, `kg`, `ml`, `cl`, `dl`, `l`, and `piece` for
a count. The symbols themselves (`g`, `ML`, `l`) are read in every language,
so no language lists them. A language's `units.toml` holds only its words:
`gram`, `kilo`, `liter`, `st`, `styck` for Swedish. A market's `units.toml`
spells every unit once.

**A language has only the tables someone has written for it.** Swedish has
`units.toml`; the other languages have none yet, which means mangel reads no
unit words in them, not that something is missing. The languages are the
codes in `src/language.rs`: `sv da nb hu hr de el ka en`, and `tr it fr es pl`
for reading only.

`categories.toml` and `category_vat.toml` must name exactly the same
categories — a test holds them to it. A category name appears once in the
whole list: it is what the register stores. `"manual"` is a category whose
products carry more than one VAT rate (Jul & Nyår, Påsk), so the rate is typed
per product. The rates themselves (6, 12, 25) and the deposits (2, 3) are the
market's, in `src/market.rs`, not here.

These files are compiled into mangel when it is built. The published library
never reads them: by the time anyone installs it, every entry is part of the
binary. That is what lets the same answers come out of the backend and the
browser, offline.

## Everything here is public

The repository is public, and every release publishes these tables: to
crates.io as the files themselves, and in the Python and npm packages as plain
text inside the compiled library. Compiling does not hide an entry. Deleting
one later does not unpublish it either — it stays in the git history and in
every version already released.

So before adding an entry, ask whether it is fine for anyone to read. A
spelling like `"Laktosfri" = "LF"` is. A category formula, or a rule taken
from an internal standard, may not be. **If you are not sure, ask before you
commit it** — pushing to this repository is already publishing.

## Adding an entry

Open the table and add one line:

```toml
"Fettfri" = "FF"
```

- **Both sides in double quotes.** Always, even for a plain word. Without
  quotes, a word containing å, ä or ö is a syntax error.
- **One entry per line**, in any order. The build sorts them.
- **Exactly as the word is spelled.** No spaces before or after it inside the
  quotes.
- **A word can appear only once.** Listing it twice is an error, so one word
  can never have two abbreviations.

## Leaving a note

Anything after `#` is a comment. Use one whenever an entry — or a missing
entry — would make someone ask why:

```toml
# Sockerfri is deliberately not abbreviated: <why>.
```

A comment explaining why a word is **not** in the table is as important as
the entries. Without it, the next person adds the word.

## What gets checked

The build refuses a table, and names the file and line, when:

- a line is not valid TOML — usually a missing quote;
- a word appears twice;
- a word or abbreviation is empty, or starts or ends with a space (including
  the non-breaking space that copy-paste from web pages and PDFs brings in);
- it contains a tab, a line break or another invisible control character;
- a letter and its accent are stored as two characters. `ä` pasted from some
  PDFs arrives as `a` followed by a separate `¨`. It looks identical and would
  never match the real letter, so the build asks you to retype it;
- the file is not saved as UTF-8.

It also refuses a file it does not recognise, so a misspelled
`abbreviatons.toml` fails loudly instead of being silently ignored. Hidden
files such as `.DS_Store` are skipped.

## Checking your change

Push a branch. CI builds mangel and reports any problem with the file and the
line. If you have Rust installed, `cargo build -p mangel` shows the same
messages locally.

**An edit here reaches CatalogOS only through a release.** Merging adds the
entry to mangel. It is in production once a version containing it has been
published and CatalogOS has been moved onto that version. That trade is
deliberate; see [docs/vocabulary.md](../../docs/vocabulary.md).

## Adding words for a language

Create `language/<code>/units.toml` if it is not there, and add lines:

```toml
"stk" = "piece"
"gramm" = "g"
```

The word on the left is lowercase. The unit on the right is one of the codes
above. The build refuses a word with capitals, a symbol (it is read already),
and a unit mangel does not know.

## Adding a table, a market or a language

Each needs a code change first, because each needs code that uses it:

- **A new market table** is a field on `Vocabulary` in `src/vocabulary.rs`, a
  name in `TABLES` in `build.rs`, and one file per market.
- **A new language table** is a field on `LanguageVocabulary` and a name in
  `LANGUAGE_TABLES`. A language without the file simply has none of its words.
- **A new market** is a variant of `Market` in `src/market.rs`, then a
  directory here containing every table. A directory for a market that does
  not exist in code is refused.
- **A new language** is a variant of `Language` in `src/language.rs`. A
  directory under `language/` for a language that does not exist in code is
  refused.
