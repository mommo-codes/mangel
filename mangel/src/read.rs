//! `read`: one call for every field.
//!
//! A caller says which field a text is, and in what [`Context`]: the market
//! whose register the value goes into, the languages on the label, the
//! language a language-neutral value is named in, and the [`Profile`] it
//! reads for. The profile decides, field by field, what mangel does:
//!
//! - [`Mode::Check`] runs the rule the field already has, unchanged. A value
//!   that is wrong is declined, never altered beyond its spelling.
//! - [`Mode::Off`] means mangel has no rule for the field in this profile.
//!   It declines with [`Code::NoRule`], so a caller can tell "no rule" from
//!   "this value is wrong".
//!
//! Fixing, rather than checking, arrives field by field, each only when it
//! measurably beats what the profile's tool does today. Weight and volume
//! come first (issue #18).

use std::fmt;

use crate::rules::{self, Category, Code, Declined};
use crate::{Language, Market, Unit, Vocabulary};

/// A field of a product record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Field {
    Name,
    Brand,
    CountryOfOrigin,
    Description,
    Ingredients,
    Allergens,
    Additives,
    Producer,
    Distributor,
    AlcoholPercentage,
    Weight,
    Volume,
    Quantity,
    /// The register's one size field, amount and unit together.
    Size,
    Category,
    Vat,
    Deposit,
}

impl Field {
    /// Every field.
    pub const ALL: &'static [Field] = &[
        Field::Name,
        Field::Brand,
        Field::CountryOfOrigin,
        Field::Description,
        Field::Ingredients,
        Field::Allergens,
        Field::Additives,
        Field::Producer,
        Field::Distributor,
        Field::AlcoholPercentage,
        Field::Weight,
        Field::Volume,
        Field::Quantity,
        Field::Size,
        Field::Category,
        Field::Vat,
        Field::Deposit,
    ];

    /// The field's code, `snake_case`: what Python and TypeScript pass.
    pub const fn code(self) -> &'static str {
        match self {
            Field::Name => "name",
            Field::Brand => "brand",
            Field::CountryOfOrigin => "country_of_origin",
            Field::Description => "description",
            Field::Ingredients => "ingredients",
            Field::Allergens => "allergens",
            Field::Additives => "additives",
            Field::Producer => "producer",
            Field::Distributor => "distributor",
            Field::AlcoholPercentage => "alcohol_percentage",
            Field::Weight => "weight",
            Field::Volume => "volume",
            Field::Quantity => "quantity",
            Field::Size => "size",
            Field::Category => "category",
            Field::Vat => "vat",
            Field::Deposit => "deposit",
        }
    }

    /// The field with this code. Exact match only.
    pub fn from_code(code: &str) -> Result<Field, UnknownField> {
        Field::ALL
            .iter()
            .copied()
            .find(|field| field.code() == code)
            .ok_or_else(|| UnknownField(code.to_owned()))
    }
}

/// Who mangel reads for. Each profile is a table from field to [`Mode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Profile {
    /// Name Scrubbing's cleaning sheet: a person typed the value, and mangel
    /// checks it with the rules it has always had.
    NameScrubbing,
    /// Laundry Room: the value is what OCR read off a label. It has no field
    /// yet; each joins with its fixing rule.
    LaundryRoom,
}

impl Profile {
    /// Every profile.
    pub const ALL: &'static [Profile] = &[Profile::NameScrubbing, Profile::LaundryRoom];

    /// The profile's code: what Python and TypeScript pass.
    pub const fn code(self) -> &'static str {
        match self {
            Profile::NameScrubbing => "name_scrubbing",
            Profile::LaundryRoom => "laundry_room",
        }
    }

    /// The profile with this code. Exact match only.
    pub fn from_code(code: &str) -> Result<Profile, UnknownProfile> {
        Profile::ALL
            .iter()
            .copied()
            .find(|profile| profile.code() == code)
            .ok_or_else(|| UnknownProfile(code.to_owned()))
    }

    /// What mangel does with `field` for this profile.
    ///
    /// Changing a row here changes what a tool gets, so every row is pinned
    /// by a test in `tests/read.rs`.
    pub const fn mode(self, field: Field) -> Mode {
        match (self, field) {
            (
                Profile::NameScrubbing,
                Field::Name | Field::Size | Field::Category | Field::Vat | Field::Deposit,
            ) => Mode::Check,
            _ => Mode::Off,
        }
    }
}

/// What a profile does with a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Mode {
    /// The field's existing rule: a wrong value is declined.
    Check,
    /// No rule for the field in this profile.
    Off,
}

/// Where a value comes from and where it goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context<'a> {
    market: Market,
    profile: Profile,
    label: Vec<Language>,
    output: Language,
    category: Option<&'a str>,
}

impl Context<'static> {
    /// A context for `market` and `profile`, with no label languages known
    /// and the market's own language as the output language.
    pub fn new(market: Market, profile: Profile) -> Context<'static> {
        Context {
            market,
            profile,
            label: Vec::new(),
            output: market.language(),
            category: None,
        }
    }
}

impl<'a> Context<'a> {
    /// The languages printed on the label, any number, in any order. A
    /// language given twice counts once.
    pub fn with_label(mut self, languages: &[Language]) -> Context<'a> {
        self.label.clear();
        for language in languages {
            if !self.label.contains(language) {
                self.label.push(*language);
            }
        }
        self
    }

    /// The language a language-neutral value is named in. Only an output
    /// language can be: mangel can read Turkish but not write it.
    pub fn with_output(mut self, language: Language) -> Result<Context<'a>, NotAnOutputLanguage> {
        if !language.is_output() {
            return Err(NotAnOutputLanguage(language));
        }
        self.output = language;
        Ok(self)
    }

    /// The product's category, which a VAT rate is read against.
    pub fn with_category<'b>(self, category: &'b str) -> Context<'b>
    where
        'a: 'b,
    {
        Context {
            category: Some(category),
            ..self
        }
    }

    /// The market.
    pub fn market(&self) -> Market {
        self.market
    }

    /// The product's category, if one was given.
    pub fn category(&self) -> Option<&'a str> {
        self.category
    }

    /// The profile.
    pub fn profile(&self) -> Profile {
        self.profile
    }

    /// The label languages, in the order given.
    pub fn label(&self) -> &[Language] {
        &self.label
    }

    /// The output language.
    pub fn output(&self) -> Language {
        self.output
    }
}

/// A field, read.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Read {
    /// The value as the market writes it: `0,5L` in Sweden.
    pub value: String,
    /// The value without a language or a market's spelling.
    pub neutral: Neutral,
    /// What was changed on the way from the text to `value`, in a fixed
    /// order. Empty when the text was already written as the market writes
    /// it.
    pub changes: Vec<Change>,
}

/// A value without a language or a market's spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Neutral {
    /// Free text, in the language it arrived in.
    Text(String),
    /// A size: the amount as a decimal with a `.` point, and the unit.
    Size { amount: String, unit: Unit },
    /// A category of the market's list.
    Category(Category),
    /// A VAT rate, in percent.
    Rate(u8),
    /// A deposit in the market's currency, or none.
    Deposit(Option<u8>),
}

/// Something `read` changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Change {
    /// Spaces trimmed, collapsed, added or removed.
    Spacing,
    /// The amount written the market's way: `1.50` to `1,5`.
    Amount,
    /// The unit spelled the market's way: `l` to `L`.
    UnitSpelling,
}

impl Change {
    /// The change's code, `snake_case`: what Python and TypeScript see.
    pub const fn code(self) -> &'static str {
        match self {
            Change::Spacing => "spacing",
            Change::Amount => "amount",
            Change::UnitSpelling => "unit_spelling",
        }
    }
}

/// Read `text` as `field`, in `context`.
///
/// ```
/// use mangel::{read, Context, Field, Market, Neutral, Profile, Unit};
///
/// let sheet = Context::new(Market::Se, Profile::NameScrubbing);
/// let read = read(Field::Size, "0,5 l", &sheet).unwrap();
/// assert_eq!(read.value, "0,5L");
/// assert_eq!(read.neutral, Neutral::Size { amount: "0.5".into(), unit: Unit::Litre });
///
/// let labels = Context::new(Market::Se, Profile::LaundryRoom);
/// let declined = mangel::read(Field::Weight, "500 g", &labels).unwrap_err();
/// assert_eq!(declined.code().as_str(), "no_rule");
/// ```
pub fn read(field: Field, text: &str, context: &Context) -> Result<Read, Declined> {
    match context.profile.mode(field) {
        Mode::Check => check(field, text, context),
        Mode::Off => Err(no_rule(field, context.profile)),
    }
}

fn check(field: Field, text: &str, context: &Context) -> Result<Read, Declined> {
    let market = context.market;
    match field {
        Field::Name => {
            let name = rules::cleaned_name(text)?;
            let changes = if name == text {
                Vec::new()
            } else {
                vec![Change::Spacing]
            };
            Ok(Read {
                value: name.clone(),
                neutral: Neutral::Text(name),
                changes,
            })
        }
        Field::Size => size(text, market),
        Field::Category => {
            let found = rules::category(text, market)?;
            Ok(Read {
                value: found.name.to_owned(),
                neutral: Neutral::Category(found),
                changes: Vec::new(),
            })
        }
        Field::Vat => {
            let found = rules::category(context.category.unwrap_or(""), market)?;
            let rate = rules::vat(&found, text, market)?;
            Ok(Read {
                value: rate.to_string(),
                neutral: Neutral::Rate(rate),
                changes: Vec::new(),
            })
        }
        Field::Deposit => {
            let amount = rules::deposit(text, market)?;
            Ok(Read {
                value: amount.map_or_else(String::new, |amount| amount.to_string()),
                neutral: Neutral::Deposit(amount),
                changes: Vec::new(),
            })
        }
        _ => Err(no_rule(field, context.profile)),
    }
}

/// A size, written as the market writes it, with what that changed.
fn size(text: &str, market: Market) -> Result<Read, Declined> {
    let reading = rules::reading(text, market)?;
    let spelled = Vocabulary::of(market).spelling(reading.unit);
    let amount = reading
        .amount
        .replace('.', &market.decimal_separator().to_string());
    let space = if market.space_before_unit() { " " } else { "" };

    let mut changes = Vec::new();
    if text.trim() != text || reading.spaced != market.space_before_unit() {
        changes.push(Change::Spacing);
    }
    if reading.number_written != amount {
        changes.push(Change::Amount);
    }
    if reading.unit_written != spelled {
        changes.push(Change::UnitSpelling);
    }
    Ok(Read {
        value: format!("{amount}{space}{spelled}"),
        neutral: Neutral::Size {
            amount: reading.amount,
            unit: reading.unit,
        },
        changes,
    })
}

fn no_rule(field: Field, profile: Profile) -> Declined {
    Declined::new(
        Code::NoRule,
        format!(
            "mangel has no rule for {} in the {} profile yet",
            field.code(),
            profile.code()
        ),
    )
}

/// A field code mangel does not know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownField(String);

impl fmt::Display for UnknownField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown field {:?}; the fields are:", self.0)?;
        for field in Field::ALL {
            write!(f, " {:?}", field.code())?;
        }
        Ok(())
    }
}

impl std::error::Error for UnknownField {}

/// A profile code mangel does not know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownProfile(String);

impl fmt::Display for UnknownProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown profile {:?}; the profiles are:", self.0)?;
        for profile in Profile::ALL {
            write!(f, " {:?}", profile.code())?;
        }
        Ok(())
    }
}

impl std::error::Error for UnknownProfile {}

/// A language mangel reads but cannot name a value in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotAnOutputLanguage(pub Language);

impl fmt::Display for NotAnOutputLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} is read but not written; the output languages are:",
            self.0.code()
        )?;
        for language in Language::ALL.iter().filter(|language| language.is_output()) {
            write!(f, " {:?}", language.code())?;
        }
        Ok(())
    }
}

impl std::error::Error for NotAnOutputLanguage {}
