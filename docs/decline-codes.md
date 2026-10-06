# Decline codes

Every decline carries a sentence and a code. The sentence is for a person and
may be reworded between versions. The code is for a program, and never changes:
a code is never renamed and never reused. A new kind of decline gets a new code.
`tests/read.rs` pins the list below.

| Code | Means | Example |
|---|---|---|
| `empty` | Nothing was given. | `""` |
| `no_amount` | A size with no amount. | `kg` |
| `no_unit` | A size with no unit. | `56` |
| `not_one_size` | More than one size in the field. | `500 g 2` |
| `unknown_unit` | A unit mangel does not know. | `56 lbs` |
| `multipack` | A count, a sign and a size: several units in one field. | `4x33cl` |
| `two_readings` | An amount that reads as two numbers. | `1.000 g` (1 g or 1000 g) |
| `zero` | A size of 0. | `0 g` |
| `small_first_letter` | A name whose first letter should be a capital and is not. | `mjölk` |
| `category_is_group` | A category's group, which holds several categories. | `Bageri` |
| `category_ambiguous` | A word in several categories. | `kaffe` |
| `category_unknown` | Not a category. | `Bilar` |
| `vat_must_be_typed` | A category whose VAT is typed per product, with none typed. | `Påsk` with no rate |
| `vat_unknown` | Not a VAT rate the market has. | `7` |
| `vat_mismatch` | A rate that contradicts the category. | `25` for Frukt |
| `deposit_unknown` | Not a deposit the market has. | `1` |
| `no_rule` | mangel has no rule for this field in this profile. | any weight, in the Laundry Room profile, until #18 |

`no_rule` is different from the rest. It says nothing about the value: mangel
did not look at it. A caller should keep the value as it was rather than
treat it as wrong.

Python: `mangel.Declined` has a `code` attribute. TypeScript: the `Declined`
error has a `code` property. Rust: `Declined::code()` returns a `Code`, and
`Code::as_str()` gives the text above.
