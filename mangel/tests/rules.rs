//! The rules the cleaning sheet reads with, and the vocabulary they lean on.

use mangel::rules::{
    categories, category, cleaned_name, deposit, size, vat, Category, CategoryVat,
};
use mangel::{Market, Vocabulary};

const SE: Market = Market::Se;

fn read_size(text: &str) -> (String, &'static str) {
    let read = size(text, SE).unwrap_or_else(|d| panic!("{text:?}: {d}"));
    (read.amount, read.unit)
}

fn declined<T: std::fmt::Debug>(answer: Result<T, mangel::rules::Declined>) -> String {
    answer.expect_err("should be declined").reason().to_owned()
}

#[test]
fn a_size_is_an_amount_and_a_unit() {
    assert_eq!(read_size("56kg"), ("56".into(), "kg"));
    assert_eq!(read_size("56 kg"), ("56".into(), "kg"));
    assert_eq!(read_size(" 500 G "), ("500".into(), "g"));
    assert_eq!(read_size("0,5 l"), ("0.5".into(), "L"));
    assert_eq!(read_size("1.50L"), ("1.5".into(), "L"));
    assert_eq!(read_size("33cl"), ("33".into(), "cl"));
    assert_eq!(read_size("2.0 Liter"), ("2".into(), "L"));
    assert_eq!(read_size("6 st"), ("6".into(), "st"));
}

#[test]
fn a_size_is_not_guessed() {
    assert!(declined(size("56", SE)).contains("no unit"));
    assert!(declined(size("kg", SE)).contains("no amount"));
    assert!(declined(size("56 lbs", SE)).contains("\"lbs\" is not a unit"));
    assert!(declined(size("4x33cl", SE)).contains("multipack"));
    assert!(declined(size("4 x 33 cl", SE)).contains("multipack"));
    assert!(declined(size("0 g", SE)).contains("more than 0"));
    assert!(declined(size("1.2.3 g", SE)).contains("no amount"));
    assert!(declined(size("500 g 2", SE)).contains("not one size"));
    assert_eq!(declined(size("  ", SE)), "is empty");
}

#[test]
fn an_amount_that_reads_two_ways_is_declined() {
    // A point and three digits is a decimal in some conventions and a
    // thousands separator in others: declined in every unit.
    for (text, readings) in [
        ("1.000 g", "could be 1 g or 1000 g"),
        ("1.000 kg", "could be 1 kg or 1000 kg"),
        ("1.500 L", "could be 1.5 L or 1500 L"),
        ("250.000 ml", "could be 250 ml or 250000 ml"),
        ("3.000st", "could be 3 st or 3000 st"),
    ] {
        let reason = declined(size(text, SE));
        assert!(reason.contains(readings), "{text:?}: {reason}");
    }
    // A comma and three digits, outside kg and L, where a thousands
    // separator is the likelier reading.
    for (text, readings) in [
        ("1,000 g", "could be 1 g or 1000 g"),
        ("1,500 ml", "could be 1,5 ml or 1500 ml"),
        ("2,250 cl", "could be 2,25 cl or 2250 cl"),
        ("1,000 dl", "could be 1 dl or 1000 dl"),
    ] {
        let reason = declined(size(text, SE));
        assert!(reason.contains(readings), "{text:?}: {reason}");
    }
}

#[test]
fn three_decimals_after_a_comma_are_whole_grams_in_kg_and_l() {
    assert_eq!(read_size("1,048kg"), ("1.048".into(), "kg"));
    assert_eq!(read_size("1,000 kg"), ("1".into(), "kg"));
    assert_eq!(read_size("1,500 L"), ("1.5".into(), "L"));
    assert_eq!(read_size("2,250 liter"), ("2.25".into(), "L"));
}

#[test]
fn an_amount_with_one_reading_is_still_read() {
    // Nothing opens a thousand with 0, so these are decimals. 0,75 L is a
    // wine bottle, with either separator.
    assert_eq!(read_size("0,750 L"), ("0.75".into(), "L"));
    assert_eq!(read_size("0.750 L"), ("0.75".into(), "L"));
    assert_eq!(read_size("0.250 kg"), ("0.25".into(), "kg"));
    assert_eq!(read_size("0,125 g"), ("0.125".into(), "g"));
    // Two decimals is no thousands separator.
    assert_eq!(read_size("1,25 kg"), ("1.25".into(), "kg"));
    assert_eq!(read_size("1.50L"), ("1.5".into(), "L"));
    assert_eq!(read_size("1000 g"), ("1000".into(), "g"));
}

#[test]
fn the_units_are_the_registers_spellings() {
    let golden = ["g", "kg", "ml", "cl", "dl", "L", "st"];
    for (written, unit) in Vocabulary::of(SE).units {
        assert_eq!(
            *written,
            written.to_lowercase(),
            "{written:?} is not lowercase"
        );
        assert!(
            golden.contains(unit),
            "{unit:?} is not a spelling the register takes"
        );
    }
}

#[test]
fn a_cleaned_name_starts_with_a_capital() {
    assert_eq!(cleaned_name("Mellanmjölk").unwrap(), "Mellanmjölk");
    assert_eq!(cleaned_name("  Arla   Mjölk ").unwrap(), "Arla Mjölk");
    assert_eq!(cleaned_name("Ägg 12-pack").unwrap(), "Ägg 12-pack");
    assert_eq!(cleaned_name("7UP").unwrap(), "7UP");
    let reason = declined(cleaned_name("ägg 12-pack"));
    assert!(reason.contains("\"Ägg 12-pack\""), "{reason}");
    assert_eq!(declined(cleaned_name("")), "is empty");
}

#[test]
fn a_category_is_found_by_name_group_or_one_keyword() {
    let found = |text: &str| category(text, SE).unwrap_or_else(|d| panic!("{text:?}: {d}"));
    assert_eq!(found("Frukt").group, "Frukt & Bär");
    assert_eq!(found("  mjölk ").name, "Mjölk"); // the name beats the keyword
    assert_eq!(found("Is").name, "Is"); // two letters, but a name
    assert_eq!(found("Receptfria Läkemedel").name, "Receptfria Läkemedel");
    assert_eq!(found("Blommor & Tillbehör").group, "Blommor & Tillbehör"); // a group of one
    assert_eq!(found("pizza").name, "Fryst Pizza");
    assert_eq!(found("OSTBÅGAR").name, "Ostbågar"); // case, not spelling
}

#[test]
fn a_category_that_could_be_two_is_declined_with_them() {
    let reason = declined(category("kaffe", SE));
    assert!(reason.contains("7 categories"), "{reason}");
    assert!(reason.contains("Malet Kaffe"), "{reason}");
    assert!(
        reason.contains("Snabbkaffe") || reason.contains("1 more"),
        "{reason}"
    );
    assert!(!reason.contains("Skafferi"), "{reason}");
    let reason = declined(category("Bageri", SE));
    assert!(reason.contains("is a group"), "{reason}");
    assert!(declined(category("Nikotin & Tobak", SE)).contains("Snus"));
    assert!(declined(category("Bilar", SE)).contains("is not a category"));
}

#[test]
fn vat_follows_the_category() {
    let of = |name: &str| category(name, SE).unwrap();
    assert_eq!(vat(&of("Frukt"), "", SE), Ok(12));
    assert_eq!(vat(&of("Frukt"), "12%", SE), Ok(12));
    assert_eq!(vat(&of("Öl & Vin"), "", SE), Ok(25));
    let reason = declined(vat(&of("Frukt"), "25", SE));
    assert!(
        reason.contains("does not match Frukt, which is 12%"),
        "{reason}"
    );
    assert!(declined(vat(&of("Frukt"), "7", SE)).contains("is not a VAT rate"));
}

#[test]
fn a_mixed_category_has_its_vat_typed() {
    let easter = category("Påsk", SE).unwrap();
    assert_eq!(easter.vat, CategoryVat::Manual);
    assert!(declined(vat(&easter, "", SE)).contains("type this one's"));
    assert_eq!(vat(&easter, "6", SE), Ok(6));
    assert_eq!(category("Jul & Nyår", SE).unwrap().vat, CategoryVat::Manual);
}

#[test]
fn the_two_category_tables_name_the_same_categories() {
    let vocabulary = Vocabulary::of(SE);
    let names: Vec<&str> = vocabulary.categories.iter().map(|(n, _)| *n).collect();
    let rated: Vec<&str> = vocabulary.category_vat.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, rated);
    assert_eq!(names.len(), 180);
    for (name, rate) in vocabulary.category_vat {
        let ok = *rate == "manual" || rate.parse().is_ok_and(|r| SE.vat_rates().contains(&r));
        assert!(ok, "{name}: {rate:?} is neither a rate nor \"manual\"");
    }
    let all: Vec<Category> = categories(SE);
    assert_eq!(all.len(), 180);
    assert_eq!(
        all.iter().filter(|c| c.vat == CategoryVat::Manual).count(),
        2
    );
}

#[test]
fn a_deposit_is_one_the_market_has() {
    assert_eq!(deposit("", SE), Ok(None));
    assert_eq!(deposit("2", SE), Ok(Some(2)));
    assert_eq!(deposit("3 kr", SE), Ok(Some(3)));
    assert_eq!(deposit("2,00", SE), Ok(Some(2)));
    assert_eq!(deposit("3:-", SE), Ok(Some(3)));
    assert!(declined(deposit("1", SE)).contains("it is 2 or 3"));
    assert!(declined(deposit("2,50", SE)).contains("not a deposit"));
    assert!(declined(deposit("pant", SE)).contains("not a deposit"));
}
