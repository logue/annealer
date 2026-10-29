---
description: Current limitations of annealer, covering supported inputs, markup and layout, class order, stylesheets, profiles and the CLI.
---

# Limitations

annealer is still under development. This page lists what it does not do yet, and what it doesn't do by design. Each item links to the page with the details.

## Status

- **Not published yet.** The npm package and the Rust crate aren't on the registries. Build the CLI with `cargo install --path crates/annealer`.
- **The profile format can still change** before it is declared stable (`schemaVersion: 1`).

## Supported input

- HTML, Vue 3 SFC templates, and CSS, SCSS, Sass (indented syntax) and Less. Anything else is reported as an unsupported file type.
- **Not supported:** JSX/TSX, Svelte, Astro, Angular templates, Markdown/MDX, and Vue 2 syntax (`.sync`, `.native`). Svelte is planned for later.
- **Pug and Stylus are not supported.** `.pug`, `.jade`, `.styl` and `.stylus` files are reported as unsupported file types without being read. A `<template>` or `<style>` with `lang="pug"`, `lang="jade"`, `lang="stylus"` or `lang="styl"` fails the whole file (`unsupported language`) instead of being skipped, so annealer never scans Pug as markup or leaves the other blocks half-formatted. Exclude such files from the paths you pass to annealer.
- In `.vue` files, `<script>` and custom blocks (`<i18n>`) are left alone.

## Markup

- **No DOM.** annealer reads start tags with a lexer. It doesn't validate HTML or fix broken markup. Deprecated markup (`<center>`, `bgcolor`) is parsed but never reported or changed.
- **No reflow.** annealer adds or removes line breaks only where a layout rule needs them. It never re-indents lines or wraps text to a line width; that is the formatter's job ([Using with other tools](./integration.md)).
- **Spreads are boundaries.** Attributes don't move across `v-bind="object"`, except compile-time directives (`spreadSafe` groups). A tag with a spread can therefore stay partly unordered, and `vue/attributes-order` reports it ([Attribute order](../concepts/attribute-order.md), [eslint-plugin-vue](./integration.md#eslint-plugin-vue)).
- **Foreign content is left as written.** Inline SVG (`<svg>`), MathML (`<math>`) and namespaced XML elements such as RDF (`<rdf:RDF>`), with everything inside them, are not touched: no attribute order, class order, self-closing, `contentNewline` or `<style>` formatting. In plain HTML, `<svg>` and `<math>` match case-insensitively; in Vue templates, `<Svg>` is a component.
- **Same-name clusters keep source order**, except for `class`, where the static form goes before the bound one. `style` and other duplicates stay as written because their order can change behavior.

## Layout

- **Void elements are self-closed by default** (`<br />`, `<hr />`, `<wbr />`). This differs from the default of `vue/html-self-closing` (`html.void: never`); set `html.void: always` there, or pass `--no-void-self-closing` (`void: never` in your profile).
- **Self-closing empty elements and components (`<div />`, `<MyComp />`) and `contentNewline` are off by default.** Turn them on with `--self-closing` / `--content-newline` ([Layout](../features/layout.md)). `<div />` is applied in Vue templates only, because it is an unclosed start tag in plain HTML; there, `--expand-non-void` rewrites it to `<div></div>` instead.
- `contentNewline` adds line breaks but doesn't re-indent the content. Elements that are closed implicitly (`<li>` without `</li>`) are left alone.

## Class order

- **Only static `class` attributes.** Bound `:class`, `clsx()`/`cn()` calls and values with template syntax are never read.
- **Only Tailwind is built in**, generated from Tailwind CSS 4.3 with the default theme. Your own theme values and `@utility` classes aren't known: a custom color is ordered like a theme color, and unknown classes go first ([Class order](../features/class-order.md)).
- **Duplicates are kept by default**, where `prettier-plugin-tailwindcss` and oxfmt remove them. Turn on `--remove-duplicate-classes` to remove them.
- No Bootstrap class order yet.

## Stylesheets

- **Stylesheets are reformatted with malva**, not only reordered. malva's layout differs from Prettier, oxfmt and Biome in places. For example, it keeps comma-separated values (`transition`, `font-family`) on one line where they break them onto several lines. No malva option removes this difference, so `anyl --check` can fail on files a formatter has just written. Run annealer before the formatter ([Run order](./integration.md#run-order)).
- **CSS strings use single quotes** by default (`quotes: prefer-single`). With a formatter that writes double quotes, align one of them ([Settings to align](./integration.md#settings-to-align)).
- Vendor-prefixed declarations move only when `declarationOrder` is set. A run of declarations with a comment between them is left alone ([Stylesheets](../concepts/stylesheets.md)).
- Static `style` attributes are reordered only when `stylesheet.styleAttribute` is on, and not when they contain template syntax, comments or values that don't parse as plain declarations. Bound `:style` is never touched.

## Profiles and the CLI

- **No profile composition (`extends`).** To change one setting of a built-in profile, copy the whole profile (`crates/annealer/profiles/*.yaml` or `builtinProfile()`) and edit the copy.
- **No configuration file.** Options such as `--profile`, `--class-order`, `--self-closing` and `--print-width` are passed on every run (put them in a `package.json` script).
- **`anyl` takes file paths only.** It doesn't walk directories, expand globs or read ignore files. Pass the paths from your shell or from `git ls-files`.
- No editor extension, language server or Prettier plugin. Run annealer from the CLI, a Git hook or the JavaScript API.
