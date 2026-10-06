//! Languages.
//!
//! A market says how a value is written in its register; a language says how
//! a label is read and how a language-neutral value is named. They are
//! separate because a pack sold in one market is often printed in another
//! language, and in several at once.
//!
//! Two kinds:
//!
//! - **Output languages**, the ones a value can be named in. They are the
//!   label languages the consuming OCR returns text in.
//! - **Reading languages**, which mangel can only read: packs on the same
//!   shelves are printed in them often enough that their words matter.
//!
//! Codes are ISO 639-1, lowercase. Norwegian is `nb`, Bokmål, because that
//! is what packaging is printed in; `no` is the macrolanguage.
//!
//! This file is also compiled into `build.rs`, which reads
//! [`Language::ALL`] to learn which vocabulary directories may exist, so it
//! depends on nothing but `std`.

use std::fmt;

/// A language mangel can read a label in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Language {
    /// Swedish.
    Sv,
    /// Danish.
    Da,
    /// Norwegian, Bokmål.
    Nb,
    /// Hungarian.
    Hu,
    /// Croatian.
    Hr,
    /// German.
    De,
    /// Greek.
    El,
    /// Georgian.
    Ka,
    /// English.
    En,
    /// Turkish. Read only.
    Tr,
    /// Italian. Read only.
    It,
    /// French. Read only.
    Fr,
    /// Spanish. Read only.
    Es,
    /// Polish. Read only.
    Pl,
}

impl Language {
    /// Every language, output languages first.
    pub const ALL: &'static [Language] = &[
        Language::Sv,
        Language::Da,
        Language::Nb,
        Language::Hu,
        Language::Hr,
        Language::De,
        Language::El,
        Language::Ka,
        Language::En,
        Language::Tr,
        Language::It,
        Language::Fr,
        Language::Es,
        Language::Pl,
    ];

    /// The language's ISO 639-1 code, lowercase. Also the name of its
    /// directory under `vocabulary/language/`.
    pub const fn code(self) -> &'static str {
        match self {
            Language::Sv => "sv",
            Language::Da => "da",
            Language::Nb => "nb",
            Language::Hu => "hu",
            Language::Hr => "hr",
            Language::De => "de",
            Language::El => "el",
            Language::Ka => "ka",
            Language::En => "en",
            Language::Tr => "tr",
            Language::It => "it",
            Language::Fr => "fr",
            Language::Es => "es",
            Language::Pl => "pl",
        }
    }

    /// The language's name in English, for messages.
    pub const fn name(self) -> &'static str {
        match self {
            Language::Sv => "Swedish",
            Language::Da => "Danish",
            Language::Nb => "Norwegian",
            Language::Hu => "Hungarian",
            Language::Hr => "Croatian",
            Language::De => "German",
            Language::El => "Greek",
            Language::Ka => "Georgian",
            Language::En => "English",
            Language::Tr => "Turkish",
            Language::It => "Italian",
            Language::Fr => "French",
            Language::Es => "Spanish",
            Language::Pl => "Polish",
        }
    }

    /// Whether a value can be named in this language, rather than only read
    /// from it.
    pub const fn is_output(self) -> bool {
        matches!(
            self,
            Language::Sv
                | Language::Da
                | Language::Nb
                | Language::Hu
                | Language::Hr
                | Language::De
                | Language::El
                | Language::Ka
                | Language::En
        )
    }

    /// The language with this code. Exact match only: `"SV"` and `"no"` are
    /// refused rather than guessed at.
    pub fn from_code(code: &str) -> Result<Language, UnknownLanguage> {
        Language::ALL
            .iter()
            .copied()
            .find(|language| language.code() == code)
            .ok_or_else(|| UnknownLanguage(code.to_owned()))
    }
}

/// A language code mangel cannot read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownLanguage(String);

impl UnknownLanguage {
    /// The code that was asked for.
    pub fn code(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UnknownLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown language {:?}; mangel reads:", self.0)?;
        for language in Language::ALL {
            write!(f, " {:?}", language.code())?;
        }
        Ok(())
    }
}

impl std::error::Error for UnknownLanguage {}
