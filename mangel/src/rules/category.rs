//! Category, and the VAT that follows from it.
//!
//! A category is found by its name, by its group when the group holds only
//! one, or by a keyword that starts or ends a word of exactly one category's
//! name — `pizza` is Fryst Pizza. A keyword in several is declined with them
//! listed, never picked. Case is ignored; spelling is not.
//!
//! VAT is the category's. It is typed only where the category cannot say
//! (`"manual"` in `category_vat` — a seasonal range mixing rates), and a
//! typed rate that contradicts the category is declined, not corrected: the
//! product or the category is wrong, and only a person knows which.

use super::{listed, Code, Declined};
use crate::{Market, Vocabulary};

/// The VAT a category carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryVat {
    /// One rate, in percent, for everything in the category.
    Rate(u8),
    /// More than one rate across the category; typed per product.
    Manual,
}

/// A category of the market's list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Category {
    /// The category — what a product is filed under.
    pub name: &'static str,
    /// The group the category sits in.
    pub group: &'static str,
    /// Its VAT.
    pub vat: CategoryVat,
}

/// Every category of the market, sorted by name.
pub fn categories(market: Market) -> Vec<Category> {
    let vocabulary = Vocabulary::of(market);
    vocabulary
        .categories
        .iter()
        .map(|&(name, group)| Category {
            name,
            group,
            vat: vat_of(vocabulary, name),
        })
        .collect()
}

fn vat_of(vocabulary: &Vocabulary, name: &str) -> CategoryVat {
    // The two tables name the same categories; `tests/rules.rs` holds them
    // to it, so a missing or unreadable rate is a vocabulary bug.
    match vocabulary.category_vat.iter().find(|(n, _)| *n == name) {
        Some((_, rate)) => rate.parse().map_or(CategoryVat::Manual, CategoryVat::Rate),
        None => CategoryVat::Manual,
    }
}

/// Find a category from what was typed: its name, its group's, or a word
/// in its name.
///
/// ```
/// use mangel::{rules::category, Market};
///
/// assert_eq!(category("frukt", Market::Se).unwrap().name, "Frukt");
/// assert_eq!(category("pizza", Market::Se).unwrap().name, "Fryst Pizza");
/// assert!(category("kaffe", Market::Se).is_err()); // in seven categories
/// ```
pub fn category(text: &str, market: Market) -> Result<Category, Declined> {
    let text = text.trim();
    if text.is_empty() {
        return Err(Declined::new(Code::Empty, "is empty"));
    }
    let wanted = text.to_lowercase();
    let all = categories(market);

    if let Some(found) = all.iter().find(|c| c.name.to_lowercase() == wanted) {
        return Ok(*found);
    }
    let in_group: Vec<&Category> = all
        .iter()
        .filter(|c| c.group.to_lowercase() == wanted)
        .collect();
    match in_group.as_slice() {
        [only] => return Ok(**only),
        [] => {}
        many => {
            return Err(Declined::new(
                Code::CategoryIsGroup,
                format!(
                    "{text:?} is a group; choose one of its categories: {}",
                    names(many)
                ),
            ))
        }
    }
    let matching: Vec<&Category> = all
        .iter()
        .filter(|c| has_word(&c.name.to_lowercase(), &wanted))
        .collect();
    match matching.as_slice() {
        [only] => Ok(**only),
        [] => Err(Declined::new(
            Code::CategoryUnknown,
            format!("{text:?} is not a category"),
        )),
        many => Err(Declined::new(
            Code::CategoryAmbiguous,
            format!(
                "{text:?} is in {} categories; choose one: {}",
                many.len(),
                names(many)
            ),
        )),
    }
}

/// The VAT for a product in `category`, given what was typed in its VAT
/// field (`12`, `12%`, or empty).
///
/// ```
/// use mangel::{rules::{category, vat}, Market};
///
/// let frukt = category("Frukt", Market::Se).unwrap();
/// assert_eq!(vat(&frukt, "", Market::Se), Ok(12));
/// assert!(vat(&frukt, "25", Market::Se).is_err()); // contradicts Frukt
///
/// let easter = category("Påsk", Market::Se).unwrap();
/// assert!(vat(&easter, "", Market::Se).is_err()); // has to be typed
/// assert_eq!(vat(&easter, "25%", Market::Se), Ok(25));
/// ```
pub fn vat(category: &Category, text: &str, market: Market) -> Result<u8, Declined> {
    let rates = market.vat_rates();
    let rate_list = || {
        let rates: Vec<String> = rates.iter().map(u8::to_string).collect();
        let rates: Vec<&str> = rates.iter().map(String::as_str).collect();
        listed(&rates).replacen(" and ", " or ", 1)
    };
    let text = text.trim();
    let typed = text.strip_suffix('%').unwrap_or(text).trim();

    if typed.is_empty() {
        return match category.vat {
            CategoryVat::Rate(rate) => Ok(rate),
            CategoryVat::Manual => Err(Declined::new(
                Code::VatMustBeTyped,
                format!(
                    "{} holds products at more than one rate; type this one's: {}",
                    category.name,
                    rate_list()
                ),
            )),
        };
    }
    let rate = match typed.parse::<u8>() {
        Ok(rate) if rates.contains(&rate) => rate,
        _ => {
            return Err(Declined::new(
                Code::VatUnknown,
                format!("{text:?} is not a VAT rate; it is {}", rate_list()),
            ))
        }
    };
    match category.vat {
        CategoryVat::Rate(expected) if expected != rate => Err(Declined::new(
            Code::VatMismatch,
            format!(
                "{rate}% does not match {}, which is {expected}%; fix the VAT or the category",
                category.name
            ),
        )),
        _ => Ok(rate),
    }
}

/// Whether `keyword` starts or ends a word of `name` — or, for a keyword of
/// several words, the name holds it at word boundaries. Not anywhere inside
/// a word: `kaffe` is in Snabbkaffe (a compound ends with its head word) but
/// not in Skafferi, which merely contains the letters.
fn has_word(name: &str, keyword: &str) -> bool {
    if keyword.contains(|c: char| !c.is_alphanumeric()) {
        return name.split(" & ").any(|part| part == keyword) || name.starts_with(keyword);
    }
    name.split(|c: char| !c.is_alphanumeric())
        .any(|word| word.starts_with(keyword) || word.ends_with(keyword))
}

/// Up to six names, then how many more.
fn names(categories: &[&Category]) -> String {
    let mut shown: Vec<&str> = categories.iter().take(6).map(|c| c.name).collect();
    let more = categories.len().saturating_sub(shown.len());
    let tail = format!("{more} more");
    if more > 0 {
        shown.push(&tail);
    }
    listed(&shown)
}
