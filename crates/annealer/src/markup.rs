//! Lexer-level markup scanner.
//!
//! No DOM is built: the scanner only locates start tags and tokenizes their
//! attribute lists. Everything else is copied through byte-for-byte.

use crate::attribute::{is_component, key_of, normalize_name};
use crate::error::{FormatError, line_col};
use crate::order::attribute_order;
use crate::profile::Profile;
use crate::stylesheet::{self, StyleLang};

/// Elements whose content is raw text and must not be scanned for tags.
const RAW_TEXT_ELEMENTS: &[&str] = &[
    "script", "style", "textarea", "title", "xmp", "iframe", "noembed", "noframes",
];

struct Attribute<'a> {
    name: &'a str,
    /// Everything after the name: `="value"`, including any spaces around `=`.
    rest: &'a str,
    value: Option<&'a str>,
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
        }
    }

    pub(crate) fn run(mut self) -> Result<String, FormatError> {
        if self.vue {
            self.scan_sfc()?;
        } else {
            self.scan_markup(false)?;
        }
        self.out.push_str(&self.src[self.copied..]);
        Ok(self.out)
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
                self.skip_past("-->");
            } else if self.at("<!") || self.at("<?") || self.at("</") {
                self.skip_past(">");
            } else if self.at_start_tag() {
                let tag = self.start_tag()?;
                if tag.self_closing {
                    continue;
                }
                let name = tag.name.to_ascii_lowercase();
                let lang = tag.attribute("lang").map(str::to_ascii_lowercase);
                match name.as_str() {
                    "template" if lang.as_deref().is_none_or(|lang| lang == "html") => {
                        self.scan_markup(true)?;
                    }
                    "style" => self.raw_text("style", lang.as_deref())?,
                    _ => self.raw_text(&name, None)?,
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
                self.skip_past("-->");
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
                self.skip_past(">");
            } else if self.at_start_tag() {
                let tag = self.start_tag()?;
                if tag.self_closing {
                    continue;
                }
                let name = tag.name.to_ascii_lowercase();
                if name == "template" {
                    template_depth += 1;
                } else if name == "style" {
                    let lang = tag.attribute("lang").map(str::to_ascii_lowercase);
                    self.raw_text("style", lang.as_deref())?;
                } else if RAW_TEXT_ELEMENTS.contains(&name.as_str()) {
                    self.raw_text(&name, None)?;
                }
            } else {
                self.pos += 1;
            }
        }
        Ok(())
    }

    /// Skips raw text up to `</name`, formatting it first if it is a stylesheet.
    fn raw_text(&mut self, name: &str, style_lang: Option<&str>) -> Result<(), FormatError> {
        let start = self.pos;
        let end =
            find_ignore_case(self.src, &format!("</{name}"), start).unwrap_or(self.bytes.len());
        self.pos = end;

        if name != "style" {
            return Ok(());
        }
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
    fn start_tag(&mut self) -> Result<StartTag<'a>, FormatError> {
        let tag = self.parse_start_tag(self.pos)?;
        self.pos = tag.end;
        if let Some(text) = self.rewrite(&tag)
            && text != self.src[tag.start..tag.end]
        {
            self.replace(tag.start, tag.end, &text);
        }
        Ok(tag)
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
            });
        }
    }

    /// Builds the rewritten start tag, or `None` when there is nothing to do.
    fn rewrite(&self, tag: &StartTag<'_>) -> Option<String> {
        if tag.attributes.is_empty() {
            return None;
        }
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
            text.push_str(tag.attributes[index].rest);
        }

        let multiline = text.contains('\n');
        if profile.layout.closing_bracket_newline && multiline {
            text.push_str(if text.contains("\r\n") { "\r\n" } else { "\n" });
            text.push_str(line_indent(self.src, tag.start));
        } else {
            text.push_str(tag.trailing);
        }
        text.push_str(if tag.self_closing { "/>" } else { ">" });
        Some(text)
    }
}

/// Leading whitespace of the line containing `offset`.
fn line_indent(src: &str, offset: usize) -> &str {
    let line_start = src[..offset].rfind('\n').map_or(0, |i| i + 1);
    let line = &src[line_start..];
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

fn find_ignore_case(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    let hay = haystack.as_bytes();
    let needle = needle.as_bytes();
    (from..=hay.len().checked_sub(needle.len())?)
        .find(|&i| hay[i..i + needle.len()].eq_ignore_ascii_case(needle))
}
