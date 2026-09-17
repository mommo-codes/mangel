"""The binding's own tests.

What the vocabulary says is tested once, in Rust. What is tested here is the
boundary: that the compiled tables arrive intact on the Python side, that a
bad market is refused rather than defaulted, and that a caller cannot reach
back through the result and change what the next caller gets.
"""

import pytest

import mangel


class TestAbbreviations:
    def test_the_compiled_table_crosses_the_boundary(self):
        # "Laktosfri" -> "LF" is the entry the vocabulary was scaffolded with.
        # It stands in for the whole table: if it arrives, the table did.
        assert mangel.abbreviations("se")["Laktosfri"] == "LF"

    def test_it_is_a_plain_dict_of_strings(self):
        table = mangel.abbreviations("se")
        assert type(table) is dict
        assert all(type(k) is str and type(v) is str for k, v in table.items())

    def test_the_core_order_survives(self):
        words = list(mangel.abbreviations("se"))
        assert words == sorted(words, key=lambda w: w.encode("utf-8"))

    def test_each_call_gets_its_own_copy(self):
        mangel.abbreviations("se")["Laktosfri"] = "changed"
        assert mangel.abbreviations("se")["Laktosfri"] == "LF"


class TestMarkets:
    @pytest.mark.parametrize("code", ["SE", " se", "se ", "", "no", "sweden"])
    def test_anything_but_an_exact_code_is_refused(self, code):
        with pytest.raises(ValueError, match="unknown market"):
            mangel.abbreviations(code)

    def test_the_refusal_says_what_would_have_worked(self):
        with pytest.raises(ValueError, match='"se"'):
            mangel.abbreviations("SE")

    def test_a_market_is_required(self):
        with pytest.raises(TypeError):
            mangel.abbreviations()  # type: ignore[call-arg]
