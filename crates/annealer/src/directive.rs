//! `annealer-*` comment directives.
//!
//! ```text
//! <!-- annealer-disable -->                    /* annealer-disable */
//! <!-- annealer-enable -->                     /* annealer-enable */
//! <!-- annealer-disable-next-line: reason -->  /* annealer-disable-next-line: reason */
//! ```
//!
//! Only block comments are recognized: CSS has no `//` comments.

const PREFIX: &str = "annealer-";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Directive {
    /// Leave everything after this comment untouched, up to `annealer-enable`
    /// or the end of the document.
    Disable,
    /// Ends an `annealer-disable` range.
    Enable,
    /// Leave constructs that start on the next line untouched.
    DisableNextLine,
}

/// Parses a comment's content. `Ok(None)` for ordinary comments; `Err` for
/// malformed `annealer-*` directives, so typos don't silently do nothing.
pub(crate) fn parse(content: &str) -> Result<Option<Directive>, String> {
    let content = content.trim();
    if !content.starts_with(PREFIX) {
        return Ok(None);
    }
    let name_end = content
        .find(|c: char| c == ':' || c.is_whitespace())
        .unwrap_or(content.len());
    let (name, rest) = content.split_at(name_end);
    let directive = match name {
        "annealer-disable" => Directive::Disable,
        "annealer-enable" => Directive::Enable,
        "annealer-disable-next-line" => Directive::DisableNextLine,
        _ => return Err(format!("unknown directive `{name}`")),
    };
    let rest = rest.trim_start();
    if rest.is_empty() || rest.starts_with(':') {
        Ok(Some(directive))
    } else {
        Err(format!("expected `:` before the reason in `{name}`"))
    }
}

/// Where directives disable formatting in one document.
#[derive(Debug, Default)]
pub(crate) struct Disabled {
    /// `(offset, disabled)`: state changes, in document order.
    switches: Vec<(usize, bool)>,
    /// 0-based lines whose constructs are left untouched.
    lines: Vec<usize>,
    line_starts: Vec<usize>,
}

impl Disabled {
    pub(crate) fn new(src: &str) -> Self {
        let line_starts = std::iter::once(0)
            .chain(src.match_indices('\n').map(|(i, _)| i + 1))
            .collect();
        Self {
            switches: Vec::new(),
            lines: Vec::new(),
            line_starts,
        }
    }

    pub(crate) fn line_of(&self, offset: usize) -> usize {
        self.line_starts.partition_point(|&start| start <= offset) - 1
    }

    /// Records a directive whose comment ends at `comment_end`.
    pub(crate) fn add(&mut self, directive: Directive, comment_end: usize) {
        match directive {
            Directive::Disable => self.switches.push((comment_end, true)),
            Directive::Enable => self.switches.push((comment_end, false)),
            Directive::DisableNextLine => self.lines.push(self.line_of(comment_end) + 1),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.switches.iter().all(|&(_, disabled)| !disabled) && self.lines.is_empty()
    }

    /// Whether a construct starting at `offset` must be left untouched.
    pub(crate) fn covers(&self, offset: usize) -> bool {
        let in_range = self
            .switches
            .iter()
            .take_while(|&&(at, _)| at <= offset)
            .last()
            .is_some_and(|&(_, disabled)| disabled);
        in_range || self.lines.contains(&self.line_of(offset))
    }

    /// Whether everything from `offset` to the end is disabled.
    pub(crate) fn covers_rest(&self, offset: usize) -> bool {
        self.covers(offset)
            && self
                .switches
                .iter()
                .all(|&(at, disabled)| at <= offset || disabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_directives_and_reasons() {
        assert_eq!(parse(" annealer-disable "), Ok(Some(Directive::Disable)));
        assert_eq!(parse("annealer-enable"), Ok(Some(Directive::Enable)));
        assert_eq!(
            parse("annealer-disable-next-line: keep legacy order"),
            Ok(Some(Directive::DisableNextLine))
        );
        assert_eq!(
            parse("annealer-disable : reason"),
            Ok(Some(Directive::Disable))
        );
        assert_eq!(parse("just a comment"), Ok(None));
        assert_eq!(parse("see annealer-disable"), Ok(None));
    }

    #[test]
    fn rejects_malformed_directives() {
        assert!(parse("annealer-disabel").is_err());
        assert!(parse("annealer-disable-next-line reason without colon").is_err());
    }

    #[test]
    fn tracks_ranges_and_lines() {
        let src = "a\n<!-- d -->\nb\n<!-- e -->\nc\n<!-- n -->\nd\ne";
        let mut disabled = Disabled::new(src);
        disabled.add(Directive::Disable, src.find("<!-- d -->").unwrap() + 10);
        disabled.add(Directive::Enable, src.find("<!-- e -->").unwrap() + 10);
        disabled.add(
            Directive::DisableNextLine,
            src.find("<!-- n -->").unwrap() + 10,
        );
        let at = |needle: &str| src.rfind(needle).unwrap();
        assert!(!disabled.covers(at("a")));
        assert!(disabled.covers(at("b")));
        assert!(!disabled.covers(at("c")));
        assert!(disabled.covers(at("d\n")));
        assert!(!disabled.covers(at("e")));
    }
}
