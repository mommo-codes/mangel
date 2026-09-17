# mangel

Product data normalisation: irregular fields in, regular fields out. Written
in Rust; this is the Python binding.

**Not yet published, and no parsing rules exist yet.** What the binding
exposes today is the vocabulary compiled into the library.

```python
import mangel

mangel.abbreviations("se")["Laktosfri"]   # 'LF'
mangel.abbreviations("SE")                # ValueError — market codes are exact
```

Every call takes a market. There is no default: one that was right for Sweden
would be silently wrong for the next market.

Full documentation: <https://github.com/mommo-codes/mangel>

MIT licensed.
