//! Stylesheet property ordering, delegated to `malva`.

use malva::Syntax;
use malva::config::DeclarationOrderGroupBy;
use raffia::ParserBuilder;
use raffia::ast::Stylesheet;
use serde_json::Value;

use crate::profile::{StylesheetOptions, VendorPrefix};

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
    options: &StylesheetOptions,
) -> Result<String, String> {
    let formatted = malva::format_text(input, lang.syntax(), &options.malva)
        .map_err(|error| error.to_string())?;
    let language = &options.malva.language;
    if options.vendor_prefix == VendorPrefix::Start && language.declaration_order.is_some() {
        let split_on_empty_line = matches!(
            language.declaration_order_group_by,
            DeclarationOrderGroupBy::NonDeclarationAndEmptyLine
        );
        Ok(move_vendor_prefixed_first(
            &formatted,
            lang.syntax(),
            split_on_empty_line,
        ))
    } else {
        Ok(formatted)
    }
}

/// Formats the content of a `<style>` element, keeping it indented relative to
/// the tag and putting the closing tag on its own line at `base_indent`.
pub(crate) fn format_embedded(
    content: &str,
    lang: StyleLang,
    options: &StylesheetOptions,
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

/// One declaration in malva's output.
struct Declaration<'a> {
    start: usize,
    end: usize,
    name: &'a str,
}

/// Moves vendor-prefixed declarations before the other declarations of each
/// run, sorted alphabetically. A run is what malva sorts as a unit:
/// consecutive declarations, optionally split by empty lines.
///
/// Works on malva's output so malva's own order is kept for everything else.
/// The AST is walked as JSON because raffia has no visitor and blocks appear
/// in many node types.
fn move_vendor_prefixed_first(css: &str, syntax: Syntax, split_on_empty_line: bool) -> String {
    let mut parser = ParserBuilder::new(css).syntax(syntax).build();
    let Ok(stylesheet) = parser.parse::<Stylesheet>() else {
        return css.to_owned();
    };
    let Ok(ast) = serde_json::to_value(&stylesheet) else {
        return css.to_owned();
    };

    let mut edits = Vec::new();
    collect_edits(&ast, css, split_on_empty_line, &mut edits);
    edits.sort_by_key(|&(start, _, _)| start);

    let mut out = String::with_capacity(css.len());
    let mut copied = 0;
    for (start, end, text) in edits {
        out.push_str(&css[copied..start]);
        out.push_str(&text);
        copied = end;
    }
    out.push_str(&css[copied..]);
    out
}

fn collect_edits(
    node: &Value,
    css: &str,
    split_on_empty_line: bool,
    edits: &mut Vec<(usize, usize, String)>,
) {
    match node {
        Value::Object(map) => {
            if let Some(Value::Array(statements)) = map.get("statements") {
                let mut run: Vec<Declaration<'_>> = Vec::new();
                for statement in statements {
                    let declaration = literal_declaration(statement, css);
                    let continues = declaration.as_ref().is_some_and(|next| {
                        run.last().is_none_or(|previous| {
                            !split_on_empty_line
                                || css[previous.end..next.start].matches('\n').count() < 2
                        })
                    });
                    if !continues {
                        edits.extend(reorder_run(&run, css));
                        run.clear();
                    }
                    run.extend(declaration);
                }
                edits.extend(reorder_run(&run, css));
            }
            for value in map.values() {
                collect_edits(value, css, split_on_empty_line, edits);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_edits(item, css, split_on_empty_line, edits);
            }
        }
        _ => {}
    }
}

fn literal_declaration<'a>(statement: &Value, css: &'a str) -> Option<Declaration<'a>> {
    if statement["type"] != "Declaration" || statement["name"]["type"] != "Ident" {
        return None;
    }
    let offset = |node: &Value, key: &str| node["span"][key].as_u64().map(|n| n as usize);
    let name = &statement["name"];
    Some(Declaration {
        start: offset(statement, "start")?,
        end: offset(statement, "end")?,
        name: css.get(offset(name, "start")?..offset(name, "end")?)?,
    })
}

/// `-webkit-foo`, `-moz-foo`, … but not custom properties (`--foo`).
fn is_vendor_prefixed(name: &str) -> bool {
    name.strip_prefix('-')
        .and_then(|rest| rest.split_once('-'))
        .is_some_and(|(vendor, property)| {
            !vendor.is_empty()
                && vendor.chars().all(|c| c.is_ascii_alphanumeric())
                && !property.is_empty()
        })
}

fn reorder_run(run: &[Declaration<'_>], css: &str) -> Option<(usize, usize, String)> {
    let separators: Vec<&str> = run
        .windows(2)
        .map(|pair| &css[pair[0].end..pair[1].start])
        .collect();
    // Comments between declarations belong to their neighbors; nested blocks
    // (Sass nested properties) are handled on their own. Leave both alone.
    if separators
        .iter()
        .any(|separator| separator.contains("/*") || separator.contains("//"))
        || run
            .iter()
            .any(|declaration| css[declaration.start..declaration.end].contains('{'))
    {
        return None;
    }

    let (mut vendor, standard): (Vec<usize>, Vec<usize>) =
        (0..run.len()).partition(|&i| is_vendor_prefixed(run[i].name));
    vendor.sort_by_cached_key(|&i| run[i].name.to_ascii_lowercase());
    let order: Vec<usize> = vendor.into_iter().chain(standard).collect();
    if order.iter().enumerate().all(|(position, &i)| position == i) {
        return None;
    }

    let mut text = String::new();
    for (position, &i) in order.iter().enumerate() {
        if position > 0 {
            text.push_str(separators[position - 1]);
        }
        text.push_str(&css[run[i].start..run[i].end]);
    }
    Some((run[0].start, run[run.len() - 1].end, text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_vendor_prefixes() {
        assert!(is_vendor_prefixed("-webkit-transition"));
        assert!(is_vendor_prefixed("-moz-appearance"));
        assert!(is_vendor_prefixed("-ms-overflow-style"));
        assert!(!is_vendor_prefixed("--custom"));
        assert!(!is_vendor_prefixed("transition"));
        assert!(!is_vendor_prefixed("-webkit-"));
    }
}
