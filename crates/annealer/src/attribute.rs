//! Attribute name normalization and classification keys.

use crate::profile::Normalize;

/// What the ordering logic needs to know about one attribute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Key {
    /// Lowercased key used for classification and clustering.
    pub name: String,
    /// Bound form (`:foo`, `v-bind:foo`, `.foo`) of an attribute.
    pub bound: bool,
    /// `v-bind="object"` spread; a reordering boundary.
    pub spread: bool,
}

/// Derives the classification key from an attribute name.
///
/// Binding prefixes and modifiers are stripped so that `class` and `:class`
/// share a key. Events map to `v-on:<event>` and slots to `v-slot:<name>`.
pub(crate) fn key_of(name: &str) -> Key {
    let bound = |arg: &str| Key {
        name: strip_modifiers(arg).to_ascii_lowercase(),
        bound: true,
        spread: false,
    };
    let directive = |prefix: &str, arg: &str| Key {
        name: format!("{prefix}:{}", strip_modifiers(arg).to_ascii_lowercase()),
        bound: false,
        spread: false,
    };

    if let Some(arg) = name.strip_prefix("v-bind:") {
        return bound(arg);
    }
    if let Some(arg) = name.strip_prefix(':').or_else(|| name.strip_prefix('.'))
        && !arg.is_empty()
    {
        return bound(arg);
    }
    if let Some(arg) = name
        .strip_prefix("v-on:")
        .or_else(|| name.strip_prefix('@'))
    {
        return directive("v-on", arg);
    }
    if let Some(arg) = name
        .strip_prefix("v-slot:")
        .or_else(|| name.strip_prefix('#'))
    {
        return directive("v-slot", arg);
    }
    if name.starts_with("v-") {
        let name = strip_modifiers(name).to_ascii_lowercase();
        let spread = name == "v-bind";
        return Key {
            name,
            bound: false,
            spread,
        };
    }
    Key {
        name: name.to_ascii_lowercase(),
        bound: false,
        spread: false,
    }
}

/// Removes `.modifier` suffixes, keeping dynamic arguments like `[a.b]` intact.
fn strip_modifiers(name: &str) -> &str {
    let mut depth = 0usize;
    for (i, c) in name.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            '.' if depth == 0 && i > 0 => return &name[..i],
            _ => {}
        }
    }
    name
}

/// Applies the profile's name rewrites and returns the new attribute name.
pub(crate) fn normalize_name(name: &str, component: bool, options: &Normalize) -> String {
    let mut name = name.to_owned();

    if options.directive_shorthand {
        if let Some(arg) = name.strip_prefix("v-bind:").filter(|arg| !arg.is_empty()) {
            name = format!(":{arg}");
        } else if let Some(arg) = name.strip_prefix("v-on:").filter(|arg| !arg.is_empty()) {
            name = format!("@{arg}");
        }
    }

    if options.hyphenate_component_props && component {
        let (prefix, rest) = match name.strip_prefix(':') {
            Some(rest) => (":", rest),
            None => ("", name.as_str()),
        };
        let hyphenatable = !rest.is_empty()
            && !rest.starts_with(['@', '#', ':', '.', '['])
            && !rest.starts_with("v-")
            && !rest.starts_with("data-")
            && !rest.starts_with("aria-");
        if hyphenatable {
            let arg = strip_modifiers(rest);
            let modifiers = &rest[arg.len()..];
            name = format!("{prefix}{}{modifiers}", hyphenate(arg));
        }
    }

    name
}

/// camelCase → kebab-case, matching Vue's `hyphenate` (`/\B([A-Z])/g`).
fn hyphenate(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    let mut prev_is_word = false;
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            if prev_is_word {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
        prev_is_word = c.is_ascii_alphanumeric() || c == '_';
    }
    out
}

/// SVG/MathML elements whose names look like components but are native.
const NATIVE_LOOKALIKES: &[&str] = &[
    "altGlyph",
    "altGlyphDef",
    "altGlyphItem",
    "animateColor",
    "animateMotion",
    "animateTransform",
    "annotation-xml",
    "clipPath",
    "color-profile",
    "feBlend",
    "feColorMatrix",
    "feComponentTransfer",
    "feComposite",
    "feConvolveMatrix",
    "feDiffuseLighting",
    "feDisplacementMap",
    "feDistantLight",
    "feDropShadow",
    "feFlood",
    "feFuncA",
    "feFuncB",
    "feFuncG",
    "feFuncR",
    "feGaussianBlur",
    "feImage",
    "feMerge",
    "feMergeNode",
    "feMorphology",
    "feOffset",
    "fePointLight",
    "feSpecularLighting",
    "feSpotLight",
    "feTile",
    "feTurbulence",
    "font-face",
    "font-face-format",
    "font-face-name",
    "font-face-src",
    "font-face-uri",
    "foreignObject",
    "glyphRef",
    "linearGradient",
    "missing-glyph",
    "radialGradient",
    "textPath",
];

/// Whether a tag name refers to a (Vue or custom-element) component.
pub(crate) fn is_component(tag: &str) -> bool {
    (tag.contains('-') || tag.chars().any(|c| c.is_ascii_uppercase()))
        && !NATIVE_LOOKALIKES.contains(&tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(name: &str) -> String {
        key_of(name).name
    }

    #[test]
    fn strips_binding_prefixes_and_modifiers() {
        assert_eq!(key("class"), "class");
        assert_eq!(key(":class"), "class");
        assert_eq!(key("v-bind:class"), "class");
        assert_eq!(key(":text-content.prop"), "text-content");
        assert_eq!(key(".textContent"), "textcontent");
        assert!(key_of(":class").bound);
        assert!(!key_of("class").bound);
    }

    #[test]
    fn maps_events_slots_and_directives() {
        assert_eq!(key("@click.prevent"), "v-on:click");
        assert_eq!(key("v-on:click"), "v-on:click");
        assert_eq!(key("v-on"), "v-on");
        assert_eq!(key("#default"), "v-slot:default");
        assert_eq!(key("v-slot"), "v-slot");
        assert_eq!(key("v-model.trim"), "v-model");
        assert_eq!(key("v-model:title"), "v-model:title");
        assert_eq!(key(":[dynamic.key]"), "[dynamic.key]");
    }

    #[test]
    fn detects_spread() {
        assert!(key_of("v-bind").spread);
        assert!(key_of("v-bind.prop").spread);
        assert!(!key_of("v-bind:foo").spread);
    }

    #[test]
    fn normalizes_shorthand_and_hyphenation() {
        let all = Normalize {
            directive_shorthand: true,
            hyphenate_component_props: true,
        };
        assert_eq!(normalize_name("v-bind:foo", false, &all), ":foo");
        assert_eq!(
            normalize_name("v-on:click.stop", false, &all),
            "@click.stop"
        );
        assert_eq!(normalize_name("v-bind", false, &all), "v-bind");
        assert_eq!(normalize_name(":fooBar", true, &all), ":foo-bar");
        assert_eq!(
            normalize_name("v-bind:fooBar.camel", true, &all),
            ":foo-bar.camel"
        );
        assert_eq!(normalize_name("myProp", true, &all), "my-prop");
        assert_eq!(normalize_name("myProp", false, &all), "myProp");
        assert_eq!(normalize_name("@updateValue", true, &all), "@updateValue");
        assert_eq!(normalize_name(":[dynamicKey]", true, &all), ":[dynamicKey]");
    }

    #[test]
    fn recognizes_components() {
        assert!(is_component("MyButton"));
        assert!(is_component("my-button"));
        assert!(!is_component("div"));
        assert!(!is_component("linearGradient"));
        assert!(!is_component("font-face"));
    }
}
