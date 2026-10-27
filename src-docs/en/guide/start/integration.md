---
description: Run annealer together with Rslint and rs fmt (Prettier), oxlint and oxfmt, or Biome, with configuration examples that keep the tools from undoing each other's output.
---

# Using with other tools

annealer only orders things: attributes, classes and declarations. It leaves line width and indentation to your formatter. This page shows how to combine it with the usual toolchains so that neither tool undoes the other.

The examples were checked with [Prettier](https://prettier.io/) 3.9 (through `rs fmt`), [oxfmt](https://oxc.rs/docs/guide/usage/formatter.html) 0.74, [oxlint](https://oxc.rs/docs/guide/usage/linter.html) 1.89 and [Biome](https://biomejs.dev/) 2.6.

## Run order

Run annealer first, then the formatter:

```bash
anyl --write src/App.vue   # 1. order
rs fmt src/App.vue         # 2. format (Prettier, oxfmt, Biome, …)
```

- None of these formatters reorders attributes or declarations, so the formatter keeps annealer's order.
- annealer formats stylesheets with malva, and malva lays out some values differently from Prettier, oxfmt and Biome (see [Limitations](./limitations.md#stylesheets)). When the formatter runs last, its layout wins and the result is stable: running both again changes nothing.
- `anyl --check` on its own can fail on files the formatter has just written, because of those layout differences. To check order in CI, run the pipeline and look for changes instead:

```bash
git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write
rs fmt                  # or: oxfmt / biome format --write .
git diff --exit-code    # fails when the order or the formatting was off
```

`anyl` doesn't walk directories or expand globs itself, so pass file paths, as `git ls-files` does above.

## Settings to align

annealer's built-in profiles use these values. Aligning the formatter with them keeps `<style>` blocks and stylesheets from flipping between two styles, and keeps the formatter's diff small:

| What                       | annealer (built-in profiles)          | Prettier / `rs fmt`              | oxfmt                | Biome                                            |
| -------------------------- | ------------------------------------- | -------------------------------- | -------------------- | ------------------------------------------------ |
| CSS string quotes          | single (`quotes: prefer-single`)      | `singleQuote: true`              | `singleQuote: true`  | `css.formatter.quoteStyle: "single"`             |
| Indentation in stylesheets | 2 spaces (malva default)              | default                          | default              | `formatter.indentStyle: "space"`                 |
| Line width in stylesheets  | 80 (malva default)                    | default                          | `printWidth: 80`     | default                                          |
| Void elements              | `<br />` (always)                     | default                          | default              | `html.formatter.selfCloseVoidElements: "always"` |
| Attribute sorting          | annealer                              | none                             | none                 | `useSortedAttributes` off (the default)          |
| Declaration sorting        | annealer (`declarationOrder: smacss`) | none                             | none                 | `useSortedProperties` off (the default)          |
| Tailwind class sorting     | `--class-order tailwind` (opt-in)     | no `prettier-plugin-tailwindcss` | no `sortTailwindcss` | `useTailwindSortedClasses` off                   |

To use double quotes instead, change the profile rather than the formatter: copy the built-in profile (`crates/annealer/profiles/*.yaml`) and set `stylesheet.malva.quotes: prefer-double`.

### Tailwind classes: pick one sorter

annealer's `tailwind` class order follows Tailwind's own order, as `prettier-plugin-tailwindcss` and oxfmt's `sortTailwindcss` do. The order is the same, but:

- Those formatters remove duplicate classes by default. annealer keeps them unless you pass `--remove-duplicate-classes`.
- They can also sort classes inside `clsx()`/`cn()` calls and attributes you list (`:class`). annealer only orders static `class` attributes.

Use one of them, not both. If you prefer the formatter's sorter, drop `--class-order` and, for oxfmt, set `preserveDuplicates: true` if you want annealer's behavior for duplicates.

## Rslint and `rs fmt` (Rstack)

Rslint lints JavaScript and TypeScript and has no Vue template rules, so nothing in it overlaps with annealer. `rs fmt` is built on Prettier, so annealer runs before it:

```ts title="rstack.config.ts"
import { define } from 'rstack';

define.lint(({ js, ts }) => [
  { ignores: ['dist/**'] },
  js.configs.recommended,
  ts.configs.recommendedTypeChecked
]);

define.fmt({
  // annealer writes single-quoted CSS strings.
  singleQuote: true
  // Don't add `prettier-plugin-tailwindcss` when you use `anyl --class-order tailwind`.
});

define.staged({
  // annealer first, then the formatter.
  '*.{vue,html,css,scss,less}': ['anyl --write', 'rs fmt'],
  '*.sass': 'anyl --write',
  '*.{js,jsx,ts,tsx,mjs,cjs,mts,cts}': ['rs lint --fix', 'rs fmt']
});
```

```json title="package.json"
{
  "scripts": {
    "format": "git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write && rs fmt"
  }
}
```

Keep the staged globs to the extensions annealer supports. Any other file is an error (`unsupported file type`).

## oxlint and oxfmt (Oxc)

oxlint's `vue` plugin only checks `<script>` code; it has no template rules such as `vue/attributes-order`. Nothing has to be turned off:

```json title=".oxlintrc.json"
{
  "$schema": "./node_modules/oxlint/configuration_schema.json",
  "plugins": ["typescript", "unicorn", "oxc", "vue"],
  "categories": { "correctness": "error" },
  "ignorePatterns": ["dist/**"]
}
```

oxfmt formats `.vue` and `.html` with its bundled Prettier and CSS/SCSS/Less natively. Its defaults differ from annealer in quotes (double) and line width (100):

```json title=".oxfmtrc.json"
{
  "$schema": "./node_modules/oxfmt/configuration_schema.json",
  "singleQuote": true,
  "printWidth": 80
}
```

`printWidth: 80` matches malva's line width for stylesheets. It applies to all files, so leave it out if you want 100 for JavaScript and rely on the run order instead. Leave `sortTailwindcss` off when annealer orders classes, or turn it on and drop `--class-order`:

```json title=".oxfmtrc.json (oxfmt sorts Tailwind classes)"
{
  "singleQuote": true,
  "sortTailwindcss": {
    "stylesheet": "./src/app.css",
    "preserveDuplicates": true
  }
}
```

Then run annealer before oxfmt:

```bash
git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write
oxfmt
```

## Biome

Biome's defaults conflict with annealer in four places: tabs, double-quoted CSS strings, `<br>` without a slash, and its own sorting actions. With HTML full support turned on (needed for `.vue` files), use:

```json title="biome.json"
{
  "$schema": "./node_modules/@biomejs/biome/configuration_schema.json",
  "formatter": { "indentStyle": "space", "indentWidth": 2 },
  "css": { "formatter": { "quoteStyle": "single" } },
  "html": {
    "experimentalFullSupportEnabled": true,
    "formatter": { "selfCloseVoidElements": "always" }
  },
  "assist": {
    "actions": {
      "source": {
        "useSortedAttributes": "off",
        "useSortedProperties": "off"
      }
    }
  },
  "linter": {
    "rules": { "nursery": { "useTailwindSortedClasses": "off" } }
  }
}
```

- `useSortedAttributes` sorts attributes alphabetically in HTML and Vue, and `useSortedProperties` sorts CSS declarations. Both are off by default; the config above keeps them off explicitly, because turning them on would undo annealer's order on every `biome check --write`.
- `useTailwindSortedClasses` is a nursery rule. Turn it off when annealer orders classes.
- If you keep tabs, set `stylesheet.malva.useTabs: true` in your profile instead of changing `indentStyle`.

Then run annealer before Biome:

```bash
git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write
biome check --write .
```
