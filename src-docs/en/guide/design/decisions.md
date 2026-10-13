---
description: Architecture of annealer and the design decisions made during implementation, with their reasons.
---

# Design decisions

## Architecture

```text
annealer (core)  pure function: format(input: &str, config: &Config)
                   -> Result<String, FormatError>
                 No file I/O. Builds for wasm32-unknown-unknown.
                 Uses malva for CSS/SCSS/Sass/Less declaration order.

anyl (CLI)       File I/O and argument parsing; calls the core.

annealer (npm)   WebAssembly build of the core with a TypeScript wrapper.
```

- **One crate.** The core and the CLI live in one crate (`crates/annealer`) for now. It will be split into a workspace only when a real need appears, to avoid premature abstraction. The CLI's `clap` dependency is behind the default `cli` feature, so the library builds for WebAssembly with `--no-default-features`.
- **Profiles are YAML, not code.** HTML, Vue and future profiles are data. Each profile has `schemaVersion`, because the format is as much a public API as the Rust function signature.
- **Strict validation.** Unknown keys are rejected when the profile is parsed, so typos surface immediately. A JSON Schema (draft-07) is provided for editor support. Two rules that draft-07 can't express (exactly one fallback group, unique group names) are checked only by the loader.
- **malva is pinned loosely** (`0.16`), in line with the library dependency policy used in other projects.
- **The npm package embeds the WebAssembly binary** as base64 instead of shipping a separate `.wasm` file. The cost is about 33% more bytes before compression. In return, `format()` is synchronous and needs no bundler or loader setup.

## Decisions made during implementation

1. **HTML profile order.** The original table had no place for unmatched attributes or for `alt`/`title`. The agreed order is id → class → supplementary (`alt`, `title`, `placeholder`) → semantic (`type`, `href`, `src`, `srcset`) → key-value (`name`, `value`, `content`) → dimensions (`width`, `height`) → other (state: `disabled`, `checked`, …) → ARIA → data → events. The Vue profile uses the same order inside its other-attributes tier.
2. **Static before bound applies to `class` only.** For other names, the cluster keeps the source order, because Vue merges `style` with the later declaration winning and keeps only one value for other duplicates.
3. **`spreadSafe` groups.** Compile-time directives never merge with a `v-bind` spread object, so they may move across spreads. Everything else stays inside its segment.
4. **One attribute per line in multiline tags.** Otherwise reordering could put two attributes on one line. This matches the multiline default of `vue/max-attributes-per-line` and can be turned off with `layout.oneAttributePerLine`.
5. **Vue tier 8 is split** into other directives, then other attributes, as in `vue/attributes-order` (`OTHER_DIRECTIVES` → `OTHER_ATTR`).
6. **`void: always` differs from `vue/html-self-closing`'s default** (`html.void: never`). Projects that keep that ESLint rule need to align one side.
7. **`contentNewline` is on in the HTML profile too.** Whitespace at the start and end of block-level content isn't rendered, and the `ignore` list covers the elements where it could be.
8. **Vendor-prefixed declarations go first** (`vendorPrefix: start`), so the standard property is declared last and wins. See [Stylesheets](../concepts/stylesheets.md#vendor-prefixes-vendorprefix-start).

## Resolved questions

- **`v-model` and a plain `value`**: never clustered. The key `v-model` differs from `value`.
- **Profile strictness**: unknown keys are rejected at parse time.
- **malva version**: loose range `0.16`.

## Open items

- Upgrade raffia to 0.13 once malva does (malva 0.16 still uses raffia 0.12).
- Svelte profile.
- [Value ordering](./value-ordering.md) for static `class` attributes (Tailwind approximation, Bootstrap proposal).
- [Library profiles](./library-profiles.md), profile composition (`extends`) and per-tag or per-component overrides.
- Fixtures taken from real projects, in addition to the hand-written ones.
