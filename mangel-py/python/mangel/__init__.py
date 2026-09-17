"""Product data normalisation: irregular fields in, regular fields out.

One implementation, shared with the TypeScript build and the Rust core. See
https://github.com/mommo-codes/mangel.

    >>> import mangel
    >>> mangel.abbreviations("se")["Laktosfri"]
    'LF'
"""

from ._mangel import abbreviations

__all__ = [
    "abbreviations",
]
