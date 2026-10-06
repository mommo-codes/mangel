//! Size: `56kg` is 56 and `kg`.
//!
//! One field in, two out — the amount and the unit, the unit in the golden
//! standard's spelling from the market's `units` table. The unit is matched
//! without regard to case; nothing else is loosened. A multipack, a size
//! with no unit, a unit the table does not hold, or an amount that reads
//! as two different numbers (`1.000 g`) is declined.

use super::{listed, Declined};
use crate::{Market, Vocabulary};

/// A size, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Size {
    /// The amount, as a decimal with a `.` point and no padding: `56`,
    /// `0.5`, `1.25`. Text rather than a float, so it is never rounded.
    pub amount: String,
    /// The unit, in the golden standard's spelling: `kg`, `L`, `st`.
    pub unit: &'static str,
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
    let text = text.trim();
    if text.is_empty() {
        return Err(Declined::new("is empty"));
    }
    let units = Vocabulary::of(market).units;
    let example = |amount: &str| format!("write it as {amount} and a unit: {}", unit_list(units));

    let digits = text
        .char_indices()
        .find(|(_, c)| !(c.is_ascii_digit() || *c == ',' || *c == '.'))
        .map_or(text.len(), |(at, _)| at);
    let (number, rest) = text.split_at(digits);
    let rest = rest.trim_start();

    if is_multipack(rest) {
        return Err(Declined::new(format!(
            "{text:?} is a multipack; give the size of one unit, {}",
            example("33 cl")
        )));
    }
    let Some(amount) = amount(number) else {
        return Err(Declined::new(format!(
            "{text:?} has no amount; {}",
            example("500 g")
        )));
    };
    if rest.is_empty() {
        return Err(Declined::new(format!(
            "{text:?} has no unit; {}",
            example(&format!("{amount} g"))
        )));
    }
    if rest
        .chars()
        .any(|c| c.is_whitespace() || c.is_ascii_digit())
    {
        return Err(Declined::new(format!(
            "{text:?} is not one size; {}",
            example("500 g")
        )));
    }
    let wanted = rest.to_lowercase();
    let Some(&(_, unit)) = units.iter().find(|(written, _)| *written == wanted) else {
        return Err(Declined::new(format!(
            "{rest:?} is not a unit; use {}",
            unit_list(units)
        )));
    };
    if let Some((decimal, thousands)) = two_readings(number, unit) {
        return Err(Declined::new(format!(
            "{text:?} could be {decimal} {unit} or {thousands} {unit}; write the one it is"
        )));
    }
    if amount == "0" {
        return Err(Declined::new("a size is more than 0"));
    }
    Ok(Size { amount, unit })
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
fn two_readings(number: &str, unit: &str) -> Option<(String, String)> {
    let at = number.find([',', '.'])?;
    let (whole, rest) = number.split_at(at);
    let (separator, fraction) = rest.split_at(1);
    let opens_a_thousand = (1..=3).contains(&whole.len()) && !whole.starts_with('0');
    if !opens_a_thousand || fraction.len() != 3 {
        return None;
    }
    if separator == "," && matches!(unit, "kg" | "L") {
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

/// The golden spellings, once each, in the order the table first gives them.
fn unit_list(units: &[(&str, &'static str)]) -> String {
    let mut golden: Vec<&str> = Vec::new();
    for (_, unit) in units {
        if !golden.contains(unit) {
            golden.push(unit);
        }
    }
    listed(&golden).replacen(" and ", " or ", 1)
}
