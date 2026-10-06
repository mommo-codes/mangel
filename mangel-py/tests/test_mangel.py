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


class TestRules:
    """The sheet's rules cross the boundary with their answers and their
    reasons intact. What each rule decides is tested once, in Rust."""

    def test_a_size_is_a_pair(self):
        assert mangel.size("56kg", "se") == ("56", "kg")
        assert mangel.size("0,5 l", "se") == ("0.5", "L")

    def test_a_decline_is_a_value_error_with_the_reason(self):
        with pytest.raises(mangel.Declined, match="no unit"):
            mangel.size("56", "se")
        assert issubclass(mangel.Declined, ValueError)

    def test_a_category_is_a_plain_dict(self):
        found = mangel.category("pizza", "se")
        assert found == {"name": "Fryst Pizza", "group": "Fryst", "vat": 12}
        assert mangel.category("Påsk", "se")["vat"] is None

    def test_every_category_is_listed(self):
        listed = mangel.categories("se")
        assert len(listed) == 180
        assert {"name": "Frukt", "group": "Frukt & Bär", "vat": 12} in listed

    def test_vat_follows_the_category_and_refuses_a_clash(self):
        assert mangel.vat("Frukt", "", "se") == 12
        with pytest.raises(mangel.Declined, match="does not match Frukt"):
            mangel.vat("Frukt", "25", "se")

    def test_the_rest(self):
        assert mangel.cleaned_name(" Mellanmjölk ") == "Mellanmjölk"
        assert mangel.deposit("", "se") is None
        assert mangel.deposit("3 kr", "se") == 3
        with pytest.raises(mangel.Declined):
            mangel.cleaned_name("mjölk")

    def test_a_bad_market_is_not_a_decline(self):
        with pytest.raises(ValueError) as raised:
            mangel.size("56kg", "SE")
        assert not isinstance(raised.value, mangel.Declined)


class TestRead:
    """read() crosses the boundary with its value, its neutral value, its
    changes and its codes. What it decides is tested once, in Rust."""

    def test_a_read_is_a_plain_dict(self):
        read = mangel.read("size", "0,5 l", market="se", profile="name_scrubbing")
        assert read == {
            "value": "0,5L",
            "neutral": {"kind": "size", "amount": "0.5", "unit": "l"},
            "changes": ["spacing", "unit_spelling"],
        }

    def test_every_decline_carries_its_code(self):
        with pytest.raises(mangel.Declined) as raised:
            mangel.read("size", "56", market="se", profile="name_scrubbing")
        assert raised.value.code == "no_unit"
        with pytest.raises(mangel.Declined) as raised:
            mangel.read("weight", "500 g", market="se", profile="laundry_room")
        assert raised.value.code == "no_rule"
        # The existing functions carry the same codes.
        with pytest.raises(mangel.Declined) as raised:
            mangel.size("1.000 g", "se")
        assert raised.value.code == "two_readings"

    def test_vat_is_read_against_the_category(self):
        read = mangel.read(
            "vat", "", market="se", profile="name_scrubbing", category="Frukt"
        )
        assert read["neutral"] == {"kind": "rate", "rate": 12}

    def test_language_inputs_are_checked(self):
        mangel.read(
            "name",
            "Mjölk",
            market="se",
            profile="name_scrubbing",
            label=["da", "tr"],
            output="da",
        )
        with pytest.raises(ValueError, match="is read but not written") as raised:
            mangel.read("name", "Mjölk", market="se", profile="name_scrubbing", output="tr")
        assert not isinstance(raised.value, mangel.Declined)
        with pytest.raises(ValueError, match="unknown language"):
            mangel.read("name", "Mjölk", market="se", profile="name_scrubbing", label=["no"])

    @pytest.mark.parametrize(
        "kwargs",
        [
            {"market": "SE", "profile": "name_scrubbing"},
            {"market": "se", "profile": "Name Scrubbing"},
        ],
    )
    def test_a_bad_code_is_not_a_decline(self, kwargs):
        with pytest.raises(ValueError) as raised:
            mangel.read("name", "Mjölk", **kwargs)
        assert not isinstance(raised.value, mangel.Declined)

    def test_the_lists_come_from_the_core(self):
        assert "weight" in mangel.fields()
        assert mangel.profiles() == ["name_scrubbing", "laundry_room"]
        output = [l["code"] for l in mangel.languages() if l["output"]]
        assert output == ["sv", "da", "nb", "hu", "hr", "de", "el", "ka", "en"]
