//! Stylesheet property ordering, delegated to `malva`.

use malva::Syntax;
use malva::config::FormatOptions;

/// Stylesheet syntaxes annealer currently formats. Less and Sass (indented
/// syntax) are deferred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StyleLang {
    Css,
    Scss,
}

impl StyleLang {
    /// Maps a `<style lang="…">` value; `None` (no attribute) means CSS.
    pub(crate) fn from_lang(lang: Option<&str>) -> Option<Self> {
        match lang {
            None | Some("css" | "postcss" | "pcss") => Some(Self::Css),
            Some("scss") => Some(Self::Scss),
            Some(_) => None,
        }
    }

    fn syntax(self) -> Syntax {
        match self {
            Self::Css => Syntax::Css,
            Self::Scss => Syntax::Scss,
        }
    }
}

/// Formats a whole stylesheet.
pub(crate) fn format(
    input: &str,
    lang: StyleLang,
    options: &FormatOptions,
) -> Result<String, String> {
    malva::format_text(input, lang.syntax(), options).map_err(|error| error.to_string())
}

/// Formats the content of a `<style>` element, keeping it indented relative to
/// the tag and putting the closing tag on its own line at `base_indent`.
pub(crate) fn format_embedded(
    content: &str,
    lang: StyleLang,
    options: &FormatOptions,
    base_indent: &str,
) -> Result<String, String> {
    let inner_indent = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| &line[..line.len() - line.trim_start_matches([' ', '\t']).len()])
        .min_by_key(|indent| indent.len())
        .unwrap_or("");
    // A style block written on the `<style>` line itself has no usable indent.
    let inner_indent = if content
        .trim_start_matches([' ', '\t'])
        .starts_with(['\n', '\r'])
    {
        inner_indent
    } else {
        base_indent
    };

    let dedented: String = content
        .lines()
        .map(|line| line.strip_prefix(inner_indent).unwrap_or(line.trim_start()))
        .collect::<Vec<_>>()
        .join("\n");
    let formatted = format(&dedented, lang, options)?;

    let newline = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut out = String::with_capacity(formatted.len() + 16);
    out.push_str(newline);
    for line in formatted.lines() {
        if !line.is_empty() {
            out.push_str(inner_indent);
            out.push_str(line);
        }
        out.push_str(newline);
    }
    out.push_str(base_indent);
    Ok(out)
}
