from typing import TypedDict

class Declined(ValueError):
    """A field mangel would not read. The message says why, as a sentence a
    person can act on."""

class Category(TypedDict):
    name: str
    """The category — what a product is filed under, and what the register
    stores."""
    group: str
    """The group the category sits in."""
    vat: int | None
    """Its VAT in percent, or ``None`` for a category whose products carry
    more than one rate, so the VAT is typed per product."""

def abbreviations(market: str) -> dict[str, str]:
    """A market's abbreviations, as a new dict: each word as it is written in
    product data, mapped to the abbreviation the golden standard uses. Keys
    are in sorted order.

    Raises ``ValueError`` for a market code mangel has no conventions for.
    The code is exact: ``"se"``, not ``"SE"``.
    """
    ...

def size(text: str, market: str) -> tuple[str, str]:
    """A size read into ``(amount, unit)``: ``"56kg"`` is ``("56", "kg")``,
    ``"0,5 l"`` is ``("0.5", "L")``. The amount is a decimal string; the
    unit is the golden standard's spelling.

    Raises ``Declined`` for no unit, no amount, a multipack, a unit the
    market does not have, or an amount that reads as two numbers:
    ``"1.000 g"`` could be 1 g or 1000 g.
    """
    ...

def cleaned_name(text: str) -> str:
    """A cleaned name, checked: trimmed, inner spacing collapsed, and the
    first word starting with a capital. Raises ``Declined`` otherwise — it
    is not capitalised for you."""
    ...

def category(text: str, market: str) -> Category:
    """The category meant by what was typed: its name, a group holding only
    it, or a keyword that starts or ends a word of exactly one category's
    name. Case is ignored. Raises ``Declined`` for none, or for several —
    naming them."""
    ...

def categories(market: str) -> list[Category]:
    """Every category of the market, sorted by name."""
    ...

def vat(category: str, text: str, market: str) -> int:
    """The VAT for a product in ``category`` given what was typed in its VAT
    field (``"12"``, ``"12%"`` or ``""``). Empty takes the category's rate.
    Raises ``Declined`` when the typed rate contradicts the category, when
    the category's rate must be typed and was not, or when it is not a rate
    the market has."""
    ...

def deposit(text: str, market: str) -> int | None:
    """A deposit: one of the market's amounts (``2``, ``3 kr``, ``2,00``),
    or ``None`` for an empty field. Raises ``Declined`` otherwise."""
    ...
