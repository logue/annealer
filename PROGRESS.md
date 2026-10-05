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
- [x] `stylesheet.vendorPrefix: start` (both profiles): vendor-prefixed
      declarations are treated as separate from standard properties (dialects,
      often never standardized). They go before the other declarations of the
      same run, in plain alphabetical order, with no grouping by property.
  - Not `end`: prefixed properties are often aliases of the standard one and
    the later declaration wins. Putting them last could let a dialect override
    the standard value (e.g. `-webkit-border-radius` with two values). The
    `start` position keeps "standard last", like Autoprefixer's output.
  - malva has no such option (it sorts by the unprefixed name), so annealer
    re-parses malva's output with raffia and moves only those declarations.
    Runs match malva's own sort units. A run is left alone if a comment sits
    between its declarations. Applies only when `declarationOrder` is set.
- [x] Schema `$id`: `https://github.com/logue/annealer/raw/refs/heads/main/crates/annealer/schema/profile.schema.json`

## Disable directives

- [x] `annealer-disable` / `annealer-enable` / `annealer-disable-next-line: reason`
      in `<!-- -->` (markup) and `/* */` (stylesheets); see PLAN.md
- [x] Unknown `annealer-*` directives are errors
- [x] CSS: covered statements are kept verbatim and in place via internal
      markers passed to malva (`malva-ignore` can't take a reason and doesn't
      stop sorting)

## Phase 2

- [x] `vue/multiline-html-element-content-newline` equivalent: `layout.contentNewline`
  - An element is multiline when its end tag starts on a later line than its
    start tag (the ESLint rule's definition). Its content then starts and ends
    with exactly one line break (`allowEmptyLines: true` keeps blank lines).
    Empty elements are left alone.
  - `ignore` defaults to the rule's defaults: `pre`, `textarea` and inline
    elements. Everything inside an ignored element is left alone too.
  - Needs the end tag, so the scanner now keeps a stack of open elements.
    Unclosed HTML elements (`<li>` without `</li>`) are closed implicitly by
    an ancestor's end tag and are not touched.
  - Only line breaks are added; indentation is not re-flowed (non-goal). The
    new line takes the attribute indentation of a multiline start tag, else
    the first deeper content line, else the tag's indentation plus two spaces
    (a tab if the tag is indented with tabs).
  - Moving content to a new line changes the indentation of the line that a
    nested tag's closing bracket aligns to, so the document is formatted again
    until stable (at most 4 passes; normally 2).
  - With `closingBracketNewline`, a start tag whose attributes fit on one line
    keeps `>` on that line (`singleline: never` of
    `vue/html-closing-bracket-newline`), e.g. Prettier's `title="t"\n  >text`.
    Whitespace inside a tag is never significant.
- [x] Less and Sass (indented syntax) in `<style lang="less|sass">` and `.less`/`.sass` files
  - Disable directives work in both. Sass has no `;`, so the internal markers
    go on lines of their own there (malva only honors its ignore comment on
    its own line in Sass).
- [x] WASM + npm wrapper built with the existing Rslib setup, and a demo page with Rsbuild
  - `wasm` Cargo feature (`wasm-bindgen`), built by `scripts/build-wasm.mjs`
    (`wasm-pack --target web`, `wasm-release` profile: LTO, `opt-level = "s"`)
    into `src/wasm/` (generated, git-ignored). `pnpm run build`/`dev`/`test`
    run it first (about 2 s when nothing changed).
  - The binary (1.3 MB, 0.65 MB gzipped in the demo bundle) is embedded as
    base64, so `format()` is synchronous and works in Node.js, browsers and
    the UMD build with no asset loading to configure. `init()` instantiates it
    asynchronously ahead of time (optional).
  - JS API: `format(input, { language, profile })`, `init()`,
    `languageFromPath()`, `builtinProfile()`, plus `Language`,
    `FormatOptions`, `Meta`. `profile` is a built-in name or YAML text.
  - Demo: playground with language/profile selection and a custom YAML editor.
- [x] Fixture-based tests: `tests/fixtures/<name>.<ext>` → `<name>.expected.<ext>`
      with an idempotence check; `ANNEALER_UPDATE_FIXTURES=1` rewrites the
      expected files.
  - [ ] The current fixtures are hand-written in the style of typical projects
        (Vue SFC, HTML page, SCSS, Less, Sass). Add samples from real projects.
- [x] Opt-in value ordering for static `style` attributes via malva
      (`stylesheet.styleAttribute`, off in both built-in profiles)
  - Declarations stay on one line; CSS strings use the quote opposite to the
    attribute's; a missing trailing `;` stays missing.
  - Left as written: bound `:style`, unquoted values, values with template
    syntax, comments, `{}`/`<`/`&`, or that don't parse as plain declarations.
- [ ] Upgrade raffia to 0.13 when malva does (0.16 still uses raffia 0.12; see the
      comment in `crates/annealer/Cargo.toml`). Blocked on malva.
- [ ] Svelte profile (follow-on)
- [ ] Value ordering for static `class` attributes (profile-defined order), see
      PLAN.md "Value ordering"
  - [ ] Tailwind: built-in approximate order
  - [ ] Bootstrap: draft profile + proposal to the Bootstrap project (v6), so
        Bootstrap owns the order; needs a stable, documented profile format first
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

7. **`contentNewline` is on in the HTML profile too**, like the other layout
   rules. Block-level content whitespace at the start and end of an element
   is not rendered, and the ignore list covers the elements where it can be.
8. **The npm package embeds the WebAssembly binary** (base64) instead of
   shipping a separate `.wasm` file. Trade-off: about 33% more bytes before
   compression, in exchange for a synchronous API and no bundler/loader setup.

## Open questions resolved

- `v-model` vs a plain `value`: never clustered. The key `v-model` differs from `value`.
- Profile strictness: unknown keys are rejected at parse time (`deny_unknown_fields`).
- `malva` pinning: loose range `0.16`.
