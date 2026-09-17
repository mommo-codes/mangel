def abbreviations(market: str) -> dict[str, str]:
    """A market's abbreviations, as a new dict: each word as it is written in
    product data, mapped to the abbreviation the golden standard uses. Keys
    are in sorted order.

    Raises ``ValueError`` for a market code mangel has no conventions for.
    The code is exact: ``"se"``, not ``"SE"``.
    """
    ...
