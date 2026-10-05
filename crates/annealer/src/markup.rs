//! Lexer-level markup scanner.
//!
//! No DOM is built: the scanner only locates start tags and tokenizes their
//! attribute lists. Everything else is copied through byte-for-byte.

use crate::attribute::{Key, is_component, key_of, normalize_name};
use crate::directive::{self, Disabled};
use crate::error::{FormatError, line_col};
use crate::order::attribute_order;
use crate::profile::{Profile, SelfClosing};
use crate::stylesheet::{self, StyleLang};

/// Elements that never have content or an end tag.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Elements where whitespace-only content is significant.
const WHITESPACE_SENSITIVE: &[&str] = &["pre", "textarea"];

/// Elements whose content is raw text and must not be scanned for tags.
///
/// Obsolete elements (`xmp`, `noembed`, `noframes`) are listed only because
/// browsers still parse their content as raw text. Warning about or removing
/// deprecated markup is out of scope (see the Non-goals in PLAN.md).
const RAW_TEXT_ELEMENTS: &[&str] = &[
    "script", "style", "textarea", "title", "xmp", "iframe", "noembed", "noframes",
];

struct Attribute<'a> {
    name: &'a str,
    /// Everything after the name: `="value"`, including any spaces around `=`.
    rest: &'a str,
    value: Option<&'a str>,
    /// Quote character around the value, if any.
    quote: Option<char>,
}

struct StartTag<'a> {
    start: usize,
    end: usize,
    name: &'a str,
    /// Whitespace before each attribute.
    separators: Vec<&'a str>,
    attributes: Vec<Attribute<'a>>,
    /// Whitespace between the last attribute and the closing bracket.
    trailing: &'a str,
    self_closing: bool,
    /// The element is complete after this tag: self-closing, or its end tag was emitted with it.
    closed: bool,
    /// An `annealer-disable*` directive covers this tag; leave it (and a
    /// `<style>` element's content) untouched.
    disabled: bool,
}

/// An element whose end tag hasn't been seen yet.
struct OpenElement<'a> {
    name: &'a str,
    /// Source offset of the start tag.
    start: usize,
    /// Offset in the output just past the start tag.
    content_start: usize,
    /// Indentation of the attribute lines of a multiline start tag.
    attribute_indent: Option<String>,
    /// Put the content on lines of its own if the element turns out multiline.
    break_content: bool,
    /// Inside an element `layout.contentNewline` ignores.
    in_ignored: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ElementKind {
    Void,
    Normal,
    Component,
}

/// How the start tag's closing is rewritten.
#[derive(Clone, Copy, Default)]
struct Closing {
    /// A self-closing rule decided the closing; normalize the space before it.
    governed: bool,
    /// Also emit `</name>` right after the start tag.
    end_tag: bool,
}

impl StartTag<'_> {
    fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|attribute| attribute.name.eq_ignore_ascii_case(name))
            .and_then(|attribute| attribute.value)
    }
}

pub(crate) struct Scanner<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
    /// Input up to this offset has already been written to `out`.
    copied: usize,
    out: String,
    profile: &'a Profile,
    vue: bool,
    disabled: Disabled,
    /// `layout.contentNewline` moved content onto a new line.
    content_moved: bool,
}

impl<'a> Scanner<'a> {
    pub(crate) fn new(src: &'a str, profile: &'a Profile, vue: bool) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
            copied: 0,
            out: String::with_capacity(src.len()),
            profile,
            vue,
            disabled: Disabled::new(src),
            content_moved: false,
        }
    }

    /// Formats `src`. Content moved onto a new line changes the indentation
    /// of the line it starts, which other layout rules (the closing bracket of
    /// a multiline tag) depend on, so it is formatted again until stable.
    pub(crate) fn format(src: &str, profile: &Profile, vue: bool) -> Result<String, FormatError> {
        const MAX_PASSES: usize = 4;
        let mut text = src.to_owned();
        for _ in 0..MAX_PASSES {
            let mut scanner = Scanner::new(&text, profile, vue);
            scanner.scan()?;
            let moved = scanner.content_moved;
            text = scanner.finish();
            if !moved {
                break;
            }
        }
        Ok(text)
    }

    fn finish(mut self) -> String {
        self.out.push_str(&self.src[self.copied..]);
        self.out
    }

    fn scan(&mut self) -> Result<(), FormatError> {
        if self.vue {
            self.scan_sfc()
        } else {
            self.scan_markup(false)
        }
    }

    /// Copies the input up to `offset` to the output.
    fn flush(&mut self, offset: usize) {
        if offset > self.copied {
            self.out.push_str(&self.src[self.copied..offset]);
            self.copied = offset;
        }
    }

    fn replace(&mut self, start: usize, end: usize, text: &str) {
        self.out.push_str(&self.src[self.copied..start]);
        self.out.push_str(text);
        self.copied = end;
    }

    fn find(&self, needle: &str, from: usize) -> Option<usize> {
        self.src[from..].find(needle).map(|i| from + i)
    }

    fn skip_past(&mut self, needle: &str) {
        self.pos = self
            .find(needle, self.pos)
            .map_or(self.bytes.len(), |i| i + needle.len());
    }

    fn at(&self, prefix: &str) -> bool {
        self.bytes[self.pos..].starts_with(prefix.as_bytes())
    }

    fn at_start_tag(&self) -> bool {
        self.bytes
            .get(self.pos + 1)
            .is_some_and(u8::is_ascii_alphabetic)
    }

    /// Vue SFC top level: `<template>` is markup, other blocks are raw text.
    fn scan_sfc(&mut self) -> Result<(), FormatError> {
        while let Some(lt) = self.find("<", self.pos) {
            self.pos = lt;
            if self.at("<!--") {
                self.comment()?;
            } else if self.at("<!") || self.at("<?") || self.at("</") {
                self.skip_past(">");
            } else if self.at_start_tag() {
                let tag = self.start_tag(false)?;
                if tag.closed {
                    continue;
                }
                let name = tag.name.to_ascii_lowercase();
                let lang = tag.attribute("lang").map(str::to_ascii_lowercase);
                match name.as_str() {
                    "template" if lang.as_deref().is_none_or(|lang| lang == "html") => {
                        self.scan_markup(true)?;
                    }
                    "style" if !tag.disabled => self.style_element(lang.as_deref())?,
                    _ => self.raw_text(&name),
                }
            } else {
                self.pos += 1;
            }
        }
        Ok(())
    }

    /// Scans markup. With `in_template`, stops at the `</template>` that closes
    /// the enclosing SFC block.
    fn scan_markup(&mut self, in_template: bool) -> Result<(), FormatError> {
        let mut template_depth = 0usize;
        let mut open: Vec<OpenElement<'a>> = Vec::new();
        while self.pos < self.bytes.len() {
            let Some(next) = self.bytes[self.pos..]
                .iter()
                .position(|&b| b == b'<' || (self.vue && b == b'{'))
            else {
                self.pos = self.bytes.len();
                break;
            };
            self.pos += next;

            if self.at("{{") {
                self.skip_past("}}");
            } else if self.at("{") {
                self.pos += 1;
            } else if self.at("<!--") {
                self.comment()?;
            } else if self.at("<!") || self.at("<?") {
                self.skip_past(">");
            } else if self.at("</") {
                let name_end = self.name_end(self.pos + 2);
                let name = &self.src[self.pos + 2..name_end];
                if in_template && name.eq_ignore_ascii_case("template") {
                    if template_depth == 0 {
                        return Ok(());
                    }
                    template_depth -= 1;
                }
                self.end_tag(&mut open, name);
                self.skip_past(">");
            } else if self.at_start_tag() {
                self.flush(self.pos);
                let tag_out_start = self.out.len();
                let tag = self.start_tag(true)?;
                if tag.closed {
                    continue;
                }
                let name = tag.name.to_ascii_lowercase();
                if !RAW_TEXT_ELEMENTS.contains(&name.as_str())
                    && !self.element_kind(tag.name).eq(&ElementKind::Void)
                {
                    self.flush(self.pos);
                    open.push(self.open_element(&tag, tag_out_start, open.last()));
                }
                if name == "template" {
                    template_depth += 1;
                } else if name == "style" && !tag.disabled {
                    let lang = tag.attribute("lang").map(str::to_ascii_lowercase);
                    self.style_element(lang.as_deref())?;
                } else if RAW_TEXT_ELEMENTS.contains(&name.as_str()) {
                    self.raw_text(&name);
                }
            } else {
                self.pos += 1;
            }
        }
        Ok(())
    }

    fn open_element(
        &self,
        tag: &StartTag<'a>,
        tag_out_start: usize,
        parent: Option<&OpenElement<'_>>,
    ) -> OpenElement<'a> {
        let rule = self.profile.layout.content_newline.as_ref();
        let ignored = rule.is_some_and(|rule| {
            let component = self.vue && is_component(tag.name);
            rule.ignore
                .iter()
                .any(|name| name == tag.name || (!component && name.eq_ignore_ascii_case(tag.name)))
        });
        let in_ignored = ignored || parent.is_some_and(|parent| parent.in_ignored);
        let tag_text = &self.out[tag_out_start..];
        let attribute_indent = tag_text.find('\n').map(|newline| {
            let line = &tag_text[newline + 1..];
            line[..line.len() - line.trim_start_matches([' ', '\t']).len()].to_owned()
        });
        OpenElement {
            name: tag.name,
            start: tag.start,
            content_start: self.out.len(),
            attribute_indent,
            break_content: rule.is_some() && !in_ignored && !tag.disabled,
            in_ignored,
        }
    }

    /// Closes the innermost open element named `name`, if any; elements
    /// opened after it are implicitly closed.
    fn end_tag(&mut self, open: &mut Vec<OpenElement<'a>>, name: &str) {
        let matches = |element: &OpenElement<'_>| {
            if self.vue && is_component(element.name) {
                element.name == name
            } else {
                element.name.eq_ignore_ascii_case(name)
            }
        };
        let Some(index) = open.iter().rposition(matches) else {
            return;
        };
        open.truncate(index + 1);
        let element = open.pop().expect("index is in bounds");
        let end = self.pos;
        if element.break_content
            && self.src[element.start..end].contains('\n')
            && !self.disabled.covers(end)
        {
            self.break_content(&element, end);
        }
    }

    /// Puts the content of `element`, which ends at `end`, on lines of its own.
    fn break_content(&mut self, element: &OpenElement<'_>, end: usize) {
        let Some(rule) = &self.profile.layout.content_newline else {
            return;
        };
        self.flush(end);
        let content = &self.out[element.content_start..];
        let body = content.trim_matches(is_html_whitespace);
        if body.is_empty() {
            return;
        }
        let leading =
            &content[..content.len() - content.trim_start_matches(is_html_whitespace).len()];
        let trailing = &content[content.trim_end_matches(is_html_whitespace).len()..];
        let newline = if self.src.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let tag_indent = line_indent(self.src, element.start);

        let break_before = |space: &str, indent: &dyn Fn() -> String| match space.rfind('\n') {
            Some(_) if rule.allow_empty_lines => space.to_owned(),
            Some(last) => format!("{newline}{}", &space[last + 1..]),
            None => format!("{newline}{}", indent()),
        };
        let content_indent = || {
            element
                .attribute_indent
                .clone()
                .or_else(|| {
                    body.lines()
                        .skip(1)
                        .filter(|line| !line.trim().is_empty())
                        .map(|line| {
                            &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
                        })
                        .find(|indent| {
                            indent.len() > tag_indent.len() && indent.starts_with(tag_indent)
                        })
                        .map(str::to_owned)
                })
                .unwrap_or_else(|| {
                    let unit = if tag_indent.contains('\t') {
                        "\t"
                    } else {
                        "  "
                    };
                    format!("{tag_indent}{unit}")
                })
        };
        let text = format!(
            "{}{body}{}",
            break_before(leading, &content_indent),
            break_before(trailing, &|| tag_indent.to_owned()),
        );
        if text != content {
            self.content_moved |= !leading.contains('\n');
            self.out.truncate(element.content_start);
            self.out.push_str(&text);
        }
    }

    /// Skips a comment, recording any `annealer-*` directive in it.
    fn comment(&mut self) -> Result<(), FormatError> {
        let start = self.pos;
        let content_start = start + "<!--".len();
        let content_end = self.find("-->", content_start).unwrap_or(self.bytes.len());
        self.pos = (content_end + "-->".len()).min(self.bytes.len());
        match directive::parse(&self.src[content_start..content_end]) {
            Ok(Some(directive)) => self.disabled.add(directive, self.pos),
            Ok(None) => {}
            Err(message) => {
                let (line, column) = line_col(self.src, start);
                return Err(FormatError::Directive {
                    line,
                    column,
                    message,
                });
            }
        }
        Ok(())
    }

    /// Skips raw text up to `</name`.
    fn raw_text(&mut self, name: &str) {
        self.pos =
            find_ignore_case(self.src, &format!("</{name}"), self.pos).unwrap_or(self.bytes.len());
    }

    /// Skips a `<style>` element's content, formatting it as a stylesheet.
    fn style_element(&mut self, style_lang: Option<&str>) -> Result<(), FormatError> {
        let start = self.pos;
        self.raw_text("style");
        let end = self.pos;

        let (Some(options), Some(lang)) =
            (&self.profile.stylesheet, StyleLang::from_lang(style_lang))
        else {
            return Ok(());
        };
        let content = &self.src[start..end];
        if content.trim().is_empty() {
            return Ok(());
        }
        let base_indent = line_indent(self.src, end);
        match stylesheet::format_embedded(content, lang, options, base_indent) {
            Ok(formatted) => {
                if formatted != content {
                    self.replace(start, end, &formatted);
                }
                Ok(())
            }
            Err(message) => {
                let (line, column) = line_col(self.src, start);
                Err(FormatError::Stylesheet {
                    line,
                    column,
                    message,
                })
            }
        }
    }

    fn name_end(&self, from: usize) -> usize {
        self.bytes[from..]
            .iter()
            .position(|&b| b.is_ascii_whitespace() || b == b'/' || b == b'>')
            .map_or(self.bytes.len(), |i| from + i)
    }

    fn skip_whitespace(&self, mut i: usize) -> usize {
        while i < self.bytes.len() && self.bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        i
    }

    /// Tokenizes the start tag at `self.pos`, rewrites it, and advances past it.
    /// With `self_closing_rules`, applies the profile's self-closing style.
    fn start_tag(&mut self, self_closing_rules: bool) -> Result<StartTag<'a>, FormatError> {
        let mut tag = self.parse_start_tag(self.pos)?;
        if self.disabled.covers(tag.start) {
            tag.disabled = true;
            self.pos = tag.end;
            return Ok(tag);
        }
        let mut end = tag.end;
        let closing = if self_closing_rules {
            self.apply_self_closing(&mut tag, &mut end)
        } else {
            Closing::default()
        };
        self.pos = end;
        let text = self.rewrite(&tag, closing);
        if text != self.src[tag.start..end] {
            self.replace(tag.start, end, &text);
        }
        Ok(tag)
    }

    fn element_kind(&self, name: &str) -> ElementKind {
        // Vue treats `<Link>` as a component, so its void check is case-sensitive.
        let void = if self.vue {
            VOID_ELEMENTS.contains(&name)
        } else {
            VOID_ELEMENTS
                .iter()
                .any(|void| void.eq_ignore_ascii_case(name))
        };
        if void {
            ElementKind::Void
        } else if self.vue && is_component(name) {
            ElementKind::Component
        } else {
            ElementKind::Normal
        }
    }

    /// Decides the closing style; may extend `end` over an empty element's end tag.
    fn apply_self_closing(&self, tag: &mut StartTag<'a>, end: &mut usize) -> Closing {
        let rules = &self.profile.layout.self_closing;
        let kind = self.element_kind(tag.name);
        let policy = match kind {
            ElementKind::Void => rules.void,
            // `<div />` is an unclosed start tag in plain HTML.
            _ if !self.vue => SelfClosing::Preserve,
            ElementKind::Normal => rules.normal,
            ElementKind::Component => rules.component,
        };
        let governed = Closing {
            governed: true,
            end_tag: false,
        };
        match (policy, kind, tag.self_closing) {
            (SelfClosing::Preserve, ..) => Closing::default(),
            (SelfClosing::Always, ElementKind::Void, _) => {
                tag.self_closing = true;
                tag.closed = true;
                governed
            }
            (SelfClosing::Never, ElementKind::Void, _) => {
                tag.self_closing = false;
                governed
            }
            (SelfClosing::Always, _, true) => governed,
            (SelfClosing::Always, _, false) => match self.empty_element_end(tag, kind) {
                Some(end_tag_end) => {
                    *end = end_tag_end;
                    tag.self_closing = true;
                    tag.closed = true;
                    governed
                }
                None => Closing::default(),
            },
            (SelfClosing::Never, _, true) => {
                tag.self_closing = false;
                Closing {
                    governed: true,
                    end_tag: true,
                }
            }
            (SelfClosing::Never, _, false) => Closing::default(),
        }
    }

    /// If the element has no content, returns the offset just past its end tag.
    fn empty_element_end(&self, tag: &StartTag<'_>, kind: ElementKind) -> Option<usize> {
        let whitespace_ok = kind == ElementKind::Component
            || !WHITESPACE_SENSITIVE
                .iter()
                .any(|name| name.eq_ignore_ascii_case(tag.name));
        let close = if whitespace_ok {
            self.skip_whitespace(tag.end)
        } else {
            tag.end
        };
        let name_start = close + 2;
        let name_end = self.name_end(name_start);
        let name = self.src.get(name_start..name_end)?;
        let same_name = match kind {
            ElementKind::Component => name == tag.name,
            _ => name.eq_ignore_ascii_case(tag.name),
        };
        let gt = self.skip_whitespace(name_end);
        (self.at_offset(close, "</") && same_name && self.bytes.get(gt) == Some(&b'>'))
            .then_some(gt + 1)
    }

    fn at_offset(&self, offset: usize, prefix: &str) -> bool {
        self.bytes[offset..].starts_with(prefix.as_bytes())
    }

    fn parse_start_tag(&self, start: usize) -> Result<StartTag<'a>, FormatError> {
        let (src, bytes) = (self.src, self.bytes);
        let unterminated = || {
            let (line, column) = line_col(src, start);
            FormatError::UnterminatedTag { line, column }
        };

        let name_end = self.name_end(start + 1);
        let mut tag = StartTag {
            start,
            end: 0,
            name: &src[start + 1..name_end],
            separators: Vec::new(),
            attributes: Vec::new(),
            trailing: "",
            self_closing: false,
            closed: false,
            disabled: false,
        };

        let mut i = name_end;
        loop {
            let separator_start = i;
            // Whitespace and stray slashes separate attributes.
            while i < bytes.len()
                && (bytes[i].is_ascii_whitespace()
                    || (bytes[i] == b'/' && bytes.get(i + 1) != Some(&b'>')))
            {
                i += 1;
            }
            match bytes.get(i) {
                None => return Err(unterminated()),
                Some(b'>') => {
                    tag.trailing = &src[separator_start..i];
                    tag.end = i + 1;
                    return Ok(tag);
                }
                Some(b'/') => {
                    tag.trailing = &src[separator_start..i];
                    tag.self_closing = true;
                    tag.closed = true;
                    tag.end = i + 2;
                    return Ok(tag);
                }
                Some(_) => {}
            }

            // Attribute name. `[` … `]` (Vue dynamic arguments) may contain anything but `>`.
            let name_start = i;
            let mut depth = 0usize;
            while let Some(&b) = bytes.get(i) {
                match b {
                    b'[' => depth += 1,
                    b']' => depth = depth.saturating_sub(1),
                    b'>' => break,
                    _ if depth > 0 => {}
                    b'/' => break,
                    b'=' if i > name_start => break,
                    _ if b.is_ascii_whitespace() => break,
                    _ => {}
                }
                i += 1;
            }
            let name = &src[name_start..i];

            // Optional `= value`.
            let mut value = None;
            let mut value_quote = None;
            let after_name = self.skip_whitespace(i);
            if bytes.get(after_name) == Some(&b'=') {
                let value_start = self.skip_whitespace(after_name + 1);
                match bytes.get(value_start) {
                    None => return Err(unterminated()),
                    Some(&quote @ (b'"' | b'\'')) => {
                        let close = bytes[value_start + 1..]
                            .iter()
                            .position(|&b| b == quote)
                            .ok_or_else(|| {
                                let (line, column) = line_col(src, name_start);
                                FormatError::UnterminatedAttributeValue { line, column }
                            })?;
                        let value_end = value_start + 1 + close;
                        value = Some(&src[value_start + 1..value_end]);
                        value_quote = Some(char::from(quote));
                        i = value_end + 1;
                    }
                    Some(_) => {
                        let mut end = value_start;
                        while end < bytes.len()
                            && !bytes[end].is_ascii_whitespace()
                            && bytes[end] != b'>'
                        {
                            end += 1;
                        }
                        value = Some(&src[value_start..end]);
                        i = end;
                    }
                }
            }

            tag.separators.push(&src[separator_start..name_start]);
            tag.attributes.push(Attribute {
                name,
                rest: &src[name_start + name.len()..i],
                value,
                quote: value_quote,
            });
        }
    }

    /// The reordered value of a static `style` attribute, when enabled.
    fn style_attribute(&self, attribute: &Attribute<'_>, key: &Key) -> Option<String> {
        let options = self.profile.stylesheet.as_ref()?;
        if !options.style_attribute || key.bound || !attribute.name.eq_ignore_ascii_case("style") {
            return None;
        }
        stylesheet::format_style_attribute(attribute.value?, attribute.quote?, options)
    }

    /// Builds the rewritten start tag (plus the end tag when `closing` asks for it).
    fn rewrite(&self, tag: &StartTag<'_>, closing: Closing) -> String {
        let profile = self.profile;
        let component = is_component(tag.name);
        let names: Vec<String> = tag
            .attributes
            .iter()
            .map(|attribute| normalize_name(attribute.name, component, &profile.normalize))
            .collect();
        let keys: Vec<_> = names.iter().map(|name| key_of(name)).collect();
        let order = attribute_order(&keys, profile);

        // Separators stay in place while attributes move between them, unless
        // the tag is laid out one attribute per line.
        let attribute_indent = tag
            .separators
            .iter()
            .find_map(|separator| {
                let newline = separator.rfind('\n')?;
                let line_break = if separator[..newline].ends_with('\r') {
                    newline - 1
                } else {
                    newline
                };
                Some(&separator[line_break..])
            })
            .filter(|_| profile.layout.one_attribute_per_line);
        let separators: Vec<&str> = match attribute_indent {
            Some(indent) => vec![indent; tag.separators.len()],
            None => tag.separators.clone(),
        };

        let mut text = String::with_capacity(tag.end - tag.start + 8);
        text.push('<');
        text.push_str(tag.name);
        for (separator, &index) in separators.iter().zip(&order) {
            text.push_str(separator);
            text.push_str(&names[index]);
            let attribute = &tag.attributes[index];
            match self.style_attribute(attribute, &keys[index]) {
                Some(value) => {
                    let value_start = attribute
                        .rest
                        .find(attribute.quote.unwrap_or('"'))
                        .map_or(0, |quote| quote + 1);
                    text.push_str(&attribute.rest[..value_start]);
                    text.push_str(&value);
                    text.push_str(
                        &attribute.rest[value_start + attribute.value.unwrap_or("").len()..],
                    );
                }
                None => text.push_str(attribute.rest),
            }
        }

        let multiline = text.contains('\n');
        if profile.layout.closing_bracket_newline && multiline {
            text.push_str(if text.contains("\r\n") { "\r\n" } else { "\n" });
            text.push_str(line_indent(self.src, tag.start));
        } else if profile.layout.closing_bracket_newline && tag.trailing.contains('\n') {
            // Attributes on one line: the bracket stays on that line, like
            // `vue/html-closing-bracket-newline`'s `singleline: never`.
            text.push_str(if tag.self_closing { " " } else { "" });
        } else if closing.governed && !tag.trailing.contains('\n') {
            // `<br />`, `<br>`: exactly one space before `/>`, none before `>`.
            text.push_str(if tag.self_closing { " " } else { "" });
        } else {
            text.push_str(tag.trailing);
        }
        text.push_str(if tag.self_closing { "/>" } else { ">" });
        if closing.end_tag {
            text.push_str("</");
            text.push_str(tag.name);
            text.push('>');
        }
        text
    }
}

/// Leading whitespace of the line containing `offset`.
fn line_indent(src: &str, offset: usize) -> &str {
    let line_start = src[..offset].rfind('\n').map_or(0, |i| i + 1);
    let line = &src[line_start..];
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

fn is_html_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0C')
}

fn find_ignore_case(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    let hay = haystack.as_bytes();
    let needle = needle.as_bytes();
    (from..=hay.len().checked_sub(needle.len())?)
        .find(|&i| hay[i..i + needle.len()].eq_ignore_ascii_case(needle))
}
