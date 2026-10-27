---
description: What annealer is, why attribute order matters, how it relates to vue/attributes-order, and what is in and out of scope.
---

# Introduction

annealer is a formatter that reorders HTML and Vue template attributes by their semantic meaning. It also reorders declarations in stylesheets (CSS, SCSS, Sass, Less). The order is defined by a swappable YAML profile, not hard-coded.

```html
<!-- Input -->
<a title="Home" href="/" id="home">Home</a>

<!-- Output (html profile) -->
<a id="home" title="Home" href="/">Home</a>
```

:::warning Work in progress
annealer is under active development. The npm package is not published yet, and the profile format can still change before it is declared stable.
:::

## Why attribute order matters

### Order is a code-review problem, not a cosmetic one

When several contributors edit the same component over time, attributes are added, removed and rewritten. Without a fixed rule, the attribute list gets reshuffled, and diffs pull in lines that didn't really change. A stable, rule-based order keeps a diff limited to what actually changed, which lowers the cost of review.

### Attributes carry different semantic weight

HTML attributes are not an unordered set of strings:

- Some define what an element **is** (`is`, a component name).
- Some control its **state or behavior** (`v-if`, `v-model`, `disabled`, `width`/`height`).
- Some **supplement its meaning** for a human reader (`title`, `alt`).
- Some **propagate meaning** to external systems such as assistive technology (`aria-*`).

Sorting alphabetically, or keeping the typed order, hides this structure. annealer orders attributes so that an element's semantic structure is visible in the source.

The grouping is not an arbitrary preference. The WAI-ARIA specification itself splits `aria-*` attributes into _properties_ (which don't change during use) and _states_ (which do). annealer uses this split.

### Stylesheets deserve the same treatment

CSS properties also have roles: layout, appearance, state-dependent presentation. The same "order by meaning" principle applies to declaration order.

## Relation to `vue/attributes-order`

The Vue.js Style Guide already defines a priority list for attributes, and eslint-plugin-vue implements it as `vue/attributes-order`. annealer's Vue profile starts from the same default order, so projects that move away from ESLint keep the same result.

On top of that baseline, annealer adds:

- **Same-name clustering**: the static and bound forms of one attribute stay together (`class` next to `:class`).
- **ARIA sub-ordering**: role → name → description → property → state, following the WAI-ARIA properties/states split.
- **Plain HTML support**, for projects without Vue templates.
- **Stylesheet declaration order**, delegated to [malva](https://github.com/g-plane/malva).

annealer is not a rival to `vue/attributes-order`. It inherits its design and extends it. For teams that replace ESLint with rslint or oxlint, it keeps this one piece of functionality alive.

## Why a new tool

Biome, rslint, oxlint, oxfmt and markup_fmt were evaluated. None of them reorders Vue/HTML attributes by meaning:

| Tool                                | Status                                                                                                                                                                                                   |
| ----------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Biome (`useSortedAttributes`, HTML) | Fixed, undocumented category split; doesn't place non-directive attributes such as `id` the way the Vue Style Guide does; shares infrastructure with a known regression (duplicated/dropped attributes). |
| rslint                              | No plans to support Vue (maintainer decision).                                                                                                                                                           |
| oxlint                              | Vue rules are about correctness only; no attribute-order rule.                                                                                                                                           |
| oxfmt                               | Can't reach template markup by design.                                                                                                                                                                   |
| markup_fmt                          | Formats Vue/Svelte/Astro templates, but has no option to reorder attributes.                                                                                                                             |

CSS property order is the exception: malva already has a configurable `declarationOrder`, so annealer delegates to it. **What annealer adds is the semantic attribute grouping. Nothing else is built from zero.**

## Scope

- Attribute reordering in HTML and in Vue 3 SFC `<template>` blocks.
- Attribute normalization: `v-bind:` → `:`, `v-on:` → `@`, camelCase → kebab-case on component props.
- Layout rules for multiline tags, equivalent to `vue/html-closing-bracket-newline`, `vue/max-attributes-per-line` (multiline), `vue/html-self-closing` (void elements always, the rest opt-in) and, opt-in, `vue/multiline-html-element-content-newline`.
- Declaration order for CSS/SCSS/Sass/Less, in `<style>` blocks and in standalone files, via malva.
- Opt-in declaration order inside static `style` attributes.
- [Disable directives](../features/disable-directives.md) in markup and stylesheet comments.
- Planned: [value ordering](../design/value-ordering.md) for static `class` attributes, and [library profiles](../design/library-profiles.md).
- Svelte is not part of the first release. The profile-based design should allow adding it later without a redesign.

## Non-goals

- Reflowing text or indentation. Line width and indentation are left to the surrounding formatter (`rs fmt`, Prettier, …).
- Vue 2 syntax (`.sync`, `.native`, …).
- General HTML/CSS validation.
- Handling deprecated or obsolete markup (`<xmp>`, `<center>`, `<font>`, `<marquee>`, `bgcolor`, …). annealer never warns about it, removes it or replaces it. It parses such markup only as far as the surrounding document needs (for example, `<xmp>` content is raw text, so tags inside it are left alone). Detecting it is a job for a linter or validator.

## Next steps

- [Getting started](./getting-started.md): run annealer from the CLI or from JavaScript.
- [Using with other tools](./integration.md): configuration for Rslint/`rs fmt`, oxlint/oxfmt and Biome.
- [Limitations](./limitations.md): what annealer does not do yet.
- [Attribute order](../concepts/attribute-order.md): how attributes are classified and sorted.
