//! JavaScript bindings, built with `wasm-pack` for the npm package.

use wasm_bindgen::prelude::*;

use crate::{ClassOrder, Config, Language, Profile};

fn language(name: &str) -> Result<Language, JsError> {
    match name {
        "html" => Ok(Language::Html),
        "vue" => Ok(Language::Vue),
        "css" => Ok(Language::Css),
        "scss" => Ok(Language::Scss),
        "sass" => Ok(Language::Sass),
        "less" => Ok(Language::Less),
        _ => Err(JsError::new(&format!("unknown language `{name}`"))),
    }
}

/// Formats `input`.
///
/// `profile` is a built-in profile name (`html`, `vue`) or a profile's YAML
/// text; when omitted, the built-in profile for `language` is used.
/// `class_order` is a built-in class order name (`tailwind`) or a class order
/// YAML text; it overrides the profile's `classOrder`. `self_closing` and
/// `content_newline` turn those layout rules on or off; when omitted, the
/// profile's setting is kept. `void_self_closing` writes void elements as
/// `<input />` (`true`) or `<input>` (`false`); `expand_non_void` turns
/// `<div />` into `<div></div>` in plain HTML; when omitted, the profile's
/// settings are kept. `remove_duplicate_classes` drops repeated
/// classes, with or without a class order; when omitted, the class order's
/// `removeDuplicates` is kept. `print_width` sets the line width of
/// stylesheets; when omitted, the profile's (80) is kept.
#[wasm_bindgen]
#[expect(
    clippy::too_many_arguments,
    reason = "positional for wasm-bindgen; `src/index.ts` exposes an options object"
)]
pub fn format(
    input: &str,
    language: &str,
    profile: Option<String>,
    class_order: Option<String>,
    self_closing: Option<bool>,
    content_newline: Option<bool>,
    remove_duplicate_classes: Option<bool>,
    print_width: Option<u32>,
    void_self_closing: Option<bool>,
    expand_non_void: Option<bool>,
) -> Result<String, JsError> {
    let language = self::language(language)?;
    if print_width == Some(0) {
        return Err(JsError::new("`printWidth` must be at least 1"));
    }
    let config = match profile {
        None => Config::for_language(language),
        Some(spec) => {
            let profile = match Profile::builtin(&spec) {
                Some(profile) => profile,
                None => Profile::from_yaml(&spec)?,
            };
            Config::new(language, profile)
        }
    };
    let config = match class_order {
        None => config,
        Some(spec) => config.with_class_order(Some(match ClassOrder::builtin(&spec) {
            Some(class_order) => class_order,
            None => ClassOrder::from_yaml(&spec)?,
        })),
    }
    .with_self_closing(self_closing)
    .with_void_self_closing(void_self_closing)
    .with_expand_non_void(expand_non_void)
    .with_content_newline(content_newline)
    .with_remove_duplicate_classes(remove_duplicate_classes)
    .with_print_width(print_width.map(|width| width as usize));
    Ok(crate::format(input, &config)?)
}

/// The YAML text of a built-in class order, as a starting point for a custom one.
#[wasm_bindgen(js_name = builtinClassOrder)]
pub fn builtin_class_order(name: &str) -> Option<String> {
    crate::class_order::builtin_yaml(name).map(str::to_owned)
}

/// The YAML text of a built-in profile, as a starting point for a custom one.
#[wasm_bindgen(js_name = builtinProfile)]
pub fn builtin_profile(name: &str) -> Option<String> {
    crate::profile::builtin_yaml(name).map(str::to_owned)
}
