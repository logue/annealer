//! Stylesheet property ordering, delegated to `malva`.

use malva::Syntax;
use malva::config::{DeclarationOrderGroupBy, Quotes};
use raffia::ParserBuilder;
use raffia::ast::Stylesheet;
use serde_json::Value;

use crate::directive::{self, Disabled};
use crate::profile::{StylesheetOptions, VendorPrefix};

/// Internal at-rule inserted around disabled declarations so malva's sorting
/// can't move them (a non-declaration statement ends a sort run). Removed
/// from the output.
const FENCE: &str = "@annealer-fence-5d1c;";
/// Internal comment directive telling malva to print the next statement
/// verbatim. Removed from the output.
const VERBATIM: &str = "annealer-verbatim-5d1c";

/// Stylesheet syntaxes annealer formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StyleLang {
    Css,
    Scss,
    /// Indented Sass syntax.
    Sass,
    Less,
}

impl StyleLang {
    /// Maps a `<style lang="…">` value; `None` (no attribute) means CSS.
    pub(crate) fn from_lang(lang: Option<&str>) -> Option<Self> {
        match lang {
            None | Some("css" | "postcss" | "pcss") => Some(Self::Css),
            Some("scss") => Some(Self::Scss),
            Some("sass") => Some(Self::Sass),
            Some("less") => Some(Self::Less),
            Some(_) => None,
        }
    }

    fn syntax(self) -> Syntax {
        match self {
            Self::Css => Syntax::Css,
            Self::Scss => Syntax::Scss,
            Self::Sass => Syntax::Sass,
            Self::Less => Syntax::Less,
        }
    }
}

/// Formats a whole stylesheet.
pub(crate) fn format(
    input: &str,
    lang: StyleLang,
    options: &StylesheetOptions,
) -> Result<String, String> {
    let protected = match protect_disabled(input, lang.syntax())? {
        Protection::None => None,
        Protection::All => return Ok(input.to_owned()),
        Protection::Marked(marked) => Some(marked),
    };
    let formatted = match &protected {
        None => malva::format_text(input, lang.syntax(), &options.malva),
        Some(marked) => {
            let mut malva_options = options.malva.clone();
            malva_options.language.ignore_comment_directive = VERBATIM.to_owned();
            malva::format_text(marked, lang.syntax(), &malva_options)
        }
    }
    .map_err(|error| error.to_string())?;
    let formatted = sort_vendor_prefixes(formatted, lang, options);
    Ok(if protected.is_some() {
        remove_markers(&formatted)
    } else {
        formatted
    })
}

fn sort_vendor_prefixes(formatted: String, lang: StyleLang, options: &StylesheetOptions) -> String {
    let language = &options.malva.language;
    if options.vendor_prefix == VendorPrefix::Start && language.declaration_order.is_some() {
        let split_on_empty_line = matches!(
            language.declaration_order_group_by,
            DeclarationOrderGroupBy::NonDeclarationAndEmptyLine
        );
        move_vendor_prefixed_first(&formatted, lang.syntax(), split_on_empty_line)
    } else {
        formatted
    }
}

enum Protection {
    /// No `annealer-disable*` directive applies.
    None,
    /// Everything is disabled; return the input unchanged.
    All,
    /// The input with markers around disabled statements.
    Marked(String),
}

/// Applies `/* annealer-disable* */` directives: marks every outermost
/// statement they cover so malva prints it verbatim and in place.
fn protect_disabled(css: &str, syntax: Syntax) -> Result<Protection, String> {
    let mut comments = Vec::new();
    let mut parser = ParserBuilder::new(css)
        .syntax(syntax)
        .comments(&mut comments)
        .build();
    // Syntax errors are left for malva to report.
    let Ok(stylesheet) = parser.parse::<Stylesheet>() else {
        return Ok(Protection::None);
    };

    let mut disabled = Disabled::new(css);
    let mut enable_comments = Vec::new();
    for comment in &comments {
        if !matches!(comment.kind, raffia::token::CommentKind::Block) {
            continue;
        }
        match directive::parse(comment.content) {
            Ok(Some(directive)) => {
                disabled.add(directive, comment.span.end);
                if directive == directive::Directive::Enable {
                    enable_comments.push(comment.span.end);
                }
            }
            Ok(None) => {}
            Err(message) => {
                let line = disabled.line_of(comment.span.start) + 1;
                return Err(format!("invalid directive on line {line}: {message}"));
            }
        }
    }
    if disabled.is_empty() {
        return Ok(Protection::None);
    }
    if stylesheet
        .statements
        .first()
        .is_some_and(|first| disabled.covers_rest(raffia::Spanned::span(first).start))
    {
        return Ok(Protection::All);
    }

    let Ok(ast) = serde_json::to_value(&stylesheet) else {
        return Ok(Protection::None);
    };
    let mut targets = Vec::new();
    find_disabled(&ast, &disabled, &mut targets);
    if targets.is_empty() {
        return Ok(Protection::None);
    }

    // Indented Sass has no `;`: statements end at line breaks, so the markers
    // go on lines of their own, at the statement's indentation.
    let indented = syntax == Syntax::Sass;
    let fence_after = |offset: usize| {
        if indented {
            format!(
                "\n{}{}",
                line_indent(css, offset),
                &FENCE[..FENCE.len() - 1]
            )
        } else {
            format!(" {FENCE}")
        }
    };

    // (offset, text) insertions, applied in order.
    let mut insertions: Vec<(usize, String)> = Vec::new();
    for (start, end, declaration) in targets {
        let before = match (declaration, indented) {
            // The newline keeps malva from treating the marker comment as a
            // trailing comment of the fence.
            (true, false) => format!("{FENCE}\n/* {VERBATIM} */ "),
            // malva only honors the comment on a line of its own here.
            (true, true) => {
                let indent = line_indent(css, start);
                format!(
                    "{}\n{indent}/* {VERBATIM} */\n{indent}",
                    &FENCE[..FENCE.len() - 1]
                )
            }
            (false, true) => format!("/* {VERBATIM} */\n{}", line_indent(css, start)),
            (false, false) => format!("/* {VERBATIM} */ "),
        };
        insertions.push((start, before));
        if declaration {
            let after_spaces =
                end + css[end..].len() - css[end..].trim_start_matches([' ', '\t']).len();
            if indented {
                insertions.push((end, fence_after(start)));
            } else if css[after_spaces..].starts_with(';') {
                insertions.push((after_spaces + 1, format!(" {FENCE}")));
            } else {
                insertions.push((end, format!("; {FENCE}")));
            }
        }
    }
    // malva attaches a comment to the following statement, which sorting may
    // move; a fence after `annealer-enable` keeps the comment in place.
    for end in enable_comments {
        insertions.push((end, fence_after(end)));
    }
    insertions.sort_by_key(|&(offset, _)| offset);

    let mut marked = String::with_capacity(css.len() + insertions.len() * 32);
    let mut copied = 0;
    for (offset, text) in insertions {
        marked.push_str(&css[copied..offset]);
        marked.push_str(&text);
        copied = offset;
    }
    marked.push_str(&css[copied..]);
    Ok(Protection::Marked(marked))
}

/// Collects `(start, end, is_declaration)` of the outermost disabled statements.
fn find_disabled(node: &Value, disabled: &Disabled, targets: &mut Vec<(usize, usize, bool)>) {
    match node {
        Value::Object(map) => {
            for (key, value) in map {
                match (key.as_str(), value) {
                    ("statements", Value::Array(statements)) => {
                        for statement in statements {
                            let span =
                                |key: &str| statement["span"][key].as_u64().map(|n| n as usize);
                            match (span("start"), span("end")) {
                                (Some(start), Some(end)) if disabled.covers(start) => {
                                    let declaration = statement["type"] == "Declaration";
                                    targets.push((start, end, declaration));
                                }
                                _ => find_disabled(statement, disabled, targets),
                            }
                        }
                    }
                    _ => find_disabled(value, disabled, targets),
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                find_disabled(item, disabled, targets);
            }
        }
        _ => {}
    }
}

/// Leading whitespace of the line containing `offset`.
fn line_indent(css: &str, offset: usize) -> &str {
    let line_start = css[..offset].rfind('\n').map_or(0, |i| i + 1);
    let line = &css[line_start..];
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

/// Removes the internal markers inserted by [`protect_disabled`], dropping
/// lines that held nothing else.
fn remove_markers(css: &str) -> String {
    let comment = format!("/* {VERBATIM} */");
    let mut out = String::with_capacity(css.len());
    for line in css.split_inclusive('\n') {
        if !line.contains(&FENCE[..FENCE.len() - 1]) && !line.contains(&comment) {
            out.push_str(line);
            continue;
        }
        let stripped = line
            .replace(&format!("{FENCE} "), "")
            .replace(&format!(" {FENCE}"), "")
            .replace(FENCE, "")
            .replace(&FENCE[..FENCE.len() - 1], "")
            .replace(&format!("{comment} "), "")
            .replace(&comment, "");
        if !stripped.trim().is_empty() {
            out.push_str(&stripped);
        }
    }
    out
}

/// Whether malva printed this statement verbatim (it follows the marker).
fn is_verbatim(css: &str, start: usize) -> bool {
    css[..start]
        .trim_end()
        .ends_with(&format!("/* {VERBATIM} */"))
}

/// Orders the declarations of a static `style` attribute value quoted with
/// `quote`. Returns `None` to leave the value as written: it doesn't parse as
/// plain declarations (e.g. it holds template syntax or comments), or the
/// result would need the attribute's quote character.
pub(crate) fn format_style_attribute(
    value: &str,
    quote: char,
    options: &StylesheetOptions,
) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.contains(['{', '}', '<', '&']) || value.contains("/*") {
        return None;
    }
    let mut options = options.clone();
    // One declaration per line, so each line is a whole declaration.
    options.malva.layout.print_width = usize::MAX;
    options.malva.language.quotes = if quote == '"' {
        Quotes::AlwaysSingle
    } else {
        Quotes::AlwaysDouble
    };
    let formatted = format(&format!("a {{ {value} }}"), StyleLang::Css, &options).ok()?;
    let mut lines = formatted.lines();
    if lines.next() != Some("a {") {
        return None;
    }
    let mut declarations = Vec::new();
    for line in lines {
        let line = line.trim();
        match line {
            "}" => break,
            // malva tolerates a missing value (`display: ;`); keep the input then.
            _ if line.ends_with(';') && !line[..line.len() - 1].trim_end().ends_with(':') => {
                declarations.push(line);
            }
            _ => return None,
        }
    }
    let mut text = declarations.join(" ");
    if !value.ends_with(';') {
        text.pop();
    }
    (!text.is_empty() && !text.contains(quote)).then_some(text)
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
                    let declaration = literal_declaration(statement, css)
                        .filter(|declaration| !is_verbatim(css, declaration.start));
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
            for (key, value) in map {
                if let ("statements", Value::Array(statements)) = (key.as_str(), value) {
                    for statement in statements {
                        let verbatim = statement["span"]["start"]
                            .as_u64()
                            .is_some_and(|start| is_verbatim(css, start as usize));
                        if !verbatim {
                            collect_edits(statement, css, split_on_empty_line, edits);
                        }
                    }
                } else {
                    collect_edits(value, css, split_on_empty_line, edits);
                }
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
