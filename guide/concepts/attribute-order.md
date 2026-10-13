> For AI agents: the complete documentation index is available at /annealer/llms.txt, the full documentation bundle is available at /annealer/llms-full.txt.

# Attribute order

## Tokenizer: no DOM

annealer does not build a DOM or XML tree. It finds each start tag (`<tagname` … `>` or `/>`) and splits the attribute list into name/value pairs, keeping only the quote style.

This works the same way for plain HTML and for Vue's non-XML syntax (`@click`, `#default`, `v-bind:[dynamic]`). It also avoids side effects of DOM- or XML-based parsing, such as collapsing whitespace or lowercasing tag and attribute names.

Raw-text elements, comments, `{{ }}` interpolations, SFC `<script>` blocks, custom blocks and `<template lang="…">` with a non-HTML language are skipped.

## Classification

Each attribute is classified by its **normalized key**:

| Written as                  | Key                |
| --------------------------- | ------------------ |
| `foo`, `:foo`, `v-bind:foo` | `foo`              |
| `@click`, `v-on:click`      | `v-on:click`       |
| `#item`, `v-slot:item`      | `v-slot:item`      |
| `@click.stop`, `:foo.camel` | modifiers stripped |

A profile is a list of **groups** in output order. Each group has `match` patterns: an exact name, or a prefix ending in `*`. An attribute goes to the group whose pattern fits it **most specifically**: an exact name beats any prefix, and a longer prefix beats a shorter one. Attributes that match nothing go to the single **fallback** group.

Inside a group, the order is set by `sort`:

- `source`: as written (default).
- `alphabetical`: by key.
- `listed`: by the order of the `match` patterns.

## HTML profile

| #  | Group            | Attributes                                                                                                                                                    | Sort         |
| -- | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ |
| 1  | global           | `id`                                                                                                                                                          |              |
| 2  | class            | `class`                                                                                                                                                       |              |
| 3  | supplementary    | `alt`, `title`, `placeholder`                                                                                                                                 | listed       |
| 4  | semantic         | `type`, `href`, `src`, `srcset`                                                                                                                               | listed       |
| 5  | key-value        | `name`, `value`, `content`                                                                                                                                    | listed       |
| 6  | dimensions       | `width`, `height`                                                                                                                                             | listed       |
| 7  | other (fallback) | state attributes such as `disabled`, `checked`, `readonly`                                                                                                    | source       |
| 8  | aria-role        | `role`                                                                                                                                                        |              |
| 9  | aria-name        | `aria-label`, `aria-labelledby`                                                                                                                               | listed       |
| 10 | aria-description | `aria-describedby`, `aria-description`, `aria-details`                                                                                                        | listed       |
| 11 | aria-property    | other `aria-*`                                                                                                                                                | source       |
| 12 | aria-state       | `aria-busy`, `aria-checked`, `aria-current`, `aria-disabled`, `aria-expanded`, `aria-grabbed`, `aria-hidden`, `aria-invalid`, `aria-pressed`, `aria-selected` | source       |
| 13 | data             | `data-*`                                                                                                                                                      | alphabetical |
| 14 | events           | `on*`                                                                                                                                                         | source       |

Why this order:

- **Supplementary attributes come right after `id`/`class`.** They describe the element for a human reader. `id` and `class` stay together at the front because that is the common idiom.
- **ARIA goes near the end.** It carries extra metadata for assistive technology that `alt`/`title` can't express. It is not the primary description of the element.
- **ARIA follows WAI-ARIA**: role → accessible name → description → properties → states.
- **`data-*` is alphabetical** because no semantic order exists for it.

## Vue profile

The Vue profile follows the default order of eslint-plugin-vue's `vue/attributes-order`, so its output always satisfies that rule:

| #  | Tier             | Attributes                                                   |
| -- | ---------------- | ------------------------------------------------------------ |
| 1  | Definition       | `is`, `v-is`                                                 |
| 2  | List rendering   | `v-for`                                                      |
| 3  | Conditionals     | `v-if`, `v-else-if`, `v-else`, `v-show`, `v-cloak`           |
| 4  | Render modifiers | `v-pre`, `v-once`                                            |
| 5  | Global           | `id`                                                         |
| 6  | Unique / slot    | `ref`, `key`, `slot`, `v-slot*`                              |
| 7  | Two-way binding  | `v-model*`                                                   |
| 8a | Other directives | other `v-*`                                                  |
| 8b | Other attributes | the HTML profile's groups from `class` to `data` (see above) |
| 9  | Events           | `v-on*` (`@…`)                                               |
| 10 | Content          | `v-html`, `v-text`                                           |

Tier 8 is split into other directives first, then other attributes. This matches `OTHER_DIRECTIVES` → `OTHER_ATTR` in `vue/attributes-order`. The HTML sub-ordering applies inside the other-attributes tier.

```vue
<!-- Input -->
<MyItem
  @click="select"
  :class="{ active }"
  class="item"
  v-for="item in items"
  :key="item.id"
  aria-label="Item"
  v-bind:title="item.name"
  :itemName="item.name"
/>

<!-- Output -->
<MyItem
  v-for="item in items"
  :key="item.id"
  class="item"
  :class="{ active }"
  :title="item.name"
  :item-name="item.name"
  aria-label="Item"
  @click="select"
/>
```

## Same-name clustering

Because classification uses the normalized key, the static and bound forms of one attribute get the same key and stay together. The first occurrence of a key sets the position of its cluster. No special-case logic is needed.

- For **`class`**, the static form goes before the bound form (`class="item" :class="{ active }"`).
- For **other names**, the cluster keeps the source order. Vue merges `style` and the later declaration wins. For other duplicate names it keeps only one of them. Reordering them could change behavior.
- **`v-model` is never clustered with a plain attribute of the same name** (`value="x" v-model="y"`). Its key is `v-model`, not `value`, and it stays in its own tier because two-way binding is a different concept.

## `v-bind` spreads

`v-bind="object"` is a boundary. Attributes are never moved across a spread, because that can change which value wins when the object is merged.

The exception is groups marked `spreadSafe`. These are compile-time directives (`is`, `v-for`, `v-if`/…, `v-pre`/`v-once`, custom directives) that never merge with the spread object, so they can be moved across it. Everything else stays inside its segment.

## Normalization

The Vue profile enables two rewrites, which run before classification:

- `directiveShorthand`: `v-bind:foo` → `:foo`, `v-on:click` → `@click`.
- `hyphenateComponentProps`: camelCase attribute names on component tags → kebab-case (`:itemName` → `:item-name`). Native SVG and MathML elements are not affected.
