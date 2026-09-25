//! Computes the new attribute order for one start tag.

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::attribute::Key;
use crate::profile::{Classification, GroupSort, Profile};

/// Returns a permutation of `0..keys.len()` giving the output order.
///
/// `v-bind="object"` spreads split the list into segments; attributes are
/// never moved across a spread because that can change the evaluated result.
/// Attributes of `spreadSafe` groups are the exception: they are sorted into
/// the first segment.
pub(crate) fn attribute_order(keys: &[Key], profile: &Profile) -> Vec<usize> {
    let classes: Vec<Classification> = keys.iter().map(|key| profile.classify(&key.name)).collect();
    let movable = |i: usize| !keys[i].spread && profile.group_spread_safe(classes[i].group);

    let mut order = Vec::with_capacity(keys.len());
    let mut segment: Vec<usize> = (0..keys.len()).filter(|&i| movable(i)).collect();
    for i in 0..=keys.len() {
        if i == keys.len() || keys[i].spread {
            order.extend(sort_segment(
                std::mem::take(&mut segment),
                keys,
                &classes,
                profile,
            ));
            if i < keys.len() {
                order.push(i);
            }
        } else if !movable(i) {
            segment.push(i);
        }
    }
    order
}

fn sort_segment(
    mut indices: Vec<usize>,
    keys: &[Key],
    classes: &[Classification],
    profile: &Profile,
) -> Vec<usize> {
    indices.sort_unstable();
    // The first occurrence of a key anchors its cluster.
    let mut anchors: HashMap<&str, usize> = HashMap::new();
    for &i in &indices {
        anchors.entry(keys[i].name.as_str()).or_insert(i);
    }
    let anchor = |i: usize| anchors[keys[i].name.as_str()];

    indices.sort_by(|&a, &b| {
        let (ca, cb) = (classes[a], classes[b]);
        ca.group
            .cmp(&cb.group)
            .then_with(|| match profile.group_sort(ca.group) {
                GroupSort::Source => Ordering::Equal,
                GroupSort::Alphabetical => keys[a].name.cmp(&keys[b].name),
                GroupSort::Listed => ca.pattern.cmp(&cb.pattern),
            })
            .then_with(|| anchor(a).cmp(&anchor(b)))
            .then_with(|| cluster_order(&keys[a], &keys[b]))
            .then_with(|| a.cmp(&b))
    });
    indices
}

/// Order inside a same-name cluster.
///
/// Only `class` puts the static form before the bound form: Vue merges the two
/// into a class list where order has no effect. For every other name (notably
/// `style`, where the later declaration wins) the source order is kept.
fn cluster_order(a: &Key, b: &Key) -> Ordering {
    if a.name == "class" {
        a.bound.cmp(&b.bound)
    } else {
        Ordering::Equal
    }
}
