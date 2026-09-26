//! YAML-defined attribute ordering profiles.

use std::collections::HashSet;

use malva::config::{DeclarationOrder, DeclarationOrderGroupBy, FormatOptions};
use serde::Deserialize;

use crate::error::ProfileError;

const SCHEMA_VERSION: u32 = 1;

const HTML_PROFILE: &str = include_str!("../profiles/html.yaml");
const VUE_PROFILE: &str = include_str!("../profiles/vue.yaml");

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
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawStylesheet {
    #[serde(default)]
    declaration_order: Option<DeclarationOrder>,
    #[serde(default)]
    declaration_order_group_by: Option<DeclarationOrderGroupBy>,
    #[serde(default)]
    malva: Option<yaml_serde::Mapping>,
}

/// malva options accepted under `stylesheet.malva`. Kept in sync with
/// `schema/profile.schema.json` by a test. `declarationOrder` and
/// `declarationOrderGroupBy` are deliberately absent: they live on
/// `stylesheet` itself.
pub(crate) const MALVA_OPTIONS: &[&str] = &[
    "printWidth",
    "useTabs",
    "indentWidth",
    "lineBreak",
    "hexCase",
    "hexColorLength",
    "quotes",
    "attrSelector.quotes",
    "operatorLineBreak",
    "blockSelectorLineBreak",
    "omitNumberLeadingZero",
    "trailingComma",
    "formatComments",
    "alignComments",
    "lineBreakInPseudoParens",
    "singleLineBlockThreshold",
    "keyframeSelectorNotation",
    "attrValueQuotes",
    "preferSingleLine",
    "selectors.preferSingleLine",
    "functionArgs.preferSingleLine",
    "sassContentAtRule.preferSingleLine",
    "sassIncludeAtRule.preferSingleLine",
    "sassMap.preferSingleLine",
    "sassModuleConfig.preferSingleLine",
    "sassParams.preferSingleLine",
    "lessImportOptions.preferSingleLine",
    "lessMixinArgs.preferSingleLine",
    "lessMixinParams.preferSingleLine",
    "singleLineTopLevelDeclarations",
    "fontFamilyNames",
    "nthPlusSpacing",
    "selectorOverrideCommentDirective",
    "ignoreCommentDirective",
    "ignoreFileCommentDirective",
];

impl RawStylesheet {
    /// Builds malva options, rejecting keys malva would silently ignore.
    fn into_options(self) -> Result<FormatOptions, ProfileError> {
        let mut options = match self.malva {
            None => FormatOptions::default(),
            Some(mapping) => {
                for key in mapping.keys() {
                    let key = key.as_str().unwrap_or_default();
                    if !MALVA_OPTIONS.contains(&key) {
                        return Err(ProfileError::MalvaOption(key.to_owned()));
                    }
                }
                yaml_serde::from_value(yaml_serde::Value::Mapping(mapping))?
            }
        };
        options.language.declaration_order = self.declaration_order;
        if let Some(group_by) = self.declaration_order_group_by {
            options.language.declaration_order_group_by = group_by;
        }
        Ok(options)
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
    pub(crate) stylesheet: Option<FormatOptions>,
}

impl Profile {
    /// Parses and validates a profile from YAML.
    pub fn from_yaml(yaml: &str) -> Result<Self, ProfileError> {
        let raw: RawProfile = yaml_serde::from_str(yaml)?;
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
            stylesheet: raw
                .stylesheet
                .map(RawStylesheet::into_options)
                .transpose()?,
        })
    }

    /// Returns a built-in profile by name (`html` or `vue`).
    pub fn builtin(name: &str) -> Option<Self> {
        let yaml = match name {
            "html" => HTML_PROFILE,
            "vue" => VUE_PROFILE,
            _ => return None,
        };
        Some(Self::from_yaml(yaml).expect("built-in profiles are valid"))
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
        let yaml = "schemaVersion: 1\nname: x\ngroups:\n  - name: a\n    fallback: true\nstylesheet:\n  declarationOrder: concentric\n  malva:\n    indentWidth: 4\n    quotes: prefer-single\n";
        let options = Profile::from_yaml(yaml).unwrap().stylesheet.unwrap();
        assert!(matches!(
            options.language.declaration_order,
            Some(DeclarationOrder::Concentric)
        ));
        assert_eq!(options.layout.indent_width, 4);
    }

    #[test]
    fn rejects_unknown_malva_options() {
        let misplaced = "schemaVersion: 1\nname: x\ngroups:\n  - name: a\n    fallback: true\nstylesheet:\n  malva:\n    declarationOrder: smacss\n";
        assert!(matches!(
            Profile::from_yaml(misplaced),
            Err(ProfileError::MalvaOption(key)) if key == "declarationOrder"
        ));
        let snake_case = "schemaVersion: 1\nname: x\ngroups:\n  - name: a\n    fallback: true\nstylesheet:\n  malva:\n    indent_width: 4\n";
        assert!(matches!(
            Profile::from_yaml(snake_case),
            Err(ProfileError::MalvaOption(_))
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
}
