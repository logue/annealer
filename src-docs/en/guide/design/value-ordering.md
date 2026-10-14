---
description: Principles behind ordering the tokens inside static class attributes with library-specific class order profiles, and what is still open.
---

# Value ordering

:::info Status
Both are implemented: static `style` attributes (opt-in, see [Stylesheets](../concepts/stylesheets.md#static-style-attributes-opt-in)) and `class` values with a swappable class order (see [Class order](../features/class-order.md)), with a built-in `tailwind` order. A Bootstrap profile is not written yet. This page records the principles behind them.
:::

Attribute order is one axis. The order of the values inside an attribute is another. The need isn't specific to Tailwind: any class-based CSS framework (Bootstrap, Tailwind, a project's own utility set) benefits from a fixed class order, for the same diff-stability reason as attributes.

## Principles

### Static values only

Bound attributes (`:class`, `:style`, `v-bind:*`) hold expressions, not values. annealer never looks inside them.

This differs from `eslint-plugin-tailwindcss` (`classnames-order`) and `prettier-plugin-tailwindcss`, which also sort string literals inside `:class` expressions and `clsx()`-style calls.

### A library-agnostic mechanism, with library-specific profiles

The class order is plain profile data, like attribute groups, not a code branch. This is the first concrete case of [library profiles](./library-profiles.md).

### The library owns its class order

A framework's canonical class order is a design decision of that framework, not of annealer. annealer provides the mechanism and the profile format.

- **Bootstrap**: annealer proposes the mechanism upstream, with a draft profile (Bootstrap 6 is in development), so that Bootstrap itself defines and maintains the order.
- **Tailwind**: the order is already defined by Tailwind's engine. annealer ships it as built-in YAML (`tailwind`), generated from Tailwind and checked against it. It follows the spec of `eslint-plugin-tailwindcss` `classnames-order`, except that duplicates are kept (see below).

### The profile format is a public contract

Third parties are expected to write and ship profiles. Before the proposal, `schemaVersion`, the JSON Schema and the format documentation must be stable. Profiles distributed outside annealer also need a loading path (for example, `extends` resolving a file shipped in a package).

### An approximate, self-contained order

The official Tailwind order is derived from each project's own Tailwind build (v4 `@theme` and custom utilities, v3 `tailwind.config.js`). Only Tailwind itself can compute it. annealer defines the order in YAML instead:

- close to the official order for standard utilities,
- custom utilities are not known,
- no JavaScript runtime is required.

### Never change meaning

Reordering classes doesn't change CSS semantics. Duplicate classes are kept (removing them is a lint concern), and unknown classes are never dropped.

## Design

- **Classes are ranked by the CSS properties they set**, with Tailwind's comparison: unknown classes, variants, the first differing property in a property order, the number of properties, then the name. Properties are library-agnostic, so a Bootstrap profile can use the same format (`d-flex` sets `display`, `mt-3` sets `margin-top`).
- **Variants** (Tailwind's `md:`, `hover:`) are a separate axis: a separator and a ranked list of variant patterns.
- **Patterns** are richer than attribute patterns: `*` anywhere and `{name}` value sets, with the most specific match winning. `text-center`, `text-lg` and `text-red-500` resolve to different rules.
- **The class order is its own profile**, not part of the attribute profile, so it can be combined with any attribute profile: `--class-order`, the `classOrder` option, or `classOrder` in a profile.

## Resolved questions

- Where unknown classes go: `unknown: start` (Tailwind's order) or `end`.
- Whitespace inside a `class` value is preserved: classes move between the existing separators, as `eslint-plugin-tailwindcss` does.

## Open questions

- Infix breakpoints (Bootstrap's `col-md-6`, `d-lg-none`) are not a separate axis yet. A profile can only rank them through patterns.
- How attribute profiles compose (for example `extends: vue`). This also serves library-specific attribute rules.
