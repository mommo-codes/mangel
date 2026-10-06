"""Product data normalisation: irregular fields in, regular fields out.

One implementation, shared with the TypeScript build and the Rust core. See
https://github.com/mommo-codes/mangel.

Every rule returns what a field means, or raises ``Declined`` with a
sentence a person can act on and a ``code`` a program can act on. Nothing is
guessed.

    >>> import mangel
    >>> mangel.abbreviations("se")["Laktosfri"]
    'LF'
    >>> mangel.size("56kg", "se")
    ('56', 'kg')
    >>> mangel.category("pizza", "se")["name"]
    'Fryst Pizza'
    >>> mangel.read("size", "0,5 l", market="se", profile="name_scrubbing")["value"]
    '0,5L'
"""

from ._mangel import (
    Declined,
    abbreviations,
    categories,
    category,
    cleaned_name,
    deposit,
    fields,
    languages,
    profiles,
    read,
    size,
    vat,
)

__all__ = [
    "Declined",
    "abbreviations",
    "categories",
    "category",
    "cleaned_name",
    "deposit",
    "fields",
    "languages",
    "profiles",
    "read",
    "size",
    "vat",
]
