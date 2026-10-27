//! Value ordering for static `class` attributes, driven by a class order
//! profile (YAML).
//!
//! A class is ranked by the CSS properties it sets, the way Tailwind CSS's
//! official order (`eslint-plugin-tailwindcss` `classnames-order`,
//! `prettier-plugin-tailwindcss`) ranks utilities:
//!
//! 1. Classes no rule matches (`unknown`) go first or last, in source order.
//! 2. Variants (`md:`, `hover:`): classes without variants first, then by the
//!    variant ranks, compared like a bit mask (the highest rank decides).
//! 3. The first property that differs, by its position in `propertyOrder`.
//!    A class with no property left sorts after one that has one.
//! 4. More properties (`count`) first.
//! 5. Name, comparing runs of digits as numbers (`p-2` before `p-10`).
//!
//! Nothing here is specific to Tailwind: the variant syntax, the property
//! order and the class patterns all come from the profile.

use std::cmp::Ordering;
use std::collections::HashMap;

use serde::Deserialize;

use crate::error::ProfileError;

const SCHEMA_VERSION: u32 = 1;

const TAILWIND: &str = include_str!("../class-orders/tailwind.yaml");

/// Name of the built-in value set matching a decimal number (`4`, `0.5`).
const NUMBER_SET: &str = "number";

/// Where classes that no rule matches go.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UnknownPosition {
    /// Before all known classes (Tailwind's order).
    #[default]
    Start,
    /// After all known classes.
    End,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawClassOrder {
    schema_version: u32,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    unknown: UnknownPosition,
    #[serde(default)]
    remove_duplicates: bool,
    #[serde(default)]
    important: Option<String>,
    #[serde(default)]
    arbitrary_properties: bool,
    #[serde(default)]
    variants: Option<RawVariants>,
    property_order: Vec<String>,
    #[serde(default)]
    values: HashMap<String, Vec<String>>,
    rules: Vec<RawRule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawVariants {
    separator: String,
    order: Vec<OneOrMany>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    #[serde(rename = "match")]
    patterns: Vec<String>,
    properties: Vec<String>,
    #[serde(default)]
    count: Option<usize>,
}

#[derive(Clone, Debug)]
enum Segment {
    Literal(String),
    /// Any run of characters, possibly empty.
    Star,
    /// One entry of a value set (index into `ClassOrder::sets`).
    Set(usize),
}

#[derive(Clone, Debug)]
struct Pattern {
    segments: Vec<Segment>,
    /// Literal text before the first wildcard; a cheap pre-check.
    prefix: String,
    specificity: Specificity,
}

/// Higher is more specific: exact patterns first, then by literal length,
/// then by the number of value sets.
type Specificity = (bool, usize, usize);

#[derive(Clone, Debug)]
enum ValueSet {
    Number,
    Patterns(Vec<Pattern>),
}

/// A class's position among the known classes.
#[derive(Clone, Debug)]
struct Rule {
    /// Positions in `propertyOrder`, ascending and unique.
    order: Vec<usize>,
    count: usize,
}

/// A validated class order profile.
#[derive(Clone, Debug)]
pub struct ClassOrder {
    name: String,
    description: Option<String>,
    unknown: UnknownPosition,
    remove_duplicates: bool,
    important: Option<String>,
    arbitrary_properties: bool,
    /// `propertyOrder` positions by property name.
    property_rank: HashMap<String, usize>,
    separator: Option<String>,
    /// `(pattern, rank)`.
    variants: Vec<(Pattern, usize)>,
    sets: Vec<ValueSet>,
    /// `(pattern, rule index)`, in profile order.
    patterns: Vec<(Pattern, usize)>,
    rules: Vec<Rule>,
}

/// The sort key of a class a rule matched.
struct Known {
    /// Variant ranks, descending and unique.
    variants: Vec<usize>,
    rule: Rule,
}

impl ClassOrder {
    /// Parses and validates a class order profile from YAML.
    pub fn from_yaml(yaml: &str) -> Result<Self, ProfileError> {
        let raw: RawClassOrder = yaml_serde::from_str(yaml)?;
        if raw.schema_version != SCHEMA_VERSION {
            return Err(ProfileError::SchemaVersion(raw.schema_version));
        }
        if raw.name.is_empty() {
            return Err(ProfileError::EmptyName);
        }
        let invalid = |message: String| ProfileError::ClassOrder(message);

        let mut set_names: Vec<&str> = raw.values.keys().map(String::as_str).collect();
        set_names.sort_unstable();
        if set_names.contains(&NUMBER_SET) {
            return Err(invalid(format!("value set `{NUMBER_SET}` is built in")));
        }
        let set_index = |name: &str| -> Option<usize> {
            if name == NUMBER_SET {
                Some(0)
            } else {
                set_names.binary_search(&name).ok().map(|index| index + 1)
            }
        };
        let mut sets = vec![ValueSet::Number];
        for name in &set_names {
            let patterns = raw.values[*name]
                .iter()
                .map(|pattern| compile(pattern, &|_| None))
                .collect::<Result<_, _>>()
                .map_err(|message| invalid(format!("value set `{name}`: {message}")))?;
            sets.push(ValueSet::Patterns(patterns));
        }

        let property_rank: HashMap<String, usize> = raw
            .property_order
            .iter()
            .enumerate()
            .map(|(index, property)| (property.clone(), index))
            .collect();
        if property_rank.len() != raw.property_order.len() {
            return Err(invalid("`propertyOrder` lists a property twice".to_owned()));
        }

        let mut rules = Vec::with_capacity(raw.rules.len());
        let mut patterns = Vec::new();
        for raw_rule in &raw.rules {
            if raw_rule.patterns.is_empty() {
                return Err(invalid("a rule has no `match` patterns".to_owned()));
            }
            let mut order = Vec::with_capacity(raw_rule.properties.len());
            for property in &raw_rule.properties {
                let rank = property_rank.get(property.as_str()).ok_or_else(|| {
                    invalid(format!("property `{property}` is not in `propertyOrder`"))
                })?;
                order.push(*rank);
            }
            order.sort_unstable();
            order.dedup();
            let count = raw_rule.count.unwrap_or(raw_rule.properties.len());
            for pattern in &raw_rule.patterns {
                let compiled = compile(pattern, &set_index)
                    .map_err(|message| invalid(format!("pattern `{pattern}`: {message}")))?;
                patterns.push((compiled, rules.len()));
            }
            rules.push(Rule { order, count });
        }

        let (separator, variants) = match raw.variants {
            None => (None, Vec::new()),
            Some(raw_variants) => {
                if raw_variants.separator.is_empty() {
                    return Err(invalid("`variants.separator` is empty".to_owned()));
                }
                let mut variants = Vec::new();
                for (rank, entry) in raw_variants.order.iter().enumerate() {
                    let names = match entry {
                        OneOrMany::One(name) => std::slice::from_ref(name),
                        OneOrMany::Many(names) => names.as_slice(),
                    };
                    for name in names {
                        let compiled = compile(name, &|_| None)
                            .map_err(|message| invalid(format!("variant `{name}`: {message}")))?;
                        variants.push((compiled, rank));
                    }
                }
                (Some(raw_variants.separator), variants)
            }
        };

        Ok(Self {
            name: raw.name,
            description: raw.description,
            unknown: raw.unknown,
            remove_duplicates: raw.remove_duplicates,
            important: raw.important.filter(|important| !important.is_empty()),
            arbitrary_properties: raw.arbitrary_properties,
            property_rank,
            separator,
            variants,
            sets,
            patterns,
            rules,
        })
    }

    /// A built-in class order profile by name (`tailwind`).
    pub fn builtin(name: &str) -> Option<Self> {
        let yaml = builtin_yaml(name)?;
        Some(Self::from_yaml(yaml).expect("built-in class orders are valid"))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Reorders the classes of a `class` attribute value, dropping repeated
    /// ones when the profile asks for it. Returns `None` when nothing changes.
    #[cfg(test)]
    pub(crate) fn sort_value(&self, value: &str) -> Option<String> {
        Classes {
            order: Some(self),
            remove_duplicates: self.remove_duplicates,
        }
        .rewrite(value)
    }

    /// Returns the classes in output order.
    fn sort<'c>(&self, classes: &[&'c str]) -> Vec<&'c str> {
        let keyed: Vec<(&str, Option<Known>)> = classes
            .iter()
            .map(|&class| (class, self.classify(class)))
            .collect();
        let (mut known, unknown): (Vec<_>, Vec<_>) =
            keyed.into_iter().partition(|(_, key)| key.is_some());
        known.sort_by(|(a, ka), (b, kb)| {
            let (ka, kb) = (ka.as_ref().expect("known"), kb.as_ref().expect("known"));
            self.compare(a, ka, b, kb)
        });
        let known = known.into_iter().map(|(class, _)| class);
        let unknown = unknown.into_iter().map(|(class, _)| class);
        match self.unknown {
            UnknownPosition::Start => unknown.chain(known).collect(),
            UnknownPosition::End => known.chain(unknown).collect(),
        }
    }

    fn compare(&self, a: &str, ka: &Known, b: &str, kb: &Known) -> Ordering {
        let (ra, rb) = (&ka.rule, &kb.rule);
        compare_variants(&ka.variants, &kb.variants)
            .then_with(|| {
                let first_difference = ra
                    .order
                    .iter()
                    .zip(&rb.order)
                    .position(|(x, y)| x != y)
                    .unwrap_or(ra.order.len().min(rb.order.len()));
                let rank = |order: &[usize]| order.get(first_difference).copied();
                match (rank(&ra.order), rank(&rb.order)) {
                    (Some(x), Some(y)) => x.cmp(&y),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => Ordering::Equal,
                }
            })
            .then_with(|| rb.count.cmp(&ra.count))
            .then_with(|| natural_cmp(a, b))
    }

    fn classify(&self, class: &str) -> Option<Known> {
        let parts = match &self.separator {
            Some(separator) => split_top_level(class, separator),
            None => vec![class],
        };
        let (&utility, variant_names) = parts.split_last()?;
        // `!` marks the utility, before or after it (`hover:!p-4`, `p-4!`).
        let utility = match &self.important {
            Some(important) => utility
                .strip_prefix(important.as_str())
                .or_else(|| utility.strip_suffix(important.as_str()))
                .unwrap_or(utility),
            None => utility,
        };

        let mut variants = Vec::with_capacity(variant_names.len());
        for name in variant_names {
            variants.push(self.variant_rank(name)?);
        }
        variants.sort_unstable_by(|a, b| b.cmp(a));
        variants.dedup();

        let whole = self.utility_rule(utility);
        // An opacity or line-height modifier (`bg-red-500/50`, `text-lg/7`),
        // unless the whole class is the better match (`w-1/2`).
        let without_modifier =
            last_top_level(utility, '/').and_then(|slash| self.utility_rule(&utility[..slash]));
        let rule = match (whole, without_modifier) {
            (Some(whole), Some(stripped)) if stripped.0 > whole.0 => Some(stripped),
            (whole, stripped) => whole.or(stripped),
        }
        .map(|(_, index)| self.rules[index].clone())
        .or_else(|| self.arbitrary_property(utility))?;
        Some(Known { variants, rule })
    }

    /// `[mask-type:luminance]`: one declaration of the named property.
    fn arbitrary_property(&self, utility: &str) -> Option<Rule> {
        if !self.arbitrary_properties {
            return None;
        }
        let inner = utility.strip_prefix('[')?.strip_suffix(']')?;
        let (property, value) = inner.split_once(':')?;
        let valid_name = !property.is_empty()
            && property
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if !valid_name || value.is_empty() {
            return None;
        }
        Some(Rule {
            order: self
                .property_rank
                .get(property)
                .copied()
                .into_iter()
                .collect(),
            count: 1,
        })
    }

    fn variant_rank(&self, name: &str) -> Option<usize> {
        best_match(
            self.variants.iter().map(|(pattern, rank)| (pattern, *rank)),
            name,
            &self.sets,
        )
        .map(|(_, rank)| rank)
    }

    /// The best rule for `utility`, with the match specificity.
    fn utility_rule(&self, utility: &str) -> Option<(Specificity, usize)> {
        if utility.is_empty() {
            return None;
        }
        best_match(
            self.patterns.iter().map(|(pattern, rule)| (pattern, *rule)),
            utility,
            &self.sets,
        )
    }
}

/// The YAML source of a built-in class order profile.
pub(crate) fn builtin_yaml(name: &str) -> Option<&'static str> {
    match name {
        "tailwind" => Some(TAILWIND),
        _ => None,
    }
}

/// What happens to the classes of static `class` attributes: a class order,
/// duplicate removal, or both.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Classes<'a> {
    pub(crate) order: Option<&'a ClassOrder>,
    /// Drop repeated classes, keeping the first.
    pub(crate) remove_duplicates: bool,
}

impl<'a> Classes<'a> {
    /// `remove_duplicates` overrides the class order's `removeDuplicates`.
    pub(crate) fn new(order: Option<&'a ClassOrder>, remove_duplicates: Option<bool>) -> Self {
        Self {
            order,
            remove_duplicates: remove_duplicates
                .unwrap_or_else(|| order.is_some_and(|order| order.remove_duplicates)),
        }
    }

    pub(crate) fn is_active(&self) -> bool {
        self.order.is_some() || self.remove_duplicates
    }

    /// Rewrites a `class` attribute value. Whitespace stays where it is:
    /// classes move between the separators. Returns `None` when nothing
    /// changes.
    pub(crate) fn rewrite(&self, value: &str) -> Option<String> {
        let mut classes: Vec<&str> = Vec::new();
        let mut spaces: Vec<&str> = Vec::new();
        let mut rest = value;
        loop {
            let space_end = rest.len() - rest.trim_start_matches(is_space).len();
            spaces.push(&rest[..space_end]);
            rest = &rest[space_end..];
            if rest.is_empty() {
                break;
            }
            let class_end = rest.find(is_space).unwrap_or(rest.len());
            classes.push(&rest[..class_end]);
            rest = &rest[class_end..];
        }
        // `spaces` has one more entry than `classes`: before each class, and
        // after the last one.

        let sorted = match self.order {
            Some(order) => order.sort(&classes),
            None => classes.clone(),
        };
        let mut kept: Vec<&str> = Vec::with_capacity(sorted.len());
        for class in sorted {
            if !(self.remove_duplicates && kept.contains(&class)) {
                kept.push(class);
            }
        }
        if kept == classes {
            return None;
        }

        // Removed duplicates take the separator before them along.
        let mut text = String::with_capacity(value.len());
        for (space, class) in spaces.iter().zip(&kept) {
            text.push_str(space);
            text.push_str(class);
        }
        text.push_str(spaces[spaces.len() - 1]);
        Some(text)
    }
}

fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0C')
}

/// The most specific pattern matching `text`; the first listed wins ties.
fn best_match<'p>(
    candidates: impl Iterator<Item = (&'p Pattern, usize)>,
    text: &str,
    sets: &[ValueSet],
) -> Option<(Specificity, usize)> {
    let mut best: Option<(&Pattern, usize)> = None;
    for (pattern, value) in candidates {
        if !text.starts_with(pattern.prefix.as_str())
            || best.is_some_and(|(current, _)| current.specificity >= pattern.specificity)
        {
            continue;
        }
        if matches(&pattern.segments, text, sets) {
            best = Some((pattern, value));
        }
    }
    best.map(|(pattern, value)| (pattern.specificity, value))
}

/// Compiles `*` (any run of characters) and `{name}` (an entry of a value
/// set) patterns.
fn compile(pattern: &str, set_index: &dyn Fn(&str) -> Option<usize>) -> Result<Pattern, String> {
    if pattern.is_empty() {
        return Err("empty pattern".to_owned());
    }
    let mut segments = Vec::new();
    let mut literal = String::new();
    let mut chars = pattern.char_indices().peekable();
    while let Some((index, c)) = chars.next() {
        match c {
            '*' => {
                if !literal.is_empty() {
                    segments.push(Segment::Literal(std::mem::take(&mut literal)));
                }
                if !matches!(segments.last(), Some(Segment::Star)) {
                    segments.push(Segment::Star);
                }
            }
            '{' => {
                let close = pattern[index..].find('}').ok_or("unclosed `{`")? + index;
                let name = &pattern[index + 1..close];
                let set = set_index(name).ok_or_else(|| format!("unknown value set `{name}`"))?;
                if !literal.is_empty() {
                    segments.push(Segment::Literal(std::mem::take(&mut literal)));
                }
                segments.push(Segment::Set(set));
                while chars.next_if(|&(i, _)| i <= close).is_some() {}
            }
            _ => literal.push(c),
        }
    }
    if !literal.is_empty() {
        segments.push(Segment::Literal(literal));
    }
    let prefix = match segments.first() {
        Some(Segment::Literal(text)) => text.clone(),
        _ => String::new(),
    };
    let literal_len = segments
        .iter()
        .map(|segment| match segment {
            Segment::Literal(text) => text.len(),
            _ => 0,
        })
        .sum();
    let set_count = segments
        .iter()
        .filter(|segment| matches!(segment, Segment::Set(_)))
        .count();
    let exact = segments.len() == 1 && matches!(segments[0], Segment::Literal(_));
    Ok(Pattern {
        segments,
        prefix,
        specificity: (exact, literal_len, set_count),
    })
}

fn matches(segments: &[Segment], text: &str, sets: &[ValueSet]) -> bool {
    let Some((first, rest)) = segments.split_first() else {
        return text.is_empty();
    };
    match first {
        Segment::Literal(literal) => text
            .strip_prefix(literal.as_str())
            .is_some_and(|remaining| matches(rest, remaining, sets)),
        Segment::Star => {
            rest.is_empty()
                || text
                    .char_indices()
                    .map(|(i, _)| i)
                    .chain(std::iter::once(text.len()))
                    .any(|i| matches(rest, &text[i..], sets))
        }
        Segment::Set(set) => text
            .char_indices()
            .map(|(i, _)| i)
            .skip(1)
            .chain(std::iter::once(text.len()))
            .any(|end| {
                set_contains(&sets[*set], &text[..end], sets) && matches(rest, &text[end..], sets)
            }),
    }
}

fn set_contains(set: &ValueSet, value: &str, sets: &[ValueSet]) -> bool {
    match set {
        ValueSet::Number => is_number(value),
        ValueSet::Patterns(patterns) => patterns
            .iter()
            .any(|pattern| matches(&pattern.segments, value, sets)),
    }
}

/// `4`, `0.5`, `12.25`.
fn is_number(value: &str) -> bool {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, "0"));
    !whole.is_empty()
        && !fraction.is_empty()
        && whole.bytes().all(|b| b.is_ascii_digit())
        && fraction.bytes().all(|b| b.is_ascii_digit())
}

/// Splits on `separator` outside `[]` and `()`.
fn split_top_level<'s>(text: &'s str, separator: &str) -> Vec<&'s str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    let mut i = 0;
    while i < text.len() {
        let rest = &text[i..];
        let c = rest.chars().next().expect("in bounds");
        match c {
            '[' | '(' => depth += 1,
            ']' | ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 && rest.starts_with(separator) => {
                parts.push(&text[start..i]);
                i += separator.len();
                start = i;
                continue;
            }
            _ => {}
        }
        i += c.len_utf8();
    }
    parts.push(&text[start..]);
    parts
}

/// Byte offset of the last `needle` outside `[]` and `()`.
fn last_top_level(text: &str, needle: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut last = None;
    for (i, c) in text.char_indices() {
        match c {
            '[' | '(' => depth += 1,
            ']' | ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 && c == needle => last = Some(i),
            _ => {}
        }
    }
    last
}

/// Variant masks compare like Tailwind's bigints: the highest rank decides.
fn compare_variants(a: &[usize], b: &[usize]) -> Ordering {
    for (x, y) in a.iter().zip(b) {
        match x.cmp(y) {
            Ordering::Equal => {}
            unequal => return unequal,
        }
    }
    a.len().cmp(&b.len())
}

/// Compares strings with runs of digits compared as numbers, like Tailwind's
/// `compare`.
fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut i = 0;
    while i < a.len() && i < b.len() {
        if a[i].is_ascii_digit() && b[i].is_ascii_digit() {
            let end = |s: &[u8]| i + s[i..].iter().take_while(|c| c.is_ascii_digit()).count();
            let (ea, eb) = (end(a), end(b));
            let (na, nb) = (&a[i..ea], &b[i..eb]);
            let numeric = |digits: &[u8]| {
                digits
                    .iter()
                    .fold(0f64, |value, d| value * 10.0 + f64::from(d - b'0'))
            };
            match numeric(na)
                .partial_cmp(&numeric(nb))
                .unwrap_or(Ordering::Equal)
            {
                Ordering::Equal => {}
                unequal => return unequal,
            }
            match na.cmp(nb) {
                Ordering::Equal => {}
                unequal => return unequal,
            }
            // Tailwind continues after the first digit of each run.
            i += 1;
            continue;
        }
        if a[i] != b[i] {
            return a[i].cmp(&b[i]);
        }
        i += 1;
    }
    a.len().cmp(&b.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tailwind() -> ClassOrder {
        ClassOrder::builtin("tailwind").unwrap()
    }

    #[test]
    fn ranks_arbitrary_properties_by_their_property() {
        assert_eq!(
            tailwind()
                .sort_value("hover:!m-2 [mask-type:luminance] p-4 [--x:1]")
                .unwrap(),
            // `mask-type` precedes `padding` in Tailwind's property order.
            "[mask-type:luminance] p-4 [--x:1] hover:!m-2"
        );
    }

    #[test]
    fn sorts_like_the_eslint_plugin_example() {
        assert_eq!(
            tailwind()
                .sort_value("p-3 border-gray-300 m-4 h-24 lg:p-4 flex border-2 lg:m-4")
                .unwrap(),
            "m-4 flex h-24 border-2 border-gray-300 p-3 lg:m-4 lg:p-4"
        );
    }

    #[test]
    fn keeps_whitespace_in_place() {
        assert_eq!(
            tailwind().sort_value("  p-4\n    m-2  flex ").unwrap(),
            "  m-2\n    flex  p-4 "
        );
        assert_eq!(tailwind().sort_value("m-2 flex p-4"), None);
        assert_eq!(tailwind().sort_value(""), None);
    }

    #[test]
    fn puts_unknown_classes_first_in_source_order() {
        assert_eq!(
            tailwind().sort_value("p-4 card m-2 js-toggle").unwrap(),
            "card js-toggle m-2 p-4"
        );
    }

    #[test]
    fn handles_modifiers_important_negative_and_arbitrary_values() {
        assert_eq!(
            tailwind()
                .sort_value("text-lg/7 bg-red-500/50 p-4! -mt-4 w-[37px] text-[#f00]")
                .unwrap(),
            "-mt-4 w-[37px] bg-red-500/50 p-4! text-lg/7 text-[#f00]"
        );
    }

    #[test]
    fn natural_order_compares_numbers() {
        assert_eq!(natural_cmp("p-2", "p-10"), Ordering::Less);
        assert_eq!(natural_cmp("p-10", "p-10"), Ordering::Equal);
        assert_eq!(natural_cmp("a", "ab"), Ordering::Less);
    }

    #[test]
    fn removes_duplicates_when_asked() {
        let yaml = TAILWIND.replace("removeDuplicates: false", "removeDuplicates: true");
        let order = ClassOrder::from_yaml(&yaml).unwrap();
        assert_eq!(order.sort_value("p-4 m-2 p-4").unwrap(), "m-2 p-4");
        assert_eq!(tailwind().sort_value("p-4 m-2 p-4").unwrap(), "m-2 p-4 p-4");
    }

    #[test]
    fn removes_duplicates_without_a_class_order() {
        let classes = Classes::new(None, Some(true));
        assert_eq!(classes.rewrite("b a b  c a").unwrap(), "b a c");
        assert_eq!(classes.rewrite("a b"), None);
        assert!(!Classes::new(None, None).is_active());
    }

    #[test]
    fn the_override_wins_over_the_class_order() {
        let yaml = TAILWIND.replace("removeDuplicates: false", "removeDuplicates: true");
        let order = ClassOrder::from_yaml(&yaml).unwrap();
        assert_eq!(
            Classes::new(Some(&order), Some(false))
                .rewrite("p-4 p-4")
                .as_deref(),
            None
        );
        let tailwind = tailwind();
        assert_eq!(
            Classes::new(Some(&tailwind), Some(true))
                .rewrite("p-4 m-2 p-4")
                .unwrap(),
            "m-2 p-4"
        );
    }

    #[test]
    fn rejects_invalid_profiles() {
        let base = "schemaVersion: 1\nname: x\npropertyOrder: [display]\n";
        let cases = [
            "rules:\n  - match: [d-*]\n    properties: [margin]\n",
            "rules:\n  - match: []\n    properties: [display]\n",
            "rules:\n  - match: ['d-{size}']\n    properties: [display]\n",
            "values:\n  number: ['1']\nrules: []\n",
        ];
        for case in cases {
            assert!(
                ClassOrder::from_yaml(&format!("{base}{case}")).is_err(),
                "{case}"
            );
        }
        let ok =
            format!("{base}rules:\n  - match: [d-*, 'd-{{number}}']\n    properties: [display]\n");
        assert!(ClassOrder::from_yaml(&ok).is_ok());
    }
}
