//! Size: `56kg` is 56 and `kg`.
//!
//! One field in, two out — the amount and the unit, the unit in the golden
//! standard's spelling from the market's `units` table. A unit is read as a
//! symbol (`g`, `ML`) in any language, or as a word of the market's language
//! (`gram`, `styck`), without regard to case; nothing else is loosened. A
//! multipack, a size with no unit, a unit the vocabulary does not hold, or an
//! amount that reads as two different numbers (`1.000 g`) is declined.

use super::{listed, Code, Declined};
use crate::{LanguageVocabulary, Market, Unit, Vocabulary};

/// A size, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Size {
    /// The amount, as a decimal with a `.` point and no padding: `56`,
    /// `0.5`, `1.25`. Text rather than a float, so it is never rounded.
    pub amount: String,
    /// The unit, in the golden standard's spelling: `kg`, `L`, `st`.
    pub unit: &'static str,
}

/// A size as `read()` needs it: the unit as a [`Unit`], and the text the
/// amount and unit were written as, so it can say what was changed.
pub(crate) struct Reading {
    pub amount: String,
    pub unit: Unit,
    pub number_written: String,
    pub unit_written: String,
    pub spaced: bool,
}

/// Read a size — an amount and a unit, with or without a space between.
///
/// ```
/// use mangel::{rules::size, Market};
///
/// let read = size("56kg", Market::Se).unwrap();
/// assert_eq!((read.amount.as_str(), read.unit), ("56", "kg"));
/// assert_eq!(size("0,5 l", Market::Se).unwrap().unit, "L");
/// assert!(size("56", Market::Se).is_err());
/// assert!(size("1.000 g", Market::Se).is_err()); // 1 g or 1000 g
/// ```
pub fn size(text: &str, market: Market) -> Result<Size, Declined> {
    let read = reading(text, market)?;
    Ok(Size {
        amount: read.amount,
        unit: Vocabulary::of(market).spelling(read.unit),
    })
}

pub(crate) fn reading(text: &str, market: Market) -> Result<Reading, Declined> {
    let text = text.trim();
    if text.is_empty() {
        return Err(Declined::new(Code::Empty, "is empty"));
    }
    let vocabulary = Vocabulary::of(market);
    let example =
        |amount: &str| format!("write it as {amount} and a unit: {}", unit_list(vocabulary));

    let digits = text
        .char_indices()
        .find(|(_, c)| !(c.is_ascii_digit() || *c == ',' || *c == '.'))
        .map_or(text.len(), |(at, _)| at);
    let (number, after) = text.split_at(digits);
    let rest = after.trim_start();

    if is_multipack(rest) {
        return Err(Declined::new(
            Code::Multipack,
            format!(
                "{text:?} is a multipack; give the size of one unit, {}",
                example("33 cl")
            ),
        ));
    }
    let Some(amount) = amount(number) else {
        return Err(Declined::new(
            Code::NoAmount,
            format!("{text:?} has no amount; {}", example("500 g")),
        ));
    };
    if rest.is_empty() {
        return Err(Declined::new(
            Code::NoUnit,
            format!("{text:?} has no unit; {}", example(&format!("{amount} g"))),
        ));
    }
    if rest
        .chars()
        .any(|c| c.is_whitespace() || c.is_ascii_digit())
    {
        return Err(Declined::new(
            Code::NotOneSize,
            format!("{text:?} is not one size; {}", example("500 g")),
        ));
    }
    let unit =
        Unit::from_symbol(rest).or_else(|| LanguageVocabulary::of(market.language()).unit(rest));
    let Some(unit) = unit else {
        return Err(Declined::new(
            Code::UnknownUnit,
            format!("{rest:?} is not a unit; use {}", unit_list(vocabulary)),
        ));
    };
    if let Some((decimal, thousands)) = two_readings(number, unit) {
        let spelled = vocabulary.spelling(unit);
        return Err(Declined::new(
            Code::TwoReadings,
            format!(
                "{text:?} could be {decimal} {spelled} or {thousands} {spelled}; write the one it is"
            ),
        ));
    }
    if amount == "0" {
        return Err(Declined::new(Code::Zero, "a size is more than 0"));
    }
    Ok(Reading {
        amount,
        unit,
        number_written: number.to_owned(),
        unit_written: rest.to_owned(),
        spaced: after.len() != rest.len(),
    })
}

/// `4x33cl`, `4 x 33 cl`, `4×33`: a count, an x, and a size.
fn is_multipack(rest: &str) -> bool {
    let mut chars = rest.chars();
    matches!(chars.next(), Some('x' | 'X' | '×'))
        && chars
            .as_str()
            .trim_start()
            .starts_with(|c: char| c.is_ascii_digit())
}

/// `056` → `56`, `0,50` → `0.5`, `2.0` → `2`; `None` if it is not one
/// number with at most one decimal point.
fn amount(number: &str) -> Option<String> {
    let number = number.replace(',', ".");
    let (whole, fraction) = number.split_once('.').unwrap_or((&number, ""));
    if whole.is_empty() || fraction.contains('.') {
        return None;
    }
    let whole = whole.trim_start_matches('0');
    let whole = if whole.is_empty() { "0" } else { whole };
    let fraction = fraction.trim_end_matches('0');
    Some(if fraction.is_empty() {
        whole.to_owned()
    } else {
        format!("{whole}.{fraction}")
    })
}

/// `1.000` and `1,500`, read both ways: as a decimal (1, 1.5) and with a
/// thousands separator (1000, 1500). Guessing either is a silent 1000x
/// error, so a number that has both readings is declined. See issue #2.
///
/// It has both when exactly three digits follow the separator and the part
/// before it could open a thousand: one to three digits, not starting with
/// 0. `0,750` and `1,25` have one reading only.
///
/// A point declines in every unit. A comma is a decimal in kg and L, where
/// three decimals are whole grams and millilitres (`1,048kg`, `1,500 L`),
/// and declines in the rest, where a thousands separator is the likelier
/// reading.
///
/// `number` holds at most one separator: `amount` has already refused more.
/// Both readings come back as written, with the separator that was typed.
fn two_readings(number: &str, unit: Unit) -> Option<(String, String)> {
    let at = number.find([',', '.'])?;
    let (whole, rest) = number.split_at(at);
    let (separator, fraction) = rest.split_at(1);
    let opens_a_thousand = (1..=3).contains(&whole.len()) && !whole.starts_with('0');
    if !opens_a_thousand || fraction.len() != 3 {
        return None;
    }
    if separator == "," && matches!(unit, Unit::Kilogram | Unit::Litre) {
        return None;
    }
    let decimals = fraction.trim_end_matches('0');
    let decimal = if decimals.is_empty() {
        whole.to_owned()
    } else {
        format!("{whole}{separator}{decimals}")
    };
    Some((decimal, format!("{whole}{fraction}")))
}

/// The market's spellings, in the order its table gives them.
fn unit_list(vocabulary: &Vocabulary) -> String {
    let spellings: Vec<&str> = vocabulary
        .units
        .iter()
        .map(|(_, spelled)| *spelled)
        .collect();
    listed(&spellings).replacen(" and ", " or ", 1)
}
