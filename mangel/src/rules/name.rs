//! Cleaned name: the first word starts with a capital.
//!
//! A check, not a fix — `mellanmjölk` is declined rather than returned as
//! `Mellanmjölk`, because whoever typed it is still looking at the sheet and
//! the rest of the name may be as unfinished as its first letter. A name
//! that starts with a digit (`7UP`) has no first letter to check.

use super::Declined;

/// Check a cleaned name. Returns it trimmed, with its inner spacing
/// collapsed to single spaces.
///
/// ```
/// use mangel::rules::cleaned_name;
///
/// assert_eq!(cleaned_name("  Mellanmjölk  1,5% ").unwrap(), "Mellanmjölk 1,5%");
/// assert!(cleaned_name("mellanmjölk").is_err());
/// assert!(cleaned_name("7UP Zero").is_ok());
/// ```
pub fn cleaned_name(text: &str) -> Result<String, Declined> {
    let name = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let Some(first) = name.chars().next() else {
        return Err(Declined::new("is empty"));
    };
    if first.is_alphabetic() && !first.is_uppercase() {
        let fixed: String = first.to_uppercase().chain(name.chars().skip(1)).collect();
        return Err(Declined::new(format!(
            "{name:?} starts with a small letter; the first word starts with a capital, as in {fixed:?}"
        )));
    }
    Ok(name)
}
