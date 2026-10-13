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

| Key             | Type   | Required | Description                                                    |
| --------------- | ------ | -------- | -------------------------------------------------------------- |
| `schemaVersion` | `1`    | yes      | Profile format version.                                        |
| `name`          | string | yes      | Profile name.                                                  |
| `description`   | string |          | Human-readable summary.                                        |
| `normalize`     | object |          | Attribute name rewrites.                                       |
| `layout`        | object |          | Start-tag layout rules.                                        |
| `groups`        | array  | yes      | Attribute groups in output order.                              |
| `stylesheet`    | object |          | Stylesheet formatting. Omit it to leave stylesheets untouched. |

## `normalize`

Rewrites applied before classification.

| Key                       | Type    | Default | Description                                                                                        |
| ------------------------- | ------- | ------- | -------------------------------------------------------------------------------------------------- |
| `directiveShorthand`      | boolean | `false` | `v-bind:foo` → `:foo`, `v-on:click` → `@click`.                                                    |
| `hyphenateComponentProps` | boolean | `false` | camelCase attribute names on component tags → kebab-case. Native SVG/MathML elements are excluded. |

## `layout`

See [Layout](../guide/features/layout.md) for details.

| Key                              | Type                              | Default                            | Description                                                                                                              |
| -------------------------------- | --------------------------------- | ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `closingBracketNewline`          | boolean                           | `false`                            | Put the closing bracket of a multiline start tag on its own line.                                                        |
| `oneAttributePerLine`            | boolean                           | `false`                            | One attribute per line in a multiline start tag.                                                                         |
| `selfClosing.void`               | `always` \| `never` \| `preserve` | `preserve`                         | Void elements: `<br />` or `<br>`.                                                                                       |
| `selfClosing.normal`             | `always` \| `never` \| `preserve` | `preserve`                         | Empty non-void elements (`<div />`). Vue templates only.                                                                 |
| `selfClosing.component`          | `always` \| `never` \| `preserve` | `preserve`                         | Empty components (`<MyComp />`). Vue templates only.                                                                     |
| `contentNewline`                 | object                            | off                                | Put the content of a multiline element on lines of its own. `{}` turns it on with the defaults.                          |
| `contentNewline.ignore`          | string[]                          | `pre`, `textarea`, inline elements | Elements left alone, with everything inside them. Element names match case-insensitively; component names match exactly. |
| `contentNewline.allowEmptyLines` | boolean                           | `false`                            | Keep blank lines before and after the content.                                                                           |

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

`stylesheet.malva` accepts malva's own options, such as `printWidth`, `useTabs`, `indentWidth`, `lineBreak`, `hexCase`, `hexColorLength`, `quotes`, `trailingComma`, `formatComments` and the `*.preferSingleLine` options. `declarationOrder` and `declarationOrderGroupBy` belong to `stylesheet` itself, not to `malva`. Unknown malva keys are rejected.
