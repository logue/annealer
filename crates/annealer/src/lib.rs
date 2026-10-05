//! Reorders HTML and Vue template attributes by semantic meaning, following a
//! swappable YAML profile, and delegates stylesheet property order to `malva`.
//!
//! ```
//! use annealer::{Config, Language, format};
//!
//! let config = Config::for_language(Language::Html);
//! let output = format(r#"<a title="Home" href="/" id="home">Home</a>"#, &config).unwrap();
//! assert_eq!(output, r#"<a id="home" title="Home" href="/">Home</a>"#);
//! ```

mod attribute;
mod directive;
mod error;
mod markup;
mod order;
mod profile;
#[cfg(test)]
mod schema_tests;
mod stylesheet;
#[cfg(feature = "wasm")]
mod wasm;

use std::path::Path;

pub use error::{FormatError, ProfileError};
pub use profile::{GroupSort, Profile};

/// Input language.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Html,
    Vue,
    Css,
    Scss,
    /// Indented Sass syntax (`.sass`).
    Sass,
    Less,
}

impl Language {
    /// Infers the language from a file extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "html" | "htm" => Some(Self::Html),
            "vue" => Some(Self::Vue),
            "css" => Some(Self::Css),
            "scss" => Some(Self::Scss),
            "sass" => Some(Self::Sass),
            "less" => Some(Self::Less),
            _ => None,
        }
    }

    /// Name of the built-in profile used by default for this language.
    pub fn default_profile(self) -> &'static str {
        match self {
            Self::Vue => "vue",
            Self::Html | Self::Css | Self::Scss | Self::Sass | Self::Less => "html",
        }
    }
}

/// Formatting configuration.
#[derive(Clone, Debug)]
pub struct Config {
    pub language: Language,
    pub profile: Profile,
}

impl Config {
    pub fn new(language: Language, profile: Profile) -> Self {
        Self { language, profile }
    }

    /// Uses the built-in profile that matches `language`.
    pub fn for_language(language: Language) -> Self {
        let profile =
            Profile::builtin(language.default_profile()).expect("built-in profile exists");
        Self::new(language, profile)
    }
}

/// Formats `input`. Pure function: no file I/O.
pub fn format(input: &str, config: &Config) -> Result<String, FormatError> {
    let stylesheet = |lang| match &config.profile.stylesheet {
        Some(options) => {
            stylesheet::format(input, lang, options).map_err(|message| FormatError::Stylesheet {
                line: 1,
                column: 1,
                message,
            })
        }
        None => Ok(input.to_owned()),
    };
    match config.language {
        Language::Html => markup::Scanner::format(input, &config.profile, false),
        Language::Vue => markup::Scanner::format(input, &config.profile, true),
        Language::Css => stylesheet(stylesheet::StyleLang::Css),
        Language::Scss => stylesheet(stylesheet::StyleLang::Scss),
        Language::Sass => stylesheet(stylesheet::StyleLang::Sass),
        Language::Less => stylesheet(stylesheet::StyleLang::Less),
    }
}
