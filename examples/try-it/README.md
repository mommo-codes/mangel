# Try it

A page for trying mangel by hand. Paste a messy value, or a column of them
one per line, and pick the field, the profile and the languages. Each line
comes back as the market writes it, with what was changed (`spacing`,
`unit_spelling`) and the value without a language. Or it comes back declined
with its reason and code, or marked `no rule` where the profile has none for
that field.

It runs the local WebAssembly build, so it always answers with the code in
this checkout. Nothing is sent anywhere.

```sh
( cd mangel-wasm/npm && npm install && npm run build )   # builds wasm/ and dist/
python3 -m http.server 8000                              # from the repository root
```

Then open <http://localhost:8000/examples/try-it/>.

Serve from the repository root, not from this folder: the page imports
`../../mangel-wasm/npm/dist/index.js`, and the browser has to be able to reach
it. Opening the file directly (`file://`) does not work, because browsers do
not load WebAssembly modules from disk.

## What it shows

- **read**: the value as the market writes it, what was changed, and the
  value without a language (a size's amount and unit code).
- **declined**: mangel's reason and its stable code.
- **no rule**: the profile has no rule for this field yet. The Laundry Room
  profile has none until its first rule, weight and volume, lands.

There is one market so far, Sweden (`se`). The fields, profiles and languages
on the page come from the library itself (`fields()`, `profiles()`,
`languages()`). No rule reads the label or output language yet, and the page
says so under the form.

The page decides nothing. `try-it.js` sends each line to `read` and prints
the answer. If a rule seems wrong here, it is wrong in the library, and the
fix goes in `mangel/`.
