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
mod class_order;
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

use std::borrow::Cow;
use std::path::Path;

use profile::SelfClosing;

pub use class_order::{ClassOrder, UnknownPosition};
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
    /// Order of the classes in static `class` attributes. Overrides the
    /// profile's `classOrder`; when both are absent, classes keep their order.
    pub class_order: Option<ClassOrder>,
    /// Turns the profile's `layout.selfClosing` rules on or off. `None` keeps
    /// the profile's setting (off in the built-in profiles).
    pub self_closing: Option<bool>,
    /// Self-closes void elements (`<input />`) when `true`, or drops the slash
    /// (`<input>`) when `false`. `None` keeps the profile's
    /// `layout.selfClosing.void` (`always` in the built-in profiles).
    pub void_self_closing: Option<bool>,
    /// Gives non-void elements written self-closing in plain HTML (`<div />`)
    /// an end tag (`<div></div>`). `None` keeps the profile's
    /// `layout.selfClosing.expandNonVoid` (off in the built-in profiles).
    pub expand_non_void: Option<bool>,
    /// Turns the profile's `layout.contentNewline` rule on or off, with its
    /// defaults when the profile has none. `None` keeps the profile's setting
    /// (off in the built-in profiles).
    pub content_newline: Option<bool>,
    /// Drops repeated classes in static `class` attributes, keeping the first.
    /// Works with or without a class order. `None` keeps the class order's
    /// `removeDuplicates` (off without a class order).
    pub remove_duplicate_classes: Option<bool>,
    /// Line width of stylesheets (`stylesheet.malva.printWidth`), for
    /// stylesheet files and `<style>` blocks. `None` keeps the profile's
    /// setting (80, malva's default, in the built-in profiles).
    pub print_width: Option<usize>,
}

impl Config {
    pub fn new(language: Language, profile: Profile) -> Self {
        Self {
            language,
            profile,
            class_order: None,
            self_closing: None,
            void_self_closing: None,
            expand_non_void: None,
            content_newline: None,
            remove_duplicate_classes: None,
            print_width: None,
        }
    }

    /// Turns `layout.selfClosing` on or off (`vue/html-self-closing`).
    #[must_use]
    pub fn with_self_closing(mut self, enabled: Option<bool>) -> Self {
        self.self_closing = enabled;
        self
    }

    /// Sets `layout.selfClosing.void`: `<input />` (`true`) or `<input>`
    /// (`false`).
    #[must_use]
    pub fn with_void_self_closing(mut self, enabled: Option<bool>) -> Self {
        self.void_self_closing = enabled;
        self
    }

    /// Turns `layout.selfClosing.expandNonVoid` on or off: `<div />` →
    /// `<div></div>` in plain HTML.
    #[must_use]
    pub fn with_expand_non_void(mut self, enabled: Option<bool>) -> Self {
        self.expand_non_void = enabled;
        self
    }

    /// Turns `layout.contentNewline` on or off
    /// (`vue/multiline-html-element-content-newline`).
    #[must_use]
    pub fn with_content_newline(mut self, enabled: Option<bool>) -> Self {
        self.content_newline = enabled;
        self
    }

    /// Turns removal of repeated classes on or off.
    #[must_use]
    pub fn with_remove_duplicate_classes(mut self, enabled: Option<bool>) -> Self {
        self.remove_duplicate_classes = enabled;
        self
    }

    /// Sets the line width of stylesheets (Prettier and Biome: 80, oxfmt: 100).
    #[must_use]
    pub fn with_print_width(mut self, print_width: Option<usize>) -> Self {
        self.print_width = print_width;
        self
    }

    /// The profile with the layout and line width overrides applied.
    fn effective_profile(&self) -> Cow<'_, Profile> {
        if self.self_closing.is_none()
            && self.void_self_closing.is_none()
            && self.expand_non_void.is_none()
            && self.content_newline.is_none()
            && self.print_width.is_none()
        {
            return Cow::Borrowed(&self.profile);
        }
        let mut profile = self.profile.clone();
        profile
            .layout
            .enable(self.self_closing, self.content_newline);
        let rules = &mut profile.layout.self_closing;
        if let Some(enabled) = self.void_self_closing {
            rules.void = if enabled {
                SelfClosing::Always
            } else {
                SelfClosing::Never
            };
        }
        if let Some(enabled) = self.expand_non_void {
            rules.expand_non_void = enabled;
        }
        if let (Some(width), Some(stylesheet)) = (self.print_width, &mut profile.stylesheet) {
            stylesheet.malva.layout.print_width = width;
        }
        Cow::Owned(profile)
    }

    /// Sets the class order (e.g. `ClassOrder::builtin("tailwind")`).
    #[must_use]
    pub fn with_class_order(mut self, class_order: Option<ClassOrder>) -> Self {
        self.class_order = class_order;
        self
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
    let profile = config.effective_profile();
    let stylesheet = |lang| match &profile.stylesheet {
        Some(options) => {
            stylesheet::format(input, lang, options).map_err(|message| FormatError::Stylesheet {
                line: 1,
                column: 1,
                message,
            })
        }
        None => Ok(input.to_owned()),
    };
    let classes = class_order::Classes::new(
        config
            .class_order
            .as_ref()
            .or(config.profile.class_order.as_ref()),
        config.remove_duplicate_classes,
    );
    match config.language {
        Language::Html => markup::Scanner::format(input, &profile, classes, false),
        Language::Vue => markup::Scanner::format(input, &profile, classes, true),
        Language::Css => stylesheet(stylesheet::StyleLang::Css),
        Language::Scss => stylesheet(stylesheet::StyleLang::Scss),
        Language::Sass => stylesheet(stylesheet::StyleLang::Sass),
        Language::Less => stylesheet(stylesheet::StyleLang::Less),
    }
}
