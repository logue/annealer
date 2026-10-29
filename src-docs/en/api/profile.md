---
description: Reference for the annealer profile YAML format, with every key, its type and its default.
---

# Profile reference

A profile is a YAML file that defines attribute order, layout rules and stylesheet options. The built-in profiles are in [`crates/annealer/profiles/`](https://github.com/logue/annealer/tree/main/crates/annealer/profiles).

```yaml
# yaml-language-server: $schema=https://github.com/logue/annealer/raw/refs/heads/main/crates/annealer/schema/profile.schema.json
schemaVersion: 1
name: my-profile
description: My attribute order.

normalize:
  directiveShorthand: true

layout:
  closingBracketNewline: true
  oneAttributePerLine: true
  selfClosing:
    void: always
  contentNewline: {}

groups:
  - name: global
    match: [id]
  - name: class
    match: [class]
  - name: other
    fallback: true
  - name: events
    match: ['on*']

stylesheet:
  declarationOrder: smacss
  vendorPrefix: start
```

:::tip Editor support
The JSON Schema (draft-07) is at `crates/annealer/schema/profile.schema.json`. With the modeline above and the [YAML extension](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-yaml), VS Code validates and completes profiles.
:::

Unknown keys are errors everywhere in a profile, so typos never go unnoticed.

## Top level

| Key             | Type             | Required | Description                                                                        |
| --------------- | ---------------- | -------- | ---------------------------------------------------------------------------------- |
| `schemaVersion` | `1`              | yes      | Profile format version.                                                            |
| `name`          | string           | yes      | Profile name.                                                                      |
| `description`   | string           |          | Human-readable summary.                                                            |
| `normalize`     | object           |          | Attribute name rewrites.                                                           |
| `layout`        | object           |          | Start-tag layout rules.                                                            |
| `groups`        | array            | yes      | Attribute groups in output order.                                                  |
| `stylesheet`    | object           |          | Stylesheet formatting. Omit it to leave stylesheets untouched.                     |
| `classOrder`    | string \| object |          | Class order for static `class` attributes. Omit it to keep classes in their order. |

## `normalize`

Rewrites applied before classification.

| Key                       | Type    | Default | Description                                                                                        |
| ------------------------- | ------- | ------- | -------------------------------------------------------------------------------------------------- |
| `directiveShorthand`      | boolean | `false` | `v-bind:foo` → `:foo`, `v-on:click` → `@click`.                                                    |
| `hyphenateComponentProps` | boolean | `false` | camelCase attribute names on component tags → kebab-case. Native SVG/MathML elements are excluded. |

## `layout`

See [Layout](../guide/features/layout.md) for details.

| Key                              | Type                              | Default                            | Description                                                                                                                                                                        |
| -------------------------------- | --------------------------------- | ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `closingBracketNewline`          | boolean                           | `false`                            | Put the closing bracket of a multiline start tag on its own line.                                                                                                                  |
| `oneAttributePerLine`            | boolean                           | `false`                            | One attribute per line in a multiline start tag.                                                                                                                                   |
| `selfClosing.enabled`            | boolean                           | `true`                             | Whether `normal` and `component` apply (`void` and `expandNonVoid` always do). The built-in `vue` profile sets `false`; `--self-closing` and the `selfClosing` option override it. |
| `selfClosing.void`               | `always` \| `never` \| `preserve` | `preserve`                         | Void elements: `<br />` or `<br>`. `--void-self-closing` and the `voidSelfClosing` option override it.                                                                             |
| `selfClosing.normal`             | `always` \| `never` \| `preserve` | `preserve`                         | Empty non-void elements (`<div />`). Vue templates only.                                                                                                                           |
| `selfClosing.component`          | `always` \| `never` \| `preserve` | `preserve`                         | Empty components (`<MyComp />`). Vue templates only.                                                                                                                               |
| `selfClosing.expandNonVoid`      | boolean                           | `false`                            | `<div />` → `<div></div>`. Plain HTML only; `<svg>` and `<math>` content is left alone. `--expand-non-void` and the `expandNonVoid` option override it.                            |
| `contentNewline`                 | object                            | off                                | Put the content of a multiline element on lines of its own. `{}` turns it on with the defaults.                                                                                    |
| `contentNewline.enabled`         | boolean                           | `true`                             | Whether the rule applies. The built-in profiles set `false`; `--content-newline` and the `contentNewline` option override it.                                                      |
| `contentNewline.ignore`          | string[]                          | `pre`, `textarea`, inline elements | Elements left alone, with everything inside them. Element names match case-insensitively; component names match exactly.                                                           |
| `contentNewline.allowEmptyLines` | boolean                           | `false`                            | Keep blank lines before and after the content.                                                                                                                                     |

## `groups`

Groups are listed in output order. Exactly one group must be the fallback, and group names must be unique.

| Key          | Type                                   | Default  | Description                                                                                            |
| ------------ | -------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------ |
| `name`       | string                                 | –        | Unique group name. Required.                                                                           |
| `match`      | string[]                               | –        | Patterns: an exact name or a prefix ending in `*`. Required unless `fallback: true`.                   |
| `sort`       | `source` \| `alphabetical` \| `listed` | `source` | Order within the group: as written, by key, or by the order of `match`.                                |
| `fallback`   | boolean                                | `false`  | Receives every attribute that matches no pattern.                                                      |
| `spreadSafe` | boolean                                | `false`  | Attributes in this group may move across a `v-bind="object"` spread. Only for compile-time directives. |

Patterns are matched against **normalized, lowercased keys**: `foo` for `foo`/`:foo`/`v-bind:foo`, `v-on:click` for `@click`, `v-slot:name` for `#name`, with modifiers stripped. An exact name beats any prefix, and a longer prefix beats a shorter one. See [Attribute order](../guide/concepts/attribute-order.md).

## `stylesheet`

See [Stylesheets](../guide/concepts/stylesheets.md) for details.

| Key                       | Type                                                  | Default           | Description                                                                  |
| ------------------------- | ----------------------------------------------------- | ----------------- | ---------------------------------------------------------------------------- |
| `declarationOrder`        | `smacss` \| `concentric` \| `alphabetical`            | –                 | Property order. Without it, declarations are not reordered.                  |
| `declarationOrderGroupBy` | `non-declaration` \| `non-declaration-and-empty-line` | `non-declaration` | How declarations are split into runs that are sorted separately.             |
| `vendorPrefix`            | `alongside` \| `start`                                | `alongside`       | `start` puts vendor-prefixed declarations first in each run, alphabetically. |
| `styleAttribute`          | boolean                                               | `false`           | Also order declarations in static `style` attributes.                        |
| `malva`                   | object                                                | –                 | Other [malva options](https://malva.netlify.app/config/), passed through.    |

`stylesheet.malva` accepts malva's own options, such as `printWidth`, `useTabs`, `indentWidth`, `lineBreak`, `hexCase`, `hexColorLength`, `quotes`, `trailingComma`, `formatComments` and the `*.preferSingleLine` options. `declarationOrder` and `declarationOrderGroupBy` belong to `stylesheet` itself, not to `malva`. Unknown malva keys are rejected. `quotes` defaults to `prefer-single` in annealer (malva's own default is `always-double`); see [Quotes](../guide/concepts/stylesheets.md#quotes). `--print-width` (JavaScript: `printWidth`) overrides `printWidth` without editing the profile.

## `classOrder`

The order of the classes in static `class` attributes (see [Class order](../guide/features/class-order.md)): a built-in class order name (`tailwind`), or a [class order profile](#class-order-profiles) written inline. The CLI's `--class-order` and the JavaScript `classOrder` option override it.

```yaml
classOrder: tailwind
```

## Class order profiles

A class order profile is a YAML file of its own, passed with `--class-order` or `classOrder`, or written inline under a profile's `classOrder`. The JSON Schema is `crates/annealer/schema/class-order.schema.json`; the built-in one is [`crates/annealer/class-orders/tailwind.yaml`](https://github.com/logue/annealer/tree/main/crates/annealer/class-orders).

| Key                   | Type                | Default | Description                                                                                                                    |
| --------------------- | ------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `schemaVersion`       | `1`                 | –       | Required. Format version.                                                                                                      |
| `name`                | string              | –       | Required. Class order name.                                                                                                    |
| `description`         | string              | –       | Human-readable summary.                                                                                                        |
| `unknown`             | `start` \| `end`    | `start` | Where classes no rule matches go, in source order.                                                                             |
| `removeDuplicates`    | boolean             | `false` | Drop repeated classes, keeping the first. `--remove-duplicate-classes` overrides it.                                           |
| `important`           | string              | –       | Marker stripped from the start or end of a utility before matching (`!`).                                                      |
| `arbitraryProperties` | boolean             | `false` | Classes of the form `[property:value]` set `property`.                                                                         |
| `variants`            | object              | –       | `separator` (e.g. `:`) and `order`: variant patterns by rank, lowest first. A list entry gives its patterns the same rank.     |
| `propertyOrder`       | string[]            | –       | Required. CSS properties in output order.                                                                                      |
| `values`              | map of pattern list | –       | Value sets for `{name}` in patterns. `number` is built in.                                                                     |
| `rules`               | array               | –       | Required. `match` (patterns), `properties` (from `propertyOrder`) and optional `count` (number of declarations, if different). |

A class with a variant that no pattern matches is unknown. Rules may only name properties listed in `propertyOrder`; anything else is an error.
