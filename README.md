# Annealer

[![npm version](https://img.shields.io/npm/v/annealer.svg)](https://www.npmjs.com/package/annealer)
[![NPM Downloads](https://img.shields.io/npm/dm/annealer.svg?style=flat)](https://www.npmjs.com/package/annealer)
[![Crates.io Downloads](https://img.shields.io/crates/dv/annealer?style=flat)](https://crates.io/crates/annealer)
[![jsdelivr CDN](https://data.jsdelivr.com/v1/package/npm/annealer/badge)](https://www.jsdelivr.com/package/npm/annealer)
[![Open in unpkg](https://img.shields.io/badge/Open%20in-unpkg-blue)](https://uiwjs.github.io/npm-unpkg/#/pkg/annealer/file/README.md)
[![Open in Gitpod](https://shields.io/badge/Open%20in-Gitpod-green?logo=Gitpod)](https://gitpod.io/#https://github.com/logue/annealer)
[![X Follow](https://img.shields.io/twitter/follow/logue256?style=plastic)](https://twitter.com/logue256)
[![GitHub Sponsors](https://img.shields.io/github/sponsors/logue?label=Sponsor&logo=github&color=ea4aaa)](https://github.com/sponsors/logue)

Reorders HTML and Vue attributes, Tailwind class names and stylesheet
declarations **by meaning**, using swappable YAML profiles.

> **Alpha (0.1.0-alpha.1).** Until 1.0, a release can change the output for the
> same input. Pin the exact version and review the diff when you upgrade.

```html
<!-- Input -->
<a title="Home" href="/" id="home">Home</a>

<!-- Output (html profile) -->
<a id="home" title="Home" href="/">Home</a>
```

Try it in the [Playground](https://logue.dev/annealer/playground), or read the
[documentation](https://logue.dev/annealer/).

## Why

Attribute order is a code-review problem, not a cosmetic one. Without a rule,
edits reshuffle attribute lists and diffs fill up with lines that did not really
change. annealer orders attributes by the role they play: identity first, then
what an element is and does, supplementary meaning, ARIA, data, and events.

None of the tools I evaluated (Biome, rslint, oxlint, oxfmt, markup_fmt)
reorders Vue/HTML attributes by meaning, so annealer fills that gap. See
[Introduction](https://logue.dev/annealer/guide/start/introduction) for the
reasoning and how it relates to `vue/attributes-order`.

## Features

- **Semantic attribute order** for HTML and Vue 3 templates.
- **A superset of `vue/attributes-order`**: same tiers, plus same-name
  clustering (`class` next to `:class`) and ARIA sub-ordering.
- **Stylesheets** (CSS, SCSS, Sass, Less) through [malva](https://github.com/g-plane/malva).
- **Tailwind class order** (opt-in), following Tailwind's own order.
- **YAML profiles**, validated by a JSON Schema; unknown keys are rejected.
- **Disable directives** (`annealer-disable`, `annealer-enable`,
  `annealer-disable-next-line`) keep selected code exactly as written.
- **One Rust core** with a CLI (`anyl`), a Rust crate, and an npm package with a
  synchronous WebAssembly API.

## Install

annealer ships in two forms. Pick the one that matches how you want to use it.

### CLI (`anyl`) — Rust crate

The command-line tool is distributed as a Rust crate only. You need a
[Rust toolchain](https://rustup.rs/); `cargo install` builds it from source.

```sh
cargo install annealer --version 0.1.0-alpha.1
```

The npm package does **not** include the `anyl` command, so
`pnpm add -D annealer` will not give you a CLI.

### JavaScript API — npm package (WebAssembly)

For build tools, scripts and editors. This is a library only; there is no
binary.

```sh
pnpm add -D annealer@0.1.0-alpha.1
```

Pre-release versions are not picked up by ranges such as `^0.1.0`, so name the
version in both cases.

## Usage

### CLI

```sh
anyl src/App.vue                 # print the result
anyl --write src/**/*.vue        # format in place
anyl --check index.html          # exit 1 if a file would change
anyl --profile my-profile.yaml page.html
```

### JavaScript

```js
import { format } from 'annealer';

format('<a title="Home" href="/" id="home">Home</a>');
// => '<a id="home" title="Home" href="/">Home</a>'

format(source, { language: 'vue' });
```

Run annealer **before** your formatter (Prettier, oxfmt, Biome, `rs fmt`); none
of them reorders attributes, so they keep annealer's order. See
[Using with other tools](https://logue.dev/annealer/guide/start/integration).

## Limitations

annealer reads start tags with a lexer; it does not build a DOM, validate HTML
or reflow text. Inline SVG, MathML and RDF are left as written. Pug and Stylus
are not supported. See
[Limitations](https://logue.dev/annealer/guide/start/limitations) for the full
list.

## Feedback

This is an early release and the profile format can still change. Issues and
discussion are welcome on GitHub.

## License

©2026 by Logue. Licensed under the [MIT License](LICENSE).
