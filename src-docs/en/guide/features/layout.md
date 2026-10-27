---
description: Layout rules for multiline start tags, self-closing elements and element content, equivalent to the matching eslint-plugin-vue rules.
---

# Layout

Reordering attributes can break the original line layout of a tag. The `layout` section of a profile defines how tags are laid out afterwards. Each rule mirrors an eslint-plugin-vue rule, so projects that keep those rules get consistent results.

Both built-in profiles turn `closingBracketNewline`, `oneAttributePerLine` and `selfClosing.void` (`<br />`, `<hr />`, `<wbr />`, …) on. Self-closing empty elements and components (`selfClosing.normal`/`component`) and `contentNewline` rewrite markup beyond what reordering needs, so the built-in profiles define them but leave them off (`enabled: false`). Turn them on when needed:

| Rule             | CLI                 | JavaScript             | Profile                        |
| ---------------- | ------------------- | ---------------------- | ------------------------------ |
| `selfClosing`    | `--self-closing`    | `selfClosing: true`    | `selfClosing.enabled: true`    |
| `contentNewline` | `--content-newline` | `contentNewline: true` | `contentNewline.enabled: true` |

The CLI flags and JavaScript options override the profile's `enabled`. `--no-self-closing` / `--no-content-newline` (or `false`) turn a rule off in a profile that enables it.

## `closingBracketNewline`

Equivalent to `vue/html-closing-bracket-newline`. The closing bracket of a multiline start tag goes on its own line. A start tag whose attributes fit on one line keeps `>` on that line (`singleline: never`).

## `oneAttributePerLine`

Equivalent to the multiline default of `vue/max-attributes-per-line`. In a start tag whose attributes span several lines, each attribute goes on its own line.

This is needed because reordering could otherwise put two attributes on one line if attributes kept their original line positions.

<!-- prettier-ignore -->
```html
<!-- Input -->
<div class="a"
  id="x"></div>

<!-- Output -->
<div
  id="x"
  class="a"
></div>
```

## `selfClosing`

Equivalent to `vue/html-self-closing`. Set per element kind, each to `always`, `never` or `preserve`. `void` always applies; `enabled` (off in the built-in `vue` profile, see above) covers `normal` and `component` only:

| Kind        | Example      | Built-in `html` | Built-in `vue` |
| ----------- | ------------ | --------------- | -------------- |
| `void`      | `<br />`     | `always`        | `always`       |
| `normal`    | `<div />`    | –               | `always`       |
| `component` | `<MyComp />` | –               | `always`       |

- `normal` and `component` apply in Vue templates only. In plain HTML, `<div />` is an unclosed start tag.
- Whitespace-only content counts as empty, except in `pre` and `textarea`.
- SFC top-level blocks (`<template>`, `<script>`, `<style>`) are never touched.

:::warning Difference from vue/html-self-closing
`void: always` differs from the default of `vue/html-self-closing` (`html.void: never`). If you keep that ESLint rule, set `html.void: always` there, or `void: never` in your annealer profile.
:::

## `contentNewline`

Equivalent to `vue/multiline-html-element-content-newline`. Off in the built-in profiles (see above). In your own profile, omit the key or set `enabled: false` to turn the rule off; `{}` turns it on with the defaults. `--content-newline` on a profile without the key also uses the defaults.

- An element is **multiline** when its end tag starts on a later line than its start tag (the ESLint rule's definition). Its content then starts and ends with exactly one line break.
- `allowEmptyLines: true` keeps blank lines instead of collapsing them.
- Empty elements are left alone.
- `ignore` lists elements that are left alone, together with everything inside them. The default is the rule's default: `pre`, `textarea` and inline elements (`a`, `span`, `strong`, …), where added whitespace could be visible.
- Unclosed HTML elements (`<li>` without `</li>`) are closed implicitly by an ancestor's end tag and are not touched.
- Only line breaks are added; indentation is not re-flowed. The new line takes the attribute indentation of a multiline start tag, else the first deeper content line, else the tag's indentation plus two spaces (a tab if the tag is indented with tabs).

<!-- prettier-ignore -->
```vue
<!-- Input -->
<template>
  <div class="a"
    id="x"><MyComp></MyComp></div>
</template>

<!-- Output (vue profile, --self-closing --content-newline) -->
<template>
  <div
    id="x"
    class="a"
  >
    <MyComp />
  </div>
</template>
```

The HTML profile defines the rule too. Whitespace at the start and end of block-level content is not rendered, and the `ignore` list covers the elements where it could be.

Moving content to a new line can change the indentation that a nested tag's closing bracket aligns to. annealer therefore formats the document again until it is stable (at most 4 passes, normally 2).
