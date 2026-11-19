//! `extends`: a profile built on top of other profiles.
//!
//! Profiles are merged as YAML before validation, so a partial profile (no
//! fallback group, no `groups` at all) is a valid base. `extends` lists the
//! bases in order and each one overrides the ones before it; the extending
//! profile overrides them all:
//!
//! - Mappings (`layout`, `normalize`, `stylesheet`, …) merge key by key.
//! - Scalars, lists and `classOrder` are replaced.
//! - `groups` merge by `name`: a group with a known name updates that group in
//!   place, key by key (its `match` list is replaced, not appended to); a new
//!   group is appended. `before`/`after` (a group name) place the group, new or
//!   existing, next to another one.

use yaml_serde::{Mapping, Value};

use crate::error::ProfileError;

/// Loads the profiles `extends` names.
///
/// `resolve(spec, from)` returns `(id, yaml)` for the profile `spec` refers
/// to, where `from` is the id of the profile that names it (`None` for the
/// profile passed in). The id identifies the profile for cycle detection and
/// is the `from` of the profiles it extends in turn (a file path, so relative
/// paths resolve against the extending file). Returning `None` means no such
/// profile.
pub trait Resolver {
    fn resolve(&mut self, spec: &str, from: Option<&str>) -> Option<(String, String)>;
}

impl<F> Resolver for F
where
    F: FnMut(&str, Option<&str>) -> Option<(String, String)>,
{
    fn resolve(&mut self, spec: &str, from: Option<&str>) -> Option<(String, String)> {
        self(spec, from)
    }
}

/// Built-in profiles only.
pub(crate) struct Builtins;

impl Resolver for Builtins {
    fn resolve(&mut self, spec: &str, _from: Option<&str>) -> Option<(String, String)> {
        builtin(spec)
    }
}

/// `(id, yaml)` of a built-in profile, as a [`Resolver`] returns it.
pub(crate) fn builtin(spec: &str) -> Option<(String, String)> {
    crate::profile::builtin_yaml(spec).map(|yaml| (format!("builtin:{spec}"), yaml.to_owned()))
}

const EXTENDS: &str = "extends";
const GROUPS: &str = "groups";
const CLASS_ORDER: &str = "classOrder";
const BEFORE: &str = "before";
const AFTER: &str = "after";

/// Parses `yaml` and merges in everything it extends.
///
/// The `extends` chain is flattened depth first into a list of layers (each
/// profile after the profiles it extends), and the layers are merged in order
/// onto one profile. So a mixin's `before`/`after` can name a group of any
/// profile listed before it, not only of the profiles it extends itself.
pub(crate) fn resolve(yaml: &str, resolver: &mut dyn Resolver) -> Result<Value, ProfileError> {
    let value: Value = yaml_serde::from_str(yaml)?;
    let Value::Mapping(own) = value else {
        // Not a profile; let validation report it.
        return Ok(value);
    };
    let mut merged = Mapping::new();
    apply(own, None, resolver, &mut Vec::new(), &mut merged)?;
    Ok(Value::Mapping(merged))
}

/// Merges the profiles `own` extends, then `own`, onto `merged`.
fn apply(
    mut own: Mapping,
    id: Option<&str>,
    resolver: &mut dyn Resolver,
    stack: &mut Vec<String>,
    merged: &mut Mapping,
) -> Result<(), ProfileError> {
    if let Some(version) = own.get("schemaVersion").and_then(Value::as_u64)
        && version != 1
    {
        return Err(ProfileError::SchemaVersion(
            u32::try_from(version).unwrap_or(u32::MAX),
        ));
    }
    let specs = match own.remove(EXTENDS) {
        None => Vec::new(),
        Some(Value::String(spec)) => vec![spec],
        Some(Value::Sequence(specs)) => specs
            .into_iter()
            .map(|spec| match spec {
                Value::String(spec) => Ok(spec),
                _ => Err(ProfileError::Extends(
                    "`extends` entries must be strings".to_owned(),
                )),
            })
            .collect::<Result<_, _>>()?,
        Some(_) => {
            return Err(ProfileError::Extends(
                "`extends` must be a string or a list of strings".to_owned(),
            ));
        }
    };

    for spec in specs {
        let (base_id, base_yaml) = resolver
            .resolve(&spec, id)
            .ok_or_else(|| ProfileError::UnknownProfile(spec.clone()))?;
        if stack.contains(&base_id) || id == Some(base_id.as_str()) {
            return Err(ProfileError::Extends(format!("`{spec}` extends itself")));
        }
        let in_base = |error: ProfileError| match error {
            ProfileError::Extends(message) => {
                ProfileError::Extends(format!("in `{spec}`: {message}"))
            }
            error => ProfileError::Extends(format!("in `{spec}`: {error}")),
        };
        let base = match yaml_serde::from_str(&base_yaml).map_err(|error| in_base(error.into()))? {
            Value::Mapping(base) => base,
            _ => return Err(in_base(ProfileError::Extends("not a mapping".to_owned()))),
        };
        stack.push(base_id.clone());
        apply(base, Some(&base_id), resolver, stack, merged).map_err(in_base)?;
        stack.pop();
    }
    merge(merged, own)
}

/// Merges `over` into `base`; `over` wins.
fn merge(base: &mut Mapping, over: Mapping) -> Result<(), ProfileError> {
    for (key, value) in over {
        let name = key.as_str().unwrap_or_default();
        if name == GROUPS {
            let Value::Sequence(groups) = value else {
                base.insert(key, value);
                continue;
            };
            let existing = match base.get_mut(GROUPS) {
                Some(Value::Sequence(existing)) => existing,
                _ => {
                    base.insert(key.clone(), Value::Sequence(Vec::new()));
                    match base.get_mut(GROUPS) {
                        Some(Value::Sequence(existing)) => existing,
                        _ => unreachable!(),
                    }
                }
            };
            merge_groups(existing, groups)?;
            continue;
        }
        match (base.get_mut(&key), value) {
            (Some(Value::Mapping(base)), Value::Mapping(over)) if name != CLASS_ORDER => {
                merge(base, over)?;
            }
            (_, value) => {
                base.insert(key, value);
            }
        }
    }
    Ok(())
}

fn group_name(group: &Value) -> Option<&str> {
    group.get("name").and_then(Value::as_str)
}

fn merge_groups(base: &mut Vec<Value>, over: Vec<Value>) -> Result<(), ProfileError> {
    // Names `over` itself declares: only inherited groups merge by name, so a
    // name repeated within one profile stays a duplicate for validation.
    let mut own = Vec::new();
    for group in over {
        let Value::Mapping(mut group) = group else {
            base.push(group);
            continue;
        };
        if let Some(name) = group.get("name").and_then(Value::as_str) {
            if own.iter().any(|own| own == name) {
                group.remove(BEFORE);
                group.remove(AFTER);
                base.push(Value::Mapping(group));
                continue;
            }
            own.push(name.to_owned());
        }
        let anchor = match (group.remove(BEFORE), group.remove(AFTER)) {
            (Some(_), Some(_)) => {
                return Err(ProfileError::Extends(format!(
                    "group `{}` sets both `before` and `after`",
                    group
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                )));
            }
            (Some(anchor), None) => Some((anchor, false)),
            (None, Some(anchor)) => Some((anchor, true)),
            (None, None) => None,
        };
        let name = group.get("name").and_then(Value::as_str).map(str::to_owned);
        let position = name
            .as_deref()
            .and_then(|name| base.iter().position(|g| group_name(g) == Some(name)));
        let merged = match position {
            Some(index) => {
                let Value::Mapping(mut existing) = base.remove(index) else {
                    unreachable!("groups with a name are mappings")
                };
                for (key, value) in group {
                    existing.insert(key, value);
                }
                if anchor.is_none() {
                    base.insert(index, Value::Mapping(existing));
                    continue;
                }
                existing
            }
            None => group,
        };
        let index = match anchor {
            None => base.len(),
            Some((anchor, after)) => {
                let anchor_name = anchor.as_str().unwrap_or_default();
                let index = base
                    .iter()
                    .position(|g| group_name(g) == Some(anchor_name))
                    .ok_or_else(|| {
                        ProfileError::Extends(format!(
                            "group `{}` is placed next to unknown group `{anchor_name}`",
                            name.as_deref().unwrap_or_default()
                        ))
                    })?;
                index + usize::from(after)
            }
        };
        base.insert(index, Value::Mapping(merged));
    }
    Ok(())
}
