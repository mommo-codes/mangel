//! `read`: profiles, language inputs, decline codes, and the value written
//! the market's way. What each rule decides is tested in `rules.rs`; this
//! tests that `read` reaches the right rule and reports it faithfully.

use mangel::rules::{self, Code};
use mangel::{
    read, Change, Context, Field, Language, LanguageVocabulary, Market, Mode, Neutral, Profile,
    Unit,
};

const SE: Market = Market::Se;

fn sheet() -> Context<'static> {
    Context::new(SE, Profile::NameScrubbing)
}

fn code(field: Field, text: &str, context: &Context) -> &'static str {
    read(field, text, context)
        .expect_err("should be declined")
        .code()
        .as_str()
}

#[test]
fn every_profile_row_is_pinned() {
    let checked = [
        Field::Name,
        Field::Size,
        Field::Category,
        Field::Vat,
        Field::Deposit,
    ];
    for field in Field::ALL {
        let expected = if checked.contains(field) {
            Mode::Check
        } else {
            Mode::Off
        };
        assert_eq!(Profile::NameScrubbing.mode(*field), expected, "{field:?}");
        // Laundry Room gains a field only with a rule that beats what it has
        // today. The first is weight and volume, issue #18.
        assert_eq!(Profile::LaundryRoom.mode(*field), Mode::Off, "{field:?}");
    }
}

#[test]
fn a_field_with_no_rule_says_so_and_is_not_a_wrong_value() {
    let labels = Context::new(SE, Profile::LaundryRoom);
    for field in Field::ALL {
        assert_eq!(code(*field, "500 g", &labels), "no_rule", "{field:?}");
    }
    assert_eq!(code(Field::Brand, "Arla", &sheet()), "no_rule");
    let declined = read(Field::Weight, "500 g", &labels).unwrap_err();
    assert_eq!(
        declined.reason(),
        "mangel has no rule for weight in the laundry_room profile yet"
    );
}

#[test]
fn name_scrubbing_reads_with_the_checks_unchanged() {
    for text in ["56kg", "0,5 l", " 500 G ", "1.50L", "6 st", "2,250 liter"] {
        let read = read(Field::Size, text, &sheet()).unwrap();
        let size = rules::size(text, SE).unwrap();
        let Neutral::Size { amount, unit } = read.neutral else {
            panic!("{text:?}: not a size")
        };
        assert_eq!(amount, size.amount, "{text:?}");
        assert_eq!(
            mangel::Vocabulary::of(SE)
                .units
                .iter()
                .find(|(u, _)| *u == unit)
                .unwrap()
                .1,
            size.unit,
            "{text:?}"
        );
    }
    let name = read(Field::Name, "  Mellanmjölk   1,5% ", &sheet()).unwrap();
    assert_eq!(name.value, "Mellanmjölk 1,5%");
    assert_eq!(name.neutral, Neutral::Text("Mellanmjölk 1,5%".into()));

    let category = read(Field::Category, "pizza", &sheet()).unwrap();
    assert_eq!(category.value, "Fryst Pizza");
    let vat = read(Field::Vat, "", &sheet().with_category("Frukt")).unwrap();
    assert_eq!(
        (vat.value.as_str(), &vat.neutral),
        ("12", &Neutral::Rate(12))
    );
    let deposit = read(Field::Deposit, "3 kr", &sheet()).unwrap();
    assert_eq!(deposit.neutral, Neutral::Deposit(Some(3)));
    assert_eq!(deposit.value, "3");
    assert_eq!(read(Field::Deposit, "", &sheet()).unwrap().value, "");
}

#[test]
fn a_size_is_written_the_markets_way_and_the_changes_are_listed() {
    use Change::{Amount, Spacing, UnitSpelling};
    for (text, value, changes) in [
        ("56kg", "56kg", vec![]),
        ("0,5L", "0,5L", vec![]),
        ("0,5 l", "0,5L", vec![Spacing, UnitSpelling]),
        ("1.50L", "1,5L", vec![Amount]),
        (" 500 G ", "500g", vec![Spacing, UnitSpelling]),
        ("2 Liter", "2L", vec![Spacing, UnitSpelling]),
        ("6 styck", "6st", vec![Spacing, UnitSpelling]),
    ] {
        let read = read(Field::Size, text, &sheet()).unwrap();
        assert_eq!(read.value, value, "{text:?}");
        assert_eq!(read.changes, changes, "{text:?}");
    }
    let read = read(Field::Size, "0,5 l", &sheet()).unwrap();
    assert_eq!(
        read.neutral,
        Neutral::Size {
            amount: "0.5".into(),
            unit: Unit::Litre
        }
    );
}

#[test]
fn every_decline_has_its_code() {
    let vat_for = |category: &'static str| sheet().with_category(category);
    let cases: &[(Field, &str, Context, &str)] = &[
        (Field::Size, " ", sheet(), "empty"),
        (Field::Size, "kg", sheet(), "no_amount"),
        (Field::Size, "56", sheet(), "no_unit"),
        (Field::Size, "500 g 2", sheet(), "not_one_size"),
        (Field::Size, "56 lbs", sheet(), "unknown_unit"),
        (Field::Size, "4x33cl", sheet(), "multipack"),
        (Field::Size, "1.000 g", sheet(), "two_readings"),
        (Field::Size, "0 g", sheet(), "zero"),
        (Field::Name, "mjölk", sheet(), "small_first_letter"),
        (Field::Name, "", sheet(), "empty"),
        (Field::Category, "Bageri", sheet(), "category_is_group"),
        (Field::Category, "kaffe", sheet(), "category_ambiguous"),
        (Field::Category, "Bilar", sheet(), "category_unknown"),
        (Field::Vat, "", vat_for("Påsk"), "vat_must_be_typed"),
        (Field::Vat, "7", vat_for("Frukt"), "vat_unknown"),
        (Field::Vat, "25", vat_for("Frukt"), "vat_mismatch"),
        (Field::Deposit, "1", sheet(), "deposit_unknown"),
    ];
    for (field, text, context, expected) in cases {
        assert_eq!(code(*field, text, context), *expected, "{field:?} {text:?}");
    }
}

#[test]
fn the_codes_never_change() {
    // Renaming a code breaks every caller that branches on it. This list is
    // only ever added to.
    let codes: Vec<&str> = Code::ALL.iter().map(|code| code.as_str()).collect();
    assert_eq!(
        codes,
        [
            "empty",
            "no_amount",
            "no_unit",
            "not_one_size",
            "unknown_unit",
            "multipack",
            "two_readings",
            "zero",
            "small_first_letter",
            "category_is_group",
            "category_ambiguous",
            "category_unknown",
            "vat_must_be_typed",
            "vat_unknown",
            "vat_mismatch",
            "deposit_unknown",
            "no_rule",
        ]
    );
}

#[test]
fn the_output_language_defaults_to_the_markets_and_is_one_mangel_can_write() {
    assert_eq!(sheet().output(), Language::Sv);
    assert_eq!(
        sheet().with_output(Language::Da).unwrap().output(),
        Language::Da
    );
    let refused = sheet().with_output(Language::Tr).unwrap_err();
    assert!(refused
        .to_string()
        .starts_with("\"tr\" is read but not written"));
}

#[test]
fn label_languages_are_any_number_and_count_once() {
    assert!(sheet().label().is_empty());
    let context = sheet().with_label(&[Language::Da, Language::Tr, Language::Da]);
    assert_eq!(context.label(), [Language::Da, Language::Tr]);
}

#[test]
fn codes_round_trip_and_are_exact() {
    for field in Field::ALL {
        assert_eq!(Field::from_code(field.code()), Ok(*field));
    }
    for profile in Profile::ALL {
        assert_eq!(Profile::from_code(profile.code()), Ok(*profile));
    }
    for language in Language::ALL {
        assert_eq!(Language::from_code(language.code()), Ok(*language));
    }
    assert!(Field::from_code("Weight").is_err());
    assert!(Profile::from_code("laundry room").is_err());
    assert!(Language::from_code("no").is_err()); // Norwegian is nb
}

#[test]
fn the_output_languages_are_the_nine_the_ocr_returns() {
    let output: Vec<&str> = Language::ALL
        .iter()
        .filter(|language| language.is_output())
        .map(|language| language.code())
        .collect();
    assert_eq!(
        output,
        ["sv", "da", "nb", "hu", "hr", "de", "el", "ka", "en"]
    );
}

#[test]
fn a_language_has_only_the_words_written_for_it() {
    assert!(LanguageVocabulary::of(Language::Sv)
        .units
        .contains(&("styck", Unit::Piece)));
    for language in Language::ALL {
        if *language != Language::Sv {
            assert!(
                LanguageVocabulary::of(*language).units.is_empty(),
                "{language:?} has unit words nobody has reviewed"
            );
        }
    }
}
