//! YAML-defined attribute ordering profiles.

use std::collections::HashSet;

use malva::config::{DeclarationOrder, DeclarationOrderGroupBy};
use serde::Deserialize;

use crate::class_order::ClassOrder;
use crate::error::ProfileError;
use crate::extends::{self, Builtins, Resolver};

const SCHEMA_VERSION: u32 = 1;

const HTML_PROFILE: &str = include_str!("../profiles/html.yaml");
const VUE_PROFILE: &str = include_str!("../profiles/vue.yaml");

/// The YAML source of a built-in profile.
pub(crate) fn builtin_yaml(name: &str) -> Option<&'static str> {
    match name {
        "html" => Some(HTML_PROFILE),
        "vue" => Some(VUE_PROFILE),
        _ => None,
    }
}

/// How attributes that fall into the same group are ordered relative to each other.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GroupSort {
    /// Keep the order in which the attributes appear in the source.
    #[default]
    Source,
    /// Sort by normalized attribute name.
    Alphabetical,
    /// Follow the order of the group's `match` patterns.
    Listed,
}

/// Attribute name rewrites applied before classification.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Normalize {
    /// `v-bind:foo` → `:foo`, `v-on:click` → `@click`.
    #[serde(default)]
    pub directive_shorthand: bool,
    /// camelCase → kebab-case for attributes of component tags.
    #[serde(default)]
    pub hyphenate_component_props: bool,
}

/// Start-tag layout rules.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Layout {
    /// Put the closing bracket of a multiline start tag on its own line.
    #[serde(default)]
    pub closing_bracket_newline: bool,
    /// In a start tag whose attributes span several lines, put each attribute on its own line.
    #[serde(default)]
    pub one_attribute_per_line: bool,
    /// Self-closing style per element kind.
    #[serde(default)]
    pub self_closing: SelfClosingRules,
    /// Line breaks around the content of multiline elements. Off when absent.
    #[serde(default)]
    pub content_newline: Option<ContentNewline>,
}

/// Puts the content of a multiline element on lines of its own, like
/// `vue/multiline-html-element-content-newline`.
///
/// An element is multiline when its end tag starts on a later line than its
/// start tag. Elements without content are left alone.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentNewline {
    /// Elements left alone, together with everything inside them. Element
    /// names match case-insensitively; component names match exactly.
    #[serde(default = "default_content_newline_ignore")]
    pub ignore: Vec<String>,
    /// Keep empty lines before and after the content instead of collapsing
    /// them to a single line break.
    #[serde(default)]
    pub allow_empty_lines: bool,
}

/// `vue/multiline-html-element-content-newline`'s default `ignores`: `pre`,
/// `textarea`, and inline elements, where added whitespace can be visible.
pub(crate) const CONTENT_NEWLINE_IGNORE: &[&str] = &[
    "pre", "textarea", "a", "abbr", "audio", "b", "bdi", "bdo", "canvas", "cite", "code", "data",
    "del", "dfn", "em", "i", "iframe", "ins", "kbd", "label", "map", "mark", "noscript", "object",
    "output", "picture", "q", "ruby", "s", "samp", "small", "span", "strong", "sub", "sup", "svg",
    "time", "u", "var", "video",
];

fn default_content_newline_ignore() -> Vec<String> {
    CONTENT_NEWLINE_IGNORE
        .iter()
        .map(|&name| name.to_owned())
        .collect()
}

/// Whether an element is written self-closing (`<br />`, `<MyComp />`).
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SelfClosing {
    /// Always self-close (`<br />`); empty elements lose their end tag.
    Always,
    /// Never self-close (`<br>`, `<MyComp></MyComp>`).
    Never,
    /// Keep whatever the source uses.
    #[default]
    Preserve,
}

/// Self-closing style per element kind, mirroring `vue/html-self-closing`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelfClosingRules {
    /// Void elements (`br`, `img`, `input`, …).
    #[serde(default)]
    pub void: SelfClosing,
    /// Empty non-void HTML/SVG elements. Vue templates only: in plain HTML,
    /// `<div />` is an unclosed start tag.
    #[serde(default)]
    pub normal: SelfClosing,
    /// Empty components. Vue templates only.
    #[serde(default)]
    pub component: SelfClosing,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawProfile {
    schema_version: u32,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    normalize: Normalize,
    #[serde(default)]
    layout: Layout,
    groups: Vec<RawGroup>,
    #[serde(default)]
    stylesheet: Option<RawStylesheet>,
    /// A built-in class order name, or an inline class order profile.
    #[serde(default)]
    class_order: Option<yaml_serde::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawStylesheet {
    #[serde(default)]
    declaration_order: Option<DeclarationOrder>,
    #[serde(default)]
    declaration_order_group_by: Option<DeclarationOrderGroupBy>,
    #[serde(default)]
    vendor_prefix: VendorPrefix,
    #[serde(default)]
    style_attribute: bool,
}

/// Where vendor-prefixed declarations (`-webkit-…`) go when declarations are sorted.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VendorPrefix {
    /// Next to the unprefixed property (malva's behavior).
    #[default]
    Alongside,
    /// Before all other declarations of the same run, in alphabetical order.
    ///
    /// Not at the end: prefixed properties are often aliases of the standard
    /// one, and the later declaration wins. Keeping them first lets the
    /// standard property win, as Autoprefixer's output does.
    Start,
}

/// Stylesheet options resolved from a profile.
#[derive(Clone, Debug)]
pub(crate) struct StylesheetOptions {
    /// Property order; without it, declarations are not reordered.
    pub declaration_order: Option<DeclarationOrder>,
    /// Whether a blank line also ends a run of sorted declarations.
    pub split_on_empty_line: bool,
    pub vendor_prefix: VendorPrefix,
    /// Also order the declarations of static `style` attributes.
    pub style_attribute: bool,
}

impl RawStylesheet {
    fn into_options(self) -> StylesheetOptions {
        // Unlike malva, a blank line ends a run by default: authors use blank
        // lines to group declarations.
        let group_by = self
            .declaration_order_group_by
            .unwrap_or(DeclarationOrderGroupBy::NonDeclarationAndEmptyLine);
        StylesheetOptions {
            declaration_order: self.declaration_order,
            split_on_empty_line: matches!(
                group_by,
                DeclarationOrderGroupBy::NonDeclarationAndEmptyLine
            ),
            vendor_prefix: self.vendor_prefix,
            style_attribute: self.style_attribute,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGroup {
    name: String,
    #[serde(default, rename = "match")]
    patterns: Vec<String>,
    #[serde(default)]
    sort: GroupSort,
    #[serde(default)]
    fallback: bool,
    #[serde(default, rename = "spreadSafe")]
    spread_safe: bool,
}

#[derive(Clone, Debug)]
enum Pattern {
    Exact(String),
    Prefix(String),
}

impl Pattern {
    /// Returns the match specificity: exact matches beat any prefix, longer prefixes beat shorter ones.
    fn specificity(&self, key: &str) -> Option<usize> {
        match self {
            Self::Exact(name) => (name == key).then_some(usize::MAX),
            Self::Prefix(prefix) => key.starts_with(prefix.as_str()).then_some(prefix.len()),
        }
    }
}

#[derive(Clone, Debug)]
struct Group {
    name: String,
    patterns: Vec<Pattern>,
    sort: GroupSort,
    /// Moving these attributes across a `v-bind="object"` spread cannot change behavior.
    spread_safe: bool,
}

/// Result of classifying one attribute key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Classification {
    /// Position of the group in the profile.
    pub group: usize,
    /// Index of the matching pattern inside the group (used by [`GroupSort::Listed`]).
    pub pattern: usize,
}

/// A validated attribute ordering profile.
#[derive(Clone, Debug)]
pub struct Profile {
    name: String,
    description: Option<String>,
    groups: Vec<Group>,
    fallback: usize,
    pub(crate) normalize: Normalize,
    pub(crate) layout: Layout,
    pub(crate) stylesheet: Option<StylesheetOptions>,
    pub(crate) class_order: Option<ClassOrder>,
}

impl Profile {
    /// Parses and validates a profile from YAML. `extends` may only name
    /// built-in profiles; see [`Profile::from_yaml_with`] for others.
    pub fn from_yaml(yaml: &str) -> Result<Self, ProfileError> {
        Self::from_yaml_with(yaml, &mut Builtins)
    }

    /// Parses and validates a profile from YAML, loading the profiles its
    /// `extends` names with `resolver`. Later entries of `extends` override
    /// earlier ones, and the profile itself overrides them all.
    pub fn from_yaml_with(yaml: &str, resolver: &mut dyn Resolver) -> Result<Self, ProfileError> {
        let raw: RawProfile = yaml_serde::from_value(extends::resolve(yaml, resolver)?)?;
        if raw.schema_version != SCHEMA_VERSION {
            return Err(ProfileError::SchemaVersion(raw.schema_version));
        }

        let fallbacks: Vec<usize> = raw
            .groups
            .iter()
            .enumerate()
            .filter_map(|(i, group)| group.fallback.then_some(i))
            .collect();
        let [fallback] = fallbacks[..] else {
            return Err(ProfileError::Fallback(fallbacks.len()));
        };

        if raw.name.is_empty() || raw.groups.iter().any(|group| group.name.is_empty()) {
            return Err(ProfileError::EmptyName);
        }

        let mut seen = HashSet::new();
        let mut groups = Vec::with_capacity(raw.groups.len());
        for group in raw.groups {
            if !seen.insert(group.name.clone()) {
                return Err(ProfileError::DuplicateGroup(group.name));
            }
            if group.patterns.is_empty() && !group.fallback {
                return Err(ProfileError::EmptyGroup(group.name));
            }
            let patterns = group
                .patterns
                .iter()
                .map(|pattern| compile_pattern(&group.name, pattern))
                .collect::<Result<_, _>>()?;
            groups.push(Group {
                name: group.name,
                patterns,
                sort: group.sort,
                spread_safe: group.spread_safe,
            });
        }

        Ok(Self {
            name: raw.name,
            description: raw.description,
            groups,
            fallback,
            normalize: raw.normalize,
            layout: raw.layout,
            stylesheet: raw.stylesheet.map(RawStylesheet::into_options),
            class_order: raw.class_order.map(class_order_of).transpose()?,
        })
    }

    /// Returns a built-in profile by name (`html` or `vue`).
    pub fn builtin(name: &str) -> Option<Self> {
        let yaml = builtin_yaml(name)?;
        Some(Self::from_yaml(yaml).expect("built-in profiles are valid"))
    }

    /// The YAML source of a built-in profile (`html` or `vue`).
    pub fn builtin_yaml(name: &str) -> Option<&'static str> {
        builtin_yaml(name)
    }

    /// The built-in plain HTML profile.
    pub fn html() -> Self {
        Self::builtin("html").expect("html profile exists")
    }

    /// The built-in Vue profile (superset of `vue/attributes-order`).
    pub fn vue() -> Self {
        Self::builtin("vue").expect("vue profile exists")
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Group names in priority order.
    pub fn group_names(&self) -> impl Iterator<Item = &str> {
        self.groups.iter().map(|group| group.name.as_str())
    }

    pub(crate) fn group_sort(&self, group: usize) -> GroupSort {
        self.groups[group].sort
    }

    pub(crate) fn group_spread_safe(&self, group: usize) -> bool {
        self.groups[group].spread_safe
    }

    /// Classifies a normalized, lowercased attribute key.
    pub(crate) fn classify(&self, key: &str) -> Classification {
        let mut best: Option<(usize, Classification)> = None;
        for (group_index, group) in self.groups.iter().enumerate() {
            for (pattern_index, pattern) in group.patterns.iter().enumerate() {
                let Some(score) = pattern.specificity(key) else {
                    continue;
                };
                if best.is_none_or(|(best_score, _)| score > best_score) {
                    best = Some((
                        score,
                        Classification {
                            group: group_index,
                            pattern: pattern_index,
                        },
                    ));
                }
            }
        }
        best.map_or(
            Classification {
                group: self.fallback,
                pattern: 0,
            },
            |(_, classification)| classification,
        )
    }
}

/// Resolves `classOrder`: a built-in name or an inline class order profile.
fn class_order_of(value: yaml_serde::Value) -> Result<ClassOrder, ProfileError> {
    match value {
        yaml_serde::Value::String(name) => {
            ClassOrder::builtin(&name).ok_or(ProfileError::UnknownClassOrder(name))
        }
        inline => ClassOrder::from_yaml(&yaml_serde::to_string(&inline)?),
    }
}

fn compile_pattern(group: &str, pattern: &str) -> Result<Pattern, ProfileError> {
    let pattern = pattern.to_ascii_lowercase();
    match pattern.find('*') {
        None => Ok(Pattern::Exact(pattern)),
        Some(i) if i == pattern.len() - 1 => Ok(Pattern::Prefix(pattern[..i].to_owned())),
        Some(_) => Err(ProfileError::Pattern {
            group: group.to_owned(),
            pattern,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group_of(profile: &Profile, key: &str) -> String {
        let group = profile.classify(key).group;
        profile.group_names().nth(group).unwrap().to_owned()
    }

    #[test]
    fn builtin_profiles_are_valid() {
        assert_eq!(Profile::html().name(), "html");
        assert_eq!(Profile::vue().name(), "vue");
    }

    #[test]
    fn exact_patterns_beat_prefixes() {
        let vue = Profile::vue();
        assert_eq!(group_of(&vue, "v-if"), "conditionals");
        assert_eq!(group_of(&vue, "v-on:click"), "events");
        assert_eq!(group_of(&vue, "v-model:title"), "two-way-binding");
        assert_eq!(group_of(&vue, "v-focus"), "other-directives");
        assert_eq!(group_of(&vue, "aria-expanded"), "aria-state");
        assert_eq!(group_of(&vue, "aria-controls"), "aria-property");
        assert_eq!(group_of(&vue, "disabled"), "other");
    }

    #[test]
    fn rejects_unknown_keys() {
        let yaml = "schemaVersion: 1\nname: x\ngroups:\n  - name: a\n    fallback: true\n    sortt: source\n";
        assert!(matches!(
            Profile::from_yaml(yaml),
            Err(ProfileError::Yaml(_))
        ));
    }

    #[test]
    fn builds_stylesheet_options() {
        let yaml = "schemaVersion: 1\nname: x\ngroups:\n  - name: a\n    fallback: true\nstylesheet:\n  declarationOrder: concentric\n";
        let options = Profile::from_yaml(yaml).unwrap().stylesheet.unwrap();
        assert!(matches!(
            options.declaration_order,
            Some(DeclarationOrder::Concentric)
        ));
        assert!(options.split_on_empty_line);
    }

    #[test]
    fn rejects_formatting_options() {
        // Formatting is left to a formatter; malva's options are not accepted.
        let yaml = "schemaVersion: 1\nname: x\ngroups:\n  - name: a\n    fallback: true\nstylesheet:\n  malva:\n    quotes: prefer-single\n";
        assert!(matches!(
            Profile::from_yaml(yaml),
            Err(ProfileError::Yaml(_))
        ));
    }

    #[test]
    fn rejects_bad_schema_and_structure() {
        let wrong_version = "schemaVersion: 2\nname: x\ngroups:\n  - name: a\n    fallback: true\n";
        assert!(matches!(
            Profile::from_yaml(wrong_version),
            Err(ProfileError::SchemaVersion(2))
        ));

        let no_fallback = "schemaVersion: 1\nname: x\ngroups:\n  - name: a\n    match: [id]\n";
        assert!(matches!(
            Profile::from_yaml(no_fallback),
            Err(ProfileError::Fallback(0))
        ));

        let bad_glob = "schemaVersion: 1\nname: x\ngroups:\n  - name: a\n    fallback: true\n  - name: b\n    match: ['*-x']\n";
        assert!(matches!(
            Profile::from_yaml(bad_glob),
            Err(ProfileError::Pattern { .. })
        ));
    }

    fn names(profile: &Profile) -> Vec<&str> {
        profile.group_names().collect()
    }

    #[test]
    fn extends_a_builtin_profile() {
        let yaml = concat!(
            "schemaVersion: 1\nname: bootstrap\nextends: html\n",
            "groups:\n",
            "  - name: bootstrap\n    match: [data-bs-toggle, data-bs-target, data-bs-*]\n",
            "    sort: listed\n    before: data\n",
        );
        let profile = Profile::from_yaml(yaml).unwrap();
        assert_eq!(profile.name(), "bootstrap");
        let groups = names(&profile);
        let data = groups.iter().position(|g| *g == "data").unwrap();
        assert_eq!(groups[data - 1], "bootstrap");
        assert_eq!(group_of(&profile, "data-bs-target"), "bootstrap");
        assert_eq!(group_of(&profile, "data-id"), "data");
        // Everything else is inherited.
        assert_eq!(group_of(&profile, "aria-label"), "aria-name");
        assert!(profile.layout.one_attribute_per_line);
        assert!(profile.stylesheet.is_some());
    }

    #[test]
    fn later_profiles_win() {
        let mut resolver = |spec: &str, _: Option<&str>| {
            let yaml = match spec {
                "a" => {
                    "schemaVersion: 1\nlayout:\n  oneAttributePerLine: true\n  closingBracketNewline: true\n"
                }
                "b" => "schemaVersion: 1\nlayout:\n  oneAttributePerLine: false\n",
                _ => return crate::extends::builtin(spec),
            };
            Some((spec.to_owned(), yaml.to_owned()))
        };
        let yaml = "schemaVersion: 1\nname: x\nextends: [html, a, b]\n";
        let profile = Profile::from_yaml_with(yaml, &mut resolver).unwrap();
        // `b` overrides `a`; mappings merge key by key.
        assert!(!profile.layout.one_attribute_per_line);
        assert!(profile.layout.closing_bracket_newline);

        let yaml = "schemaVersion: 1\nname: x\nextends: [html, b, a]\n";
        let profile = Profile::from_yaml_with(yaml, &mut resolver).unwrap();
        assert!(profile.layout.one_attribute_per_line);
    }

    #[test]
    fn updates_and_moves_inherited_groups() {
        let yaml = concat!(
            "schemaVersion: 1\nname: x\nextends: html\n",
            "groups:\n",
            "  - name: dimensions\n    match: [width, height, size]\n",
            "  - name: events\n    after: class\n",
        );
        let profile = Profile::from_yaml(yaml).unwrap();
        // Updated in place: `match` is replaced, the position kept.
        let groups = names(&profile);
        let dimensions = groups.iter().position(|g| *g == "dimensions").unwrap();
        assert_eq!(groups[dimensions - 1], "key-value");
        assert_eq!(group_of(&profile, "size"), "dimensions");
        assert_eq!(&names(&profile)[..3], ["global", "class", "events"]);
        assert_eq!(group_of(&profile, "onclick"), "events");
    }

    #[test]
    fn a_mixin_places_groups_among_earlier_profiles() {
        // `bs` doesn't extend `html`, yet places its group before `html`'s `data`.
        let mut resolver = |spec: &str, _: Option<&str>| {
            match spec {
            "bs" => Some((
                spec.to_owned(),
                "schemaVersion: 1\ngroups:\n  - name: bootstrap\n    match: [data-bs-*]\n    before: data\n"
                    .to_owned(),
            )),
            _ => crate::extends::builtin(spec),
        }
        };
        let yaml = "schemaVersion: 1\nname: x\nextends: [html, bs]\n";
        let profile = Profile::from_yaml_with(yaml, &mut resolver).unwrap();
        let groups = names(&profile);
        let data = groups.iter().position(|g| *g == "data").unwrap();
        assert_eq!(groups[data - 1], "bootstrap");
    }

    #[test]
    fn a_base_may_be_partial() {
        let mut resolver = |spec: &str, _: Option<&str>| {
            (spec == "layout").then(|| {
                (
                    spec.to_owned(),
                    "schemaVersion: 1\nlayout:\n  oneAttributePerLine: true\n".to_owned(),
                )
            })
        };
        let yaml = "schemaVersion: 1\nname: x\nextends: layout\ngroups:\n  - name: a\n    fallback: true\n";
        let profile = Profile::from_yaml_with(yaml, &mut resolver).unwrap();
        assert!(profile.layout.one_attribute_per_line);
    }

    #[test]
    fn rejects_bad_extends() {
        let unknown = "schemaVersion: 1\nname: x\nextends: nope\n";
        assert!(matches!(
            Profile::from_yaml(unknown),
            Err(ProfileError::UnknownProfile(name)) if name == "nope"
        ));

        let anchor = "schemaVersion: 1\nname: x\nextends: html\ngroups:\n  - name: b\n    match: [x]\n    before: nope\n";
        assert!(matches!(
            Profile::from_yaml(anchor),
            Err(ProfileError::Extends(_))
        ));

        let both = "schemaVersion: 1\nname: x\nextends: html\ngroups:\n  - name: b\n    match: [x]\n    before: data\n    after: data\n";
        assert!(matches!(
            Profile::from_yaml(both),
            Err(ProfileError::Extends(_))
        ));

        let mut cycle = |spec: &str, _: Option<&str>| {
            let other = if spec == "a" { "b" } else { "a" };
            Some((
                spec.to_owned(),
                format!("schemaVersion: 1\nextends: {other}\n"),
            ))
        };
        let yaml = "schemaVersion: 1\nname: x\nextends: a\n";
        assert!(matches!(
            Profile::from_yaml_with(yaml, &mut cycle),
            Err(ProfileError::Extends(_))
        ));

        // A base with two fallback groups after merging is still invalid.
        let two = "schemaVersion: 1\nname: x\nextends: html\ngroups:\n  - name: rest\n    fallback: true\n";
        assert!(matches!(
            Profile::from_yaml(two),
            Err(ProfileError::Fallback(2))
        ));
    }
}
