//! JavaScript bindings, built with `wasm-pack` for the npm package.

use wasm_bindgen::prelude::*;

use crate::{Config, Language, Profile};

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
#[wasm_bindgen]
pub fn format(input: &str, language: &str, profile: Option<String>) -> Result<String, JsError> {
    let language = self::language(language)?;
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
    Ok(crate::format(input, &config)?)
}

/// The YAML text of a built-in profile, as a starting point for a custom one.
#[wasm_bindgen(js_name = builtinProfile)]
pub fn builtin_profile(name: &str) -> Option<String> {
    crate::profile::builtin_yaml(name).map(str::to_owned)
}
