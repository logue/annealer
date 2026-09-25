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
- [ ] Publish the schema at a stable URL and set `$id` (needs the final repo URL)

## Phase 2: Next

- [ ] `vue/multiline-html-element-content-newline` equivalent (content on its own line)
- [ ] Less and Sass (indented syntax) in `<style>` and standalone files
- [ ] WASM + npm wrapper built with the existing Rslib setup, and a demo page with Rsbuild
- [ ] Fixture-based tests from real-world Vue/HTML projects
- [ ] Svelte profile (follow-on)

## Decisions made during implementation (please confirm)

1. **Fallback group placement (HTML profile).** Unmatched attributes
   (`width`, `disabled`, `name`, …) had no tier in the plan's table. They go
   after `semantic` and before a new `supplementary` group (`alt`, `title`,
   `placeholder`). This follows the Background's order: state-bearing →
   meaning-supplementing → meaning-propagating. So the order is
   id → class → semantic (`type`, `href`, `src`, `srcset`) → other →
   supplementary → ARIA → data → events.
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

## Open questions resolved

- `v-model` vs a plain `value`: never clustered. The key `v-model` differs from `value`.
- Profile strictness: unknown keys are rejected at parse time (`deny_unknown_fields`).
- `malva` pinning: loose range `0.16`.
