---
description: Planned ordering of tokens inside static class attributes, defined by library-specific profiles such as Tailwind and Bootstrap.
---

# Value ordering

:::info Status
Static `style` attributes are implemented (opt-in, see [Stylesheets](../concepts/stylesheets.md#static-style-attributes-opt-in)). Ordering of `class` values is **planned** and not implemented yet. This page records the agreed principles.
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
- **Tailwind**: the order is already defined by Tailwind's engine. annealer can ship a built-in YAML approximation of it.

### The profile format is a public contract

Third parties are expected to write and ship profiles. Before the proposal, `schemaVersion`, the JSON Schema and the format documentation must be stable. Profiles distributed outside annealer also need a loading path (for example, `extends` resolving a file shipped in a package).

### An approximate, self-contained order

The official Tailwind order is derived from each project's own Tailwind build (v4 `@theme` and custom utilities, v3 `tailwind.config.js`). Only Tailwind itself can compute it. annealer defines the order in YAML instead:

- close to the official order for standard utilities,
- custom utilities are not known,
- no JavaScript runtime is required.

### Never change meaning

Reordering classes doesn't change CSS semantics. Duplicate classes are kept (removing them is a lint concern), and unknown classes are never dropped.

## Sketch (to be designed)

- Split the `class` value on whitespace, classify each token with group patterns (as with attribute groups), sort by group, then by the group's `sort` mode. Source order is the tie-breaker.
- Variants and breakpoints need their own axis:
  - Tailwind uses **prefixes** (`md:`, `hover:`, `dark:`). The sort key is the base class plus the variant order.
  - Bootstrap uses **infixes** (`col-md-6`, `d-lg-none`, `mt-sm-3`). The breakpoint is taken from the middle of the name.
- Patterns need more than a trailing `*`. In Bootstrap, `text-center` (alignment) and `text-primary` (color) share a prefix. Value patterns will be richer than attribute patterns.

## Open questions

- Where unknown classes go. Tailwind's Prettier plugin puts custom classes first; annealer may offer a `start`/`end` option.
- Whether whitespace inside a `class` value (multi-line class lists) is normalized to single spaces or preserved.
- How profiles compose (for example `extends: vue` plus a `bootstrap` class order). This also serves library-specific attribute rules.
