> For AI agents: the complete documentation index is available at /annealer/llms.txt, the full documentation bundle is available at /annealer/llms-full.txt.

# Class order

annealer can reorder the classes inside static `class` attributes. The order comes from a **class order profile**, a YAML file separate from the attribute profile, so it can be swapped for another library's order.

It is off by default. Turn it on with a class order name or file:

```bash
anyl --class-order tailwind src/**/*.vue
anyl --class-order ./my-classes.yaml index.html
```

```ts
format(source, { language: 'vue', classOrder: 'tailwind' });
```

An attribute profile can also name one with [`classOrder`](/annealer/api/profile.md#classorder). `--class-order` and the `classOrder` option override it.

## Built-in: `tailwind`

The built-in `tailwind` class order follows the order that `eslint-plugin-tailwindcss` (`classnames-order`) and `prettier-plugin-tailwindcss` apply. Both take it from the Tailwind CSS compiler.

<!-- prettier-ignore -->

```html
<!-- Input -->
<div class="p-3 border-gray-300 m-4 h-24 lg:p-4 flex border-2 lg:m-4 card"></div>

<!-- Output -->
<div class="card m-4 flex h-24 border-2 border-gray-300 p-3 lg:m-4 lg:p-4"></div>
```

annealer doesn't run Tailwind. The order is data, generated from Tailwind CSS 4.3 with its default theme by `scripts/generate-tailwind-order.mjs`. The same script writes a test corpus of 3,000 class lists sorted by Tailwind itself, and annealer's output matches all of them.

Differences from running Tailwind:

- **Your own theme and utilities aren't known.** A custom color (`bg-brand-500`) is ordered like a theme color, because a value that matches no other rule falls back to the color rule. A custom utility (`@utility`) is an unknown class.
- **Duplicates are kept.** `eslint-plugin-tailwindcss` removes them by default (`removeDuplicates`). Removing classes is a lint concern, so the built-in profile keeps them. A class order with `removeDuplicates: true` removes them.
- **Only static `class` attributes.** `:class` bindings and `clsx()`-style calls are expressions, and annealer never reads them.

## How classes are ordered

1. Classes no rule matches (`card`, `js-toggle`) go first, in their source order (`unknown: end` puts them last).
2. Classes without variants come first. Classes with variants follow, by the ranks of their variants (`variants.order`). The highest rank decides, like comparing bit masks.
3. Then by the CSS properties a class sets: the first property that differs, by its position in `propertyOrder`.
4. Then more properties first (`count`).
5. Then by name, with runs of digits compared as numbers (`p-2` before `p-10`).

This is the comparison Tailwind uses. Nothing in it is specific to Tailwind: any utility library can describe its classes by the properties they set.

## What changes in the markup

- Whitespace stays in place: classes move between the existing separators, so multi-line class lists keep their line breaks.
- Values with template syntax (`{{ }}`, `{% %}`, `<?php ?>`, `${}`) and unquoted values are left as written.
- [Disable directives](/annealer/guide/features/disable-directives.md) apply as for attributes.

## Writing a class order

```yaml
# yaml-language-server: $schema=https://github.com/logue/annealer/raw/refs/heads/main/crates/annealer/schema/class-order.schema.json
schemaVersion: 1
name: my-utilities
unknown: end
propertyOrder: [display, margin, padding, color]
values:
  size: [sm, md, lg]
rules:
  - match: [d-*]
    properties: [display]
  - match: ['m-{number}', 'm-{size}', mx-auto]
    properties: [margin]
  - match: ['p-{number}']
    properties: [padding]
  - match: [text-*]
    properties: [color]
```

- `match` patterns: `*` matches any run of characters; `{name}` matches one entry of a value set in `values`. `{number}` (a decimal number) is built in.
- The most specific pattern wins: an exact name, then the most literal characters, then the most value sets.
- A modifier after `/` (`bg-red-500/50`) is ignored when the class without it matches better.

See the [class order reference](/annealer/api/profile.md#class-order-profiles) for every key. `builtinClassOrder('tailwind')` in the JavaScript API returns the built-in YAML as a starting point.
