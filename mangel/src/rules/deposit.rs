//! Deposit (pant): one of the market's amounts, or none.
//!
//! `2`, `2 kr`, `2,00` and `2:-` are all 2. Anything that is not one of
//! [`Market::deposits`] is declined — a deposit the register does not know
//! would be charged at the till.

use super::{listed, Declined};
use crate::Market;

/// Read a deposit. An empty field is no deposit.
///
/// ```
/// use mangel::{rules::deposit, Market};
///
/// assert_eq!(deposit("", Market::Se), Ok(None));
/// assert_eq!(deposit("2 kr", Market::Se), Ok(Some(2)));
/// assert!(deposit("1", Market::Se).is_err());
/// ```
pub fn deposit(text: &str, market: Market) -> Result<Option<u8>, Declined> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    let allowed = market.deposits();
    let refuse = || {
        let amounts: Vec<String> = allowed.iter().map(u8::to_string).collect();
        let amounts: Vec<&str> = amounts.iter().map(String::as_str).collect();
        Declined::new(format!(
            "{text:?} is not a deposit; it is {}, or empty for none",
            listed(&amounts).replacen(" and ", " or ", 1)
        ))
    };

    let lowered = text.to_lowercase();
    let number = ["kr", "sek", ":-"]
        .iter()
        .find_map(|suffix| lowered.strip_suffix(suffix))
        .unwrap_or(&lowered)
        .trim();
    let (whole, fraction) = number.split_once([',', '.']).unwrap_or((number, ""));
    if fraction.chars().any(|c| c != '0') {
        return Err(refuse());
    }
    match whole.parse::<u8>() {
        Ok(amount) if allowed.contains(&amount) => Ok(Some(amount)),
        _ => Err(refuse()),
    }
}
