# annealer — Plan

## Background

### 1. Attribute order is a readability and code-review problem, not a cosmetic one

When multiple contributors (or the same author at different times) edit the same
component, attributes get added, removed, and rewritten. Without a consistent
ordering rule, these edits produce diffs that pull in unrelated lines just because
the attribute list happened to be reshuffled. A stable, rule-based order keeps
diffs limited to what actually changed, which directly reduces the cognitive load
of code review.

### 2. HTML is a markup language whose attributes carry different semantic weight

HTML attributes are not an unordered bag of strings. Some define what an element
_is_ (component name, `is`); some control its state or behavior (`v-if`,
`v-model`, `disabled`); some supplement its meaning for a reader (`title`, `alt`);
some carry meaning to external systems (`aria-*`). Sorting attributes purely
alphabetically, or leaving them in whatever order they were typed, discards this
semantic structure. Attribute order should make an element's semantic structure
legible in the source, not just in the rendered output.

This is not a novel idea — the official Vue.js Style Guide already encodes it as
a 10-tier priority list (Definition → List Rendering → Conditionals → Render
Modifiers → Global Awareness → Unique Attributes → Two-Way Binding → Other
Attributes → Events → Content). `annealer` generalizes this same
priority-by-meaning approach beyond Vue templates, to plain HTML and to
stylesheets.

### 3. Attribute grouping should follow three semantic roles

- **State-bearing attributes** (`width`/`height`, `disabled`, `v-if`, …) — define
  an element's state or behavior.
- **Meaning-supplementing attributes** (`title`, `alt`) — help a reader understand
  the element's content.
- **Meaning-propagating attributes** (`aria-*`) — communicate meaning to external
  systems (assistive technology) through an accessibility API.

Notably, the third category mirrors a distinction the WAI-ARIA specification
itself makes official: `aria-*` attributes are formally split into _properties_
(don't change during use) and _states_ (do change during use). The proposed
grouping isn't an arbitrary preference — it surfaces a semantic structure the
underlying specs already define.

### 4. Stylesheets deserve the same treatment

CSS properties likewise fall into different roles — layout, appearance, and
state-dependent presentation. The same priority-by-meaning principle that applies
to HTML attributes should apply to stylesheet property order.

### 5. No existing tool in the Rust toolchain does this

Biome, rslint, oxlint, oxfmt, and markup_fmt were all evaluated. None of them
reorders Vue/HTML attributes by semantic meaning:

- Biome's `useSortedAttributes` (HTML variant) uses a fixed, undocumented
  category split, doesn't correctly place non-directive attributes like `id`
  against the Vue Style Guide's own stated order, and shares its underlying
  infrastructure with a known regression (duplicate/dropped attributes).
- rslint has no plans to support Vue at all (maintainer decision).
- oxlint's native Vue rules are semantic/correctness rules only; no
  attribute-order rule exists.
- oxfmt cannot reach template markup by design — the current community
  integration path (`oxlint-vue`) blanks out everything outside `<script>` to
  preserve byte offsets before handing the file to oxfmt.
- markup_fmt formats Vue/Svelte/Astro templates but has no attribute-reorder
  option in its configuration surface.

CSS property order is the one exception: `malva` (also by the markup_fmt
author) already ships a configurable `declarationOrder` option. `annealer`
delegates to it rather than reimplementing property ordering from scratch.
**The gap `annealer` fills is specifically the semantic attribute-grouping
logic — nothing else needs to be built from zero.**

### 6. Superset of `vue/attributes-order` (eslint-plugin-vue)

`annealer`'s default Vue profile starts from the same priority list
`eslint-plugin-vue`'s `vue/attributes-order` rule uses (`DEFINITION` →
`LIST_RENDERING` → `CONDITIONALS` → `RENDER_MODIFIERS` → `GLOBAL` →
`UNIQUE`/`SLOT` → `TWO_WAY_BINDING` → `OTHER_DIRECTIVES` → `OTHER_ATTR` →
`EVENTS` → `CONTENT`). Projects migrating away from ESLint keep the same
default ordering behavior out of the box.

On top of that baseline, `annealer` adds what `vue/attributes-order` doesn't
cover:

- **Same-name clustering** of static and bound forms of the same attribute
  (`class` next to `:class`) — `vue/attributes-order` doesn't do this; a
  competing proposal (eslint-plugin-vue#1728) even argues for separating
  static and dynamic attributes into different groups instead.
- **`aria-*` sub-ordering** (role → name → description → property → state,
  following the WAI-ARIA properties/states split) — `vue/attributes-order`
  treats all `aria-*` as an undifferentiated part of `OTHER_ATTR`.
- **Plain HTML support**, for projects with no Vue templates at all —
  `vue/attributes-order` only operates on Vue SFCs.
- **Stylesheet property order** (delegated to `malva`) — entirely outside
  `vue/attributes-order`'s scope.

`annealer` is not a replacement for `vue/attributes-order`; it inherits its
design and extends its reach to plain HTML, CSS, and finer-grained semantic
categories. For teams dropping ESLint in favor of rslint/oxlint, it's the
direct successor that keeps this specific piece of functionality alive.

## Goal

Given HTML-like markup as input, produce formatted markup with attributes
reordered by semantic meaning, according to a swappable, YAML-defined profile.
Given a stylesheet (CSS/SCSS/Sass/Less), delegate property ordering to `malva`.

## Scope

- Tag-attribute reordering for HTML and Vue 3 SFC `<template>` blocks, via a
  lexer-level approach (no DOM/XML tree construction — see Architecture).
- Attribute normalization the pipeline no longer has elsewhere: `v-bind:` →
  `:`, `v-on:` → `@`, camelCase → kebab-case attribute names.
- Multiline-tag layout rules equivalent to `vue/html-closing-bracket-newline`
  and `vue/multiline-html-element-content-newline` (closing bracket alone on
  its own line, content on its own line, when a tag's attributes force it to
  wrap).
- CSS/SCSS/Sass/Less property ordering, via delegation to `malva`
  (`format_text`), inside `<style>` blocks and standalone stylesheets.
- Svelte support is out of scope for the initial release (low priority; the
  HTML-first, profile-based architecture should make it a follow-on addition
  rather than a redesign).
- Less/Sass indented-syntax are deferred (minor usage).

## Non-goals

- Whitespace-insignificant reflow of text nodes (left to the surrounding
  formatter, e.g. `rs fmt`/Prettier, for line width and indentation).
- Vue 2 compatibility (`.sync`, `.native`, and other deprecated syntax are not
  targeted).
- General-purpose HTML/CSS validation.
- Handling deprecated or obsolete markup (`<xmp>`, `<center>`, `<font>`,
  `<marquee>`, presentational attributes such as `bgcolor`, …). annealer never
  warns about, removes, or replaces it. Such markup is parsed only as far as
  formatting the surrounding document correctly requires (e.g. `<xmp>` content
  is raw text, so tags inside it are left untouched). Detecting it is the job of
  a linter or validator.

## Architecture

```
annealer-core   — pure function: format(input: &str, config: &Config)
                    -> Result<String, FormatError>
                  No file I/O. WASM-compatible by construction.
                  Depends on `malva` for CSS/SCSS/Sass/Less property order.

annealer-cli    — file I/O, CLI argument parsing, calls annealer-core.
                  Binary/command name: `anyl`.
```

Single crate for the initial MVP; split into a workspace only once multiple
profiles (HTML, Vue) are implemented and a real need for separation appears
(avoiding premature abstraction, per prior experience with template
over-engineering).

### Tokenizer

Tag-attribute parsing does not build a DOM or XML tree. It locates the
start-tag range (`<tagname` … `>` or `/>`) and tokenizes the attribute list as
name/value pairs, tracking only quote style. This works identically for plain
HTML and for Vue's non-XML-compliant attribute syntax (`@click`, `#default`,
`v-bind:[dynamic]`), and avoids the whitespace-collapsing and
tag/attribute-name lowercasing that DOM- or XML-based parsing would introduce.

### Category classification

Category assignment is based on the _normalized_ attribute name — binding
prefix (`:`) and modifiers (e.g. `.camel`) stripped before classification.
Same-name clustering (static + bound forms of one attribute) falls out of this
rule automatically rather than requiring special-case logic: the first
occurrence position of a normalized name anchors its cluster, and the cluster
groups static form before bound form.

`v-bind="object"` spread syntax is treated as a cluster boundary: attributes
are never reordered across a spread, since doing so can change evaluated
order/behavior.

### Configuration

Profiles (HTML, Vue, …) are plain YAML, not code branches inside
`annealer-core`. Each profile carries a `schemaVersion` field, since the
config format is as much a public API surface as the Rust function signature
is.

## Category tables

### HTML profile (baseline)

1. Global Awareness (`id`)
2. Class (`class`, clustered with `:class`)
3. Semantic (`alt`, `href`, `src`, `title`, `type`, …)
4. ARIA (`role` → name → description → property → state)
5. Data (`data-*`, alphabetical — no semantic ordering is possible here)
6. Events (`on*`)

### Vue profile (superset of `vue/attributes-order`)

1. Definition (`is`)
2. List Rendering (`v-for`)
3. Conditionals (`v-if`/`v-else-if`/`v-else`/`v-show`/`v-cloak`)
4. Render Modifiers (`v-pre`/`v-once`)
5. Global Awareness (`id`)
6. Unique Attributes (`ref`/`key`) / Slot
7. Two-Way Binding (`v-model`)
8. Other Directives / Other Attributes (same-name clustering + ARIA
   sub-ordering applied within this tier)
9. Events (`v-on`/`@`)
10. Content (`v-html`/`v-text`)

## Open questions

- Interaction between `v-model` and a same-named plain attribute (e.g.
  `value="x" v-model="y"`) — proposed resolution: `v-model` stays in its own
  tier and is never clustered with same-named plain attributes, since it's a
  structurally different concept.
- Exact `malva` version-pinning policy (loose range, consistent with the
  library dependency philosophy already established for other projects).
- Whether `strictImportMetaEnv`-equivalent strictness is relevant to
  `annealer`'s own config validation (surfacing typos in profile YAML at
  parse time rather than silently ignoring unknown keys).
