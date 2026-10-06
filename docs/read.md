# read(): fields, profiles and languages

`read` is one call for every field. A caller says which field a text is and
in what context. mangel answers with the value as the market writes it, or
declines with a code.

```python
mangel.read("size", "0,5 l", market="se", profile="name_scrubbing")
# {"value": "0,5L",
#  "neutral": {"kind": "size", "amount": "0.5", "unit": "l"},
#  "changes": ["spacing", "unit_spelling"]}
```

```ts
read("size", "0,5 l", { market: "se", profile: "name_scrubbing" });
```

```rust
let context = Context::new(Market::Se, Profile::NameScrubbing);
let read = mangel::read(Field::Size, "0,5 l", &context)?;
```

The existing functions (`size`, `cleaned_name`, `category`, `vat`, `deposit`)
are unchanged and stay. `read` is the call new rules are added behind.

## What comes back

| Part | What it is |
|---|---|
| `value` | The value as the market writes it. A size in Sweden is `0,5L`: a comma, no space, a capital L. |
| `neutral` | The value without a language or a market's spelling. A size is an amount with a `.` point and a unit code (`g kg ml cl dl l piece`). A category is the category. Free text is the text, in the language it arrived in. |
| `changes` | What was changed on the way, as codes. Empty when the text was already written the market's way. |

| Change | Means |
|---|---|
| `spacing` | Spaces trimmed, collapsed, added or removed. |
| `amount` | The amount written the market's way: `1.50` to `1,5`. |
| `unit_spelling` | The unit spelled the market's way: `l` to `L`. |

A decline carries a code; see [decline-codes.md](decline-codes.md).

## Context

| Input | What it is | Default |
|---|---|---|
| `market` | Whose register the value goes into. Sets how the value is written. | Required. |
| `profile` | Who mangel reads for. | Required. |
| `label` | The languages printed on the pack, any number, as ISO 639-1 codes. It decides which words a label is read with. | None known. |
| `output` | The language a language-neutral value is named in, such as a country. Free text is never translated. | The market's language. |
| `category` | The product's category, which a VAT rate is read against. | None. |

A code mangel does not know, for a market, field, profile or language, raises
a plain error (`ValueError` in Python), never a decline.

### Languages

`languages()` lists them. There are two kinds:

- **Output languages**, which a value can be named in: `sv da nb hu hr de el ka en`.
  They are the label languages the consuming OCR returns text in.
- **Reading languages**, which mangel only reads: `tr it fr es pl`. Real packs
  on the same shelves carry them.

Today no rule reads `label` or `output`; the market's own language decides.
They are part of the call now so that the rules that need them (weight and
volume first, #18) are added without changing it.

## Profiles

A profile is a table, compiled in, from field to mode:

| Mode | Means |
|---|---|
| Check | The field's existing rule. A wrong value is declined, never altered beyond its spelling. |
| Off | No rule for this field in this profile: declined with `no_rule`. |

| Field | `name_scrubbing` | `laundry_room` |
|---|---|---|
| name, size, category, vat, deposit | Check | Off |
| every other field | Off | Off |

**Name Scrubbing** reads a value a person typed into the cleaning sheet, with
the rules it has always had.

**Laundry Room** reads what OCR returned from a label. It has no field yet.
Each field joins with a rule that fixes what is mechanical and safe, and only
once that rule measurably beats what Laundry Room produces today. A third
mode, Fix, arrives with the first of them, weight and volume (#18). Moving a
field from Off to Fix, or from Check to Fix, is one row in `Profile::mode`
and its pinned test.

## Fields

`fields()` lists them:

- `name`, `brand`, `country_of_origin`, `description`, `ingredients`,
  `allergens`, `additives`, `producer`, `distributor`;
- `alcohol_percentage`, `weight`, `volume`, `quantity`;
- `size`, the register's one size field, and `category`, `vat`, `deposit`.

Nutrition is eight numbers and a basis, not one text, so it gets its own call
(#33).
