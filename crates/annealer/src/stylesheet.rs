//! Stylesheet declaration ordering.
//!
//! annealer only reorders: declarations are moved as written, and everything
//! else (whitespace, quotes, colors, numbers, comments, blank lines) is kept
//! byte for byte. Formatting is left to a formatter such as Prettier or Oxfmt.
//!
//! The order itself comes from `malva`'s `declarationOrder`. malva is a
//! formatter, so its output is not used; annealer only asks it how a run of
//! property names sorts and applies that permutation to the source.

use malva::Syntax;
use malva::config::{DeclarationOrder, FormatOptions, LayoutOptions};
use raffia::ast::Stylesheet;
use raffia::token::{Comment, CommentKind};
use raffia::{ParserBuilder, ParserOptions};
use serde_json::Value;

use crate::directive::{self, Disabled};
use crate::profile::{StylesheetOptions, VendorPrefix};

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

/// Reorders the declarations of a whole stylesheet.
pub(crate) fn format(
    input: &str,
    lang: StyleLang,
    options: &StylesheetOptions,
) -> Result<String, String> {
    let Some(order) = &options.declaration_order else {
        return Ok(input.to_owned());
    };

    let mut comments = Vec::new();
    let mut parser = ParserBuilder::new(input)
        .syntax(lang.syntax())
        .comments(&mut comments)
        .options(ParserOptions {
            try_parsing_value_in_custom_property: true,
            tolerate_semicolon_in_sass: true,
        })
        .build();
    let stylesheet = parser.parse::<Stylesheet>().map_err(|error| {
        let (line, column) = line_col(input, error.span.start);
        format!("syntax error at line {line}, col {column}: {}", error.kind)
    })?;

    let disabled = disabled_ranges(input, &comments)?;
    if stylesheet
        .statements
        .first()
        .is_some_and(|first| disabled.covers_rest(raffia::Spanned::span(first).start))
    {
        return Ok(input.to_owned());
    }

    let Ok(ast) = serde_json::to_value(&stylesheet) else {
        return Ok(input.to_owned());
    };
    let comment_starts: Vec<usize> = comments.iter().map(|comment| comment.span.start).collect();
    let mut runs = Vec::new();
    RunCollector {
        css: input,
        disabled: &disabled,
        comment_starts: &comment_starts,
        split_on_empty_line: options.split_on_empty_line,
        runs: &mut runs,
    }
    .visit(&ast);
    runs.retain(|run| run.len() > 1);

    let orders = sorted_orders(&runs, order);
    let mut edits = Vec::new();
    for (run, order) in runs.iter().zip(orders) {
        let Some(mut order) = order else { continue };
        if options.vendor_prefix == VendorPrefix::Start {
            move_vendor_prefixed_first(run, &mut order);
        }
        edits.extend(permute(run, &order, input));
    }
    Ok(apply(input, edits))
}

/// 1-based line and column of `offset`.
fn line_col(css: &str, offset: usize) -> (usize, usize) {
    let before = &css[..offset.min(css.len())];
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    (
        before.matches('\n').count() + 1,
        before[line_start..].chars().count() + 1,
    )
}

/// Applies `/* annealer-disable* */` directives found in block comments.
fn disabled_ranges(css: &str, comments: &[Comment<'_>]) -> Result<Disabled, String> {
    let mut disabled = Disabled::new(css);
    for comment in comments {
        if !matches!(comment.kind, CommentKind::Block) {
            continue;
        }
        match directive::parse(comment.content) {
            Ok(Some(directive)) => disabled.add(directive, comment.span.end),
            Ok(None) => {}
            Err(message) => {
                let line = disabled.line_of(comment.span.start) + 1;
                return Err(format!("invalid directive on line {line}: {message}"));
            }
        }
    }
    Ok(disabled)
}

/// One declaration in the source.
struct Declaration<'a> {
    start: usize,
    end: usize,
    name: &'a str,
}

/// Collects the runs of declarations that are sorted as a unit.
///
/// A run is a sequence of consecutive declarations with a literal name. It is
/// ended by any other statement, a comment, a declaration an
/// `annealer-disable*` directive covers, a declaration holding a nested block
/// and, with `split_on_empty_line`, a blank line. Declarations never move
/// across these, so comments and blank lines stay attached to their
/// neighbors.
///
/// The AST is walked as JSON because raffia has no visitor and blocks appear
/// in many node types.
struct RunCollector<'r, 'a> {
    css: &'a str,
    disabled: &'r Disabled,
    comment_starts: &'r [usize],
    split_on_empty_line: bool,
    runs: &'r mut Vec<Vec<Declaration<'a>>>,
}

impl<'a> RunCollector<'_, 'a> {
    fn visit(&mut self, node: &Value) {
        match node {
            Value::Object(map) => {
                for (key, value) in map {
                    match (key.as_str(), value) {
                        ("statements", Value::Array(statements)) => self.statements(statements),
                        _ => self.visit(value),
                    }
                }
            }
            Value::Array(items) => {
                for item in items {
                    self.visit(item);
                }
            }
            _ => {}
        }
    }

    fn statements(&mut self, statements: &[Value]) {
        let mut run: Vec<Declaration<'a>> = Vec::new();
        for statement in statements {
            let start = offset(statement, "start");
            if start.is_some_and(|start| self.disabled.covers(start)) {
                // Left exactly as written, nested blocks included.
                self.runs.push(std::mem::take(&mut run));
                continue;
            }
            let declaration = self
                .sortable_declaration(statement)
                .filter(|_| !contains_block(statement));
            match declaration {
                Some(next) => {
                    if run
                        .last()
                        .is_some_and(|previous| !self.joins(previous, &next))
                    {
                        self.runs.push(std::mem::take(&mut run));
                    }
                    run.push(next);
                }
                None => {
                    self.runs.push(std::mem::take(&mut run));
                    self.visit(statement);
                }
            }
        }
        self.runs.push(run);
    }

    fn sortable_declaration(&self, statement: &Value) -> Option<Declaration<'a>> {
        if statement["type"] != "Declaration" || statement["name"]["type"] != "Ident" {
            return None;
        }
        let name = &statement["name"];
        let mut end = offset(statement, "end")?;
        // Spaces before the `;` belong to the declaration and move with it.
        let rest = self.css.get(end..)?;
        let spaces = rest.len() - rest.trim_start_matches([' ', '\t']).len();
        if rest[spaces..].starts_with(';') {
            end += spaces;
        }
        Some(Declaration {
            start: offset(statement, "start")?,
            end,
            name: self.css.get(offset(name, "start")?..offset(name, "end")?)?,
        })
    }

    /// Whether `next` continues the run that ends with `previous`.
    fn joins(&self, previous: &Declaration<'_>, next: &Declaration<'_>) -> bool {
        let has_comment = self
            .comment_starts
            .iter()
            .any(|&start| previous.end <= start && start < next.start);
        let lines: Vec<&str> = self.css[previous.end..next.start].split('\n').collect();
        // The first and last pieces are the rest of the previous line and the
        // indentation of the next one; anything between is a whole line.
        let has_empty_line = lines.len() > 2
            && lines[1..lines.len() - 1]
                .iter()
                .any(|line| line.trim().is_empty());
        !has_comment && !(self.split_on_empty_line && has_empty_line)
    }
}

fn offset(node: &Value, key: &str) -> Option<usize> {
    node["span"][key].as_u64().map(|n| n as usize)
}

/// Whether the node holds a block (e.g. Sass nested properties).
fn contains_block(node: &Value) -> bool {
    match node {
        Value::Object(map) => map
            .iter()
            .any(|(key, value)| key == "statements" || contains_block(value)),
        Value::Array(items) => items.iter().any(contains_block),
        _ => false,
    }
}

/// Asks malva how each run sorts. Returns, per run, the source indices in
/// sorted order, or `None` to leave the run as written.
///
/// All runs go into one synthetic stylesheet, one rule per run, with each
/// declaration's index as its value. malva sorts by name only, and stably.
fn sorted_orders(
    runs: &[Vec<Declaration<'_>>],
    order: &DeclarationOrder,
) -> Vec<Option<Vec<usize>>> {
    if runs.is_empty() {
        return Vec::new();
    }
    let mut synthetic = String::new();
    for run in runs {
        synthetic.push_str("a{");
        for (index, declaration) in run.iter().enumerate() {
            synthetic.push_str(declaration.name);
            synthetic.push(':');
            synthetic.push_str(&index.to_string());
            synthetic.push(';');
        }
        synthetic.push_str("}\n");
    }
    let mut options = FormatOptions {
        layout: LayoutOptions {
            print_width: usize::MAX,
            ..LayoutOptions::default()
        },
        ..FormatOptions::default()
    };
    options.language.declaration_order = Some(order.clone());
    let Ok(sorted) = malva::format_text(&synthetic, Syntax::Css, &options) else {
        return vec![None; runs.len()];
    };

    let mut blocks: Vec<Vec<usize>> = Vec::with_capacity(runs.len());
    let mut current: Option<Vec<usize>> = None;
    for line in sorted.lines().map(str::trim) {
        match line {
            "a {" => current = Some(Vec::new()),
            "}" => blocks.extend(current.take()),
            _ => {
                let index = line
                    .strip_suffix(';')
                    .and_then(|declaration| declaration.rsplit_once(':'))
                    .and_then(|(_, value)| value.trim().parse().ok());
                match (&mut current, index) {
                    (Some(block), Some(index)) => block.push(index),
                    _ => return vec![None; runs.len()],
                }
            }
        }
    }
    if blocks.len() != runs.len() {
        return vec![None; runs.len()];
    }
    runs.iter()
        .zip(blocks)
        .map(|(run, block)| {
            let mut seen = vec![false; run.len()];
            let valid = block.len() == run.len()
                && block
                    .iter()
                    .all(|&i| i < run.len() && !std::mem::replace(&mut seen[i], true));
            valid.then_some(block)
        })
        .collect()
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

/// Moves vendor-prefixed declarations before the others, alphabetically.
/// malva sorts them by the unprefixed name, next to the standard property.
fn move_vendor_prefixed_first(run: &[Declaration<'_>], order: &mut Vec<usize>) {
    let (mut vendor, standard): (Vec<usize>, Vec<usize>) = order
        .iter()
        .partition(|&&i| is_vendor_prefixed(run[i].name));
    vendor.sort_by_cached_key(|&i| run[i].name.to_ascii_lowercase());
    *order = vendor.into_iter().chain(standard).collect();
}

/// Rewrites a run in `order`. Separators (`;`, whitespace) stay in place, so
/// a missing trailing `;` stays missing and the layout doesn't change.
fn permute(run: &[Declaration<'_>], order: &[usize], css: &str) -> Option<(usize, usize, String)> {
    if order.iter().enumerate().all(|(position, &i)| position == i) {
        return None;
    }
    let mut text = String::new();
    for (position, &i) in order.iter().enumerate() {
        if position > 0 {
            text.push_str(&css[run[position - 1].end..run[position].start]);
        }
        text.push_str(&css[run[i].start..run[i].end]);
    }
    Some((run[0].start, run[run.len() - 1].end, text))
}

/// Applies non-overlapping `(start, end, replacement)` edits.
fn apply(css: &str, mut edits: Vec<(usize, usize, String)>) -> String {
    if edits.is_empty() {
        return css.to_owned();
    }
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

/// Orders the declarations of a static `style` attribute value. Returns
/// `None` to leave the value as written: nothing moves, or it doesn't parse
/// as plain declarations (e.g. it holds template syntax).
pub(crate) fn format_style_attribute(value: &str, options: &StylesheetOptions) -> Option<String> {
    if value.trim().is_empty() || value.contains(['{', '}', '<', '&']) {
        return None;
    }
    let wrapped = format!("a{{{value}}}");
    let formatted = format(&wrapped, StyleLang::Css, options).ok()?;
    let reordered = formatted.strip_prefix("a{")?.strip_suffix('}')?;
    (reordered != value).then(|| reordered.to_owned())
}

/// Reorders the content of a `<style>` element.
pub(crate) fn format_embedded(
    content: &str,
    lang: StyleLang,
    options: &StylesheetOptions,
) -> Result<String, String> {
    if lang != StyleLang::Sass {
        return format(content, lang, options);
    }
    // Indented Sass reads indentation as structure, so the block's common
    // indentation is removed first and put back afterwards. Declarations
    // are whole lines in Sass, so every non-blank line of the result had it.
    let Some((first, rest)) = content.split_once('\n') else {
        return format(content, lang, options);
    };
    if !first.trim().is_empty() {
        return format(content, lang, options);
    }
    let indent = rest
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| &line[..line.len() - line.trim_start_matches([' ', '\t']).len()])
        .min_by_key(|indent| indent.len())
        .unwrap_or("");
    let is_blank = |line: &str| line.trim().is_empty();
    if indent.is_empty()
        || !rest
            .lines()
            .all(|line| is_blank(line) || line.starts_with(indent))
    {
        return format(content, lang, options);
    }
    let dedented: String = rest
        .split_inclusive('\n')
        .map(|line| {
            if is_blank(line) {
                line
            } else {
                &line[indent.len()..]
            }
        })
        .collect();
    let formatted = format(&dedented, lang, options)?;
    let mut out = String::with_capacity(content.len());
    out.push_str(first);
    out.push('\n');
    for line in formatted.split_inclusive('\n') {
        if !is_blank(line) {
            out.push_str(indent);
        }
        out.push_str(line);
    }
    Ok(out)
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

    #[test]
    fn reports_line_and_column() {
        assert_eq!(line_col("a\nbc", 3), (2, 2));
        assert_eq!(line_col("abc", 0), (1, 1));
    }
}
