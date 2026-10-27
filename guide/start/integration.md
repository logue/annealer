> For AI agents: the complete documentation index is available at /annealer/llms.txt, the full documentation bundle is available at /annealer/llms-full.txt.

# Using with other tools

annealer only orders things: attributes, classes and declarations. It leaves line width and indentation to your formatter. This page shows how to combine it with the usual toolchains so that neither tool undoes the other.

The examples were checked with:

- [Prettier](https://prettier.io/) 3.9 (through `rs fmt`)
- [oxfmt](https://oxc.rs/docs/guide/usage/formatter.html) 0.74
- [oxlint](https://oxc.rs/docs/guide/usage/linter.html) 1.89
- [eslint-plugin-vue](https://eslint.vuejs.org/) 10.11, with [eslint-config-prettier](https://npm.im/eslint-config-prettier) 10.1
- [Biome](https://biomejs.dev/) 2.6

## Run order

Run annealer first, then the formatter:

```bash
anyl --write src/App.vue   # 1. order
rs fmt src/App.vue         # 2. format (Prettier, oxfmt, Biome, …)
```

- None of these formatters reorders attributes or declarations, so the formatter keeps annealer's order.
- annealer formats stylesheets with malva, and malva lays out some values differently from Prettier, oxfmt and Biome (see [Limitations](/annealer/guide/start/limitations.md#stylesheets)). When the formatter runs last, its layout wins and the result is stable: running both again changes nothing.
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
| Line width in stylesheets  | 80; `--print-width` to change         | default (80)                     | `--print-width 100`  | default (80)                                     |
| Void elements              | `<br />` (always)                     | default                          | default              | `html.formatter.selfCloseVoidElements: "always"` |
| Attribute sorting          | annealer                              | none                             | none                 | `useSortedAttributes` off (required)             |
| Declaration sorting        | annealer (`declarationOrder: smacss`) | none                             | none                 | `useSortedProperties` off (required)             |
| Tailwind class sorting     | `--class-order tailwind` (opt-in)     | no `prettier-plugin-tailwindcss` | no `sortTailwindcss` | `useTailwindSortedClasses` off                   |

To use double quotes instead, change the profile rather than the formatter: copy the built-in profile (`crates/annealer/profiles/*.yaml`) and set `stylesheet.malva.quotes: prefer-double`.

### No alphabetical sorting

annealer exists to put attributes and declarations in an order that carries meaning: identity first, then what an element is and does, ARIA, data, and events. Alphabetical order throws that away, so any tool that sorts attributes or declarations alphabetically works against annealer, and the two keep rewriting each other's output. Turn these off:

| Tool              | Setting                                                                                   |
| ----------------- | ----------------------------------------------------------------------------------------- |
| eslint-plugin-vue | `vue/attributes-order`: keep `alphabetical: false` (the default)                          |
| Biome             | `assist.actions.source.useSortedAttributes` and `useSortedProperties`: `off`              |
| Stylelint         | don't use `stylelint-order` `properties-alphabetical-order` on files that annealer orders |

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

## eslint-plugin-vue

`vue/attributes-order` (in `flat/recommended`) checks the same tiers that annealer's `vue` profile uses, so annealer's output passes it. Keep `alphabetical` at its default, `false` (see [No alphabetical sorting](#no-alphabetical-sorting)). With Prettier, add `eslint-config-prettier` last: it turns off the layout rules (`vue/html-self-closing`, `vue/max-attributes-per-line`, …), and Prettier decides the layout.

```js title="eslint.config.js"
import vue from 'eslint-plugin-vue';
import prettier from 'eslint-config-prettier';

export default [
  ...vue.configs['flat/recommended'],
  prettier,
  {
    rules: {
      // annealer orders the "other attributes" tier by meaning.
      'vue/attributes-order': ['warn', { alphabetical: false }]
    }
  }
];
```

- **Spreads:** annealer never moves an attribute across `v-bind="object"`, because in Vue 3 the later one wins. `vue/attributes-order` still checks the order across the spread, and its `--fix` moves attributes across it (`title="t" v-bind="obj"` becomes `v-bind="obj" title="t"`, so `title` now overrides `obj.title`). On such tags, order the attributes by hand or add an `eslint-disable` comment.
- **Not rewritten by annealer:** `vue/v-on-event-hyphenation` (`@myEvent` → `@my-event`) and `vue/v-slot-style` (`v-slot:item` → `#item`). `eslint --fix` rewrites them, and annealer leaves its result alone.

Run annealer, then Prettier, then `eslint --fix`. `anyl --check` passes on the result.

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

oxfmt formats `.vue` and `.html` with its bundled Prettier and CSS/SCSS/Less natively. Its defaults differ from annealer in quotes (double) and line width (100 instead of 80). Set the quotes in oxfmt:

```json title=".oxfmtrc.json"
{
  "$schema": "./node_modules/oxfmt/configuration_schema.json",
  "singleQuote": true
}
```

For the line width, change annealer rather than oxfmt: pass `--print-width 100` (JavaScript: `printWidth: 100`), so that stylesheets and `<style>` blocks wrap where oxfmt wraps them. If you set `printWidth` in `.oxfmtrc.json`, pass the same value to `--print-width`. Leave `sortTailwindcss` off when annealer orders classes, or turn it on and drop `--class-order`:

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
git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write --print-width 100
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

- **Turn off `useSortedAttributes` and `useSortedProperties`.** `useSortedAttributes` sorts HTML and Vue attributes alphabetically, and `useSortedProperties` sorts CSS declarations by its own order. Both conflict with annealer's ordering by meaning and undo it on every `biome check --write` (see [No alphabetical sorting](#no-alphabetical-sorting)). They are off by default; set them to `off` explicitly as above, so that a shared config can't turn them on.
- `useTailwindSortedClasses` is a nursery rule. Turn it off when annealer orders classes.
- If you keep tabs, set `stylesheet.malva.useTabs: true` in your profile instead of changing `indentStyle`.

Then run annealer before Biome:

```bash
git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write
biome check --write .
```
