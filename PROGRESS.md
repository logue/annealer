# PROGRESS.md

## Phase 1: Rust MVP (`crates/annealer`)

- [x] Cargo workspace at repo root, single crate `crates/annealer` (lib + `anyl` bin)
  - The workspace wrapper only keeps Rust out of the TS `src/`; it is still one crate.
  - `clap` is behind the default `cli` feature; the library builds for
    `wasm32-unknown-unknown` with `--no-default-features`.
- [x] YAML profiles (`schemaVersion: 1`, `deny_unknown_fields`) with built-in `html` and `vue`
- [x] Lexer-level start-tag tokenizer (no DOM); raw-text elements, comments,
      `{{ }}` interpolations, SFC `<script>`/custom blocks/non-HTML `<template lang>` skipped
- [x] Classification by normalized key, most-specific pattern wins (exact > longest `prefix*`)
- [x] Same-name clustering, `v-bind="obj"` spread boundaries
- [x] Normalization: `v-bind:` → `:`, `v-on:` → `@`, camelCase → kebab-case on component props
- [x] Layout: closing bracket of multiline tags on its own line; one attribute per line in multiline tags
- [x] `<style>` / `.css` / `.scss` property order via `malva` (`declarationOrder: smacss`)
- [x] CLI `anyl`: stdout / `--write` / `--check`, `--profile <name|path>`, stdin with `--language`
- [x] Tests: 9 unit + 26 integration (every case checks idempotence) + doctest

## Phase 1.5: Profile schema

- [x] JSON Schema (draft-07) at `crates/annealer/schema/profile.schema.json`
  - Built-in profiles reference it with a `yaml-language-server` modeline;
    `redhat.vscode-yaml` added to the recommended VS Code extensions.
- [x] `stylesheet` split into annealer-owned keys (`declarationOrder`,
      `declarationOrderGroupBy`) and a `malva:` passthrough. Unknown malva
      keys are rejected, so typos no longer disappear silently.
- [x] Tests keep schema and loader in agreement (valid and invalid cases,
      malva key list). Two rules draft-07 can't express (exactly one fallback
      group, unique group names) are enforced only by the loader.
- [x] `layout.selfClosing` (`void` / `normal` / `component`: `always` | `never` | `preserve`)
  - Both profiles: `void: always` (`<br />`). Vue profile also sets `normal` and
    `component` to `always` (`<div />`, `<MyComp />`).
  - `normal`/`component` apply in Vue templates only (`<div />` is an unclosed
    start tag in plain HTML). Whitespace-only content counts as empty, except
    in `pre`/`textarea`. SFC top-level blocks are never touched.
- [ ] Publish the schema at a stable URL and set `$id` (needs the final repo URL)

## Phase 2: Next

- [ ] `vue/multiline-html-element-content-newline` equivalent (content on its own line)
- [ ] Less and Sass (indented syntax) in `<style>` and standalone files
- [ ] WASM + npm wrapper built with the existing Rslib setup, and a demo page with Rsbuild
- [ ] Fixture-based tests from real-world Vue/HTML projects
- [ ] Svelte profile (follow-on)
- [ ] Library-specific rule profiles, built on the same mechanisms (groups,
      `layout.selfClosing`, normalize). Likely needs profile composition
      (`extends`) and per-tag/per-component overrides.

## Decisions made during implementation (please confirm)

1. **HTML profile order (agreed).** The plan's table had no place for
   unmatched attributes or for `alt`/`title`. The order is
   id → class → supplementary (`alt`, `title`, `placeholder`) → semantic
   (`type`, `href`, `src`, `srcset`) → key-value (`name`, `value`, `content`)
   → dimensions (`width`, `height`) → other (state: `disabled`, `checked`, …)
   → ARIA → data → events.
   - Supplementary attributes describe the element, so they follow `id`/`class`
     (kept together as the common idiom).
   - ARIA stays near the end: it carries extra metadata that `alt`/`title`
     can't express, rather than the primary description.
   - The Vue profile uses the same order inside its other-attributes tier.
2. **Cluster order: static before bound applies to `class` only.** For other
   names, the source order is kept inside the cluster. Vue merges `style`
   with the later declaration winning, and for other duplicate names it keeps
   only one of them, so reordering those could change behavior.
3. **`spreadSafe` groups.** Compile-time directives (`is`, `v-for`, `v-if`/…,
   `v-pre`/`v-once`, custom directives) never merge with a `v-bind` spread
   object, so they are moved across spreads. Everything else stays inside
   its segment.
4. **One attribute per line in multiline tags.** If attributes kept their
   original line positions, reordering could put two attributes on one line.
   The rule matches the `vue/max-attributes-per-line` multiline default and can
   be switched off per profile (`layout.oneAttributePerLine`).
5. **Vue tier 8 is split** into `other-directives` (before) and the other
   attribute groups, matching `vue/attributes-order`'s default
   `OTHER_DIRECTIVES` → `OTHER_ATTR`.

6. **`void: always` differs from `vue/html-self-closing`'s default**
   (`html.void: never`). Projects that keep that ESLint rule need
   `html.void: always` there, or `void: never` in the annealer profile.

## Open questions resolved

- `v-model` vs a plain `value`: never clustered. The key `v-model` differs from `value`.
- Profile strictness: unknown keys are rejected at parse time (`deny_unknown_fields`).
- `malva` pinning: loose range `0.16`.
