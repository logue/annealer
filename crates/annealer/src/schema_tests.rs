//! Keeps `schema/profile.schema.json` and the Rust profile loader in agreement.

use serde_json::Value;

use crate::profile::{MALVA_OPTIONS, Profile};

const SCHEMA: &str = include_str!("../schema/profile.schema.json");

fn schema() -> Value {
    serde_json::from_str(SCHEMA).unwrap()
}

fn validator() -> jsonschema::Validator {
    jsonschema::validator_for(&schema()).unwrap()
}

fn schema_accepts(yaml: &str) -> bool {
    let instance: Value = yaml_serde::from_str(yaml).unwrap();
    validator().is_valid(&instance)
}

/// A minimal valid profile followed by `extra` top-level YAML.
fn profile_with(extra: &str) -> String {
    format!("schemaVersion: 1\nname: x\ngroups:\n  - name: rest\n    fallback: true\n{extra}")
}

fn profile_with_groups(groups: &str) -> String {
    format!("schemaVersion: 1\nname: x\ngroups:\n  - name: rest\n    fallback: true\n{groups}")
}

#[test]
fn builtin_profiles_match_schema() {
    for yaml in [
        include_str!("../profiles/html.yaml"),
        include_str!("../profiles/vue.yaml"),
    ] {
        let instance: Value = yaml_serde::from_str(yaml).unwrap();
        let errors: Vec<String> = validator()
            .iter_errors(&instance)
            .map(|error| format!("{}: {error}", error.instance_path()))
            .collect();
        assert!(errors.is_empty(), "{errors:#?}");
    }
}

#[test]
fn malva_options_match_schema() {
    let schema = schema();
    let mut in_schema: Vec<&str> = schema["definitions"]["malvaOptions"]["properties"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let mut in_rust = MALVA_OPTIONS.to_vec();
    in_schema.sort_unstable();
    in_rust.sort_unstable();
    assert_eq!(in_schema, in_rust);
}

#[test]
fn schema_and_loader_agree() {
    let cases = [
        // (profile, valid)
        (profile_with(""), true),
        (
            profile_with(
                "description: d\nnormalize:\n  directiveShorthand: true\nlayout:\n  oneAttributePerLine: true\n",
            ),
            true,
        ),
        (
            profile_with(
                "stylesheet:\n  declarationOrder: concentric\n  malva:\n    printWidth: 100\n",
            ),
            true,
        ),
        (
            profile_with_groups("  - name: all\n    match: ['*']\n"),
            true,
        ),
        (profile_with("extra: 1\n"), false),
        (profile_with("normalize:\n  shorthand: true\n"), false),
        (
            profile_with("stylesheet:\n  declarationOrder: random\n"),
            false,
        ),
        (
            profile_with("stylesheet:\n  malva:\n    declarationOrder: smacss\n"),
            false,
        ),
        (
            profile_with("stylesheet:\n  malva:\n    indent_width: 4\n"),
            false,
        ),
        (profile_with_groups("  - name: a\n"), false),
        (profile_with_groups("  - name: a\n    match: []\n"), false),
        (
            profile_with_groups("  - name: a\n    match: ['*-x']\n"),
            false,
        ),
        (
            profile_with_groups("  - name: a\n    match: [id]\n    sort: random\n"),
            false,
        ),
        (
            profile_with_groups("  - name: ''\n    match: [id]\n"),
            false,
        ),
        (
            "schemaVersion: 2\nname: x\ngroups:\n  - name: a\n    fallback: true\n".to_owned(),
            false,
        ),
        (
            "schemaVersion: 1\nname: x\ngroups:\n  - name: a\n    match: [id]\n".to_owned(),
            false,
        ),
    ];
    for (yaml, valid) in cases {
        assert_eq!(schema_accepts(&yaml), valid, "schema disagrees on:\n{yaml}");
        assert_eq!(
            Profile::from_yaml(&yaml).is_ok(),
            valid,
            "loader disagrees on:\n{yaml}"
        );
    }
}

/// Rules JSON Schema draft-07 cannot express; only the loader enforces them.
#[test]
fn loader_is_stricter_where_schema_cannot_be() {
    let two_fallbacks = profile_with_groups("  - name: more\n    fallback: true\n");
    let duplicate_names = profile_with_groups("  - name: rest\n    match: [id]\n");
    for yaml in [two_fallbacks, duplicate_names] {
        assert!(schema_accepts(&yaml));
        assert!(Profile::from_yaml(&yaml).is_err());
    }
}
