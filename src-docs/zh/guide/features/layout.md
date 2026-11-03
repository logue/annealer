---
description: 多行起始标签、自闭合元素和元素内容的布局规则，与对应的 eslint-plugin-vue 规则等效。
---

# 布局

对属性重新排序可能会破坏标签原有的行布局。配置文件的 `layout` 部分定义排序之后如何布局标签。每条规则都对应一条 eslint-plugin-vue 规则，因此保留这些规则的项目能得到一致的结果。

两个内置配置文件都开启了 `closingBracketNewline`、`oneAttributePerLine` 和 `selfClosing.void`（`<br />`、`<hr />`、`<wbr />` 等）。内容为空的元素和组件的自闭合（`selfClosing.normal`/`component`）以及 `contentNewline` 对标记的改写超出了排序所需，因此内置配置文件虽然定义了它们，但默认关闭（`enabled: false`）。需要时可将其开启：

| 规则             | CLI                 | JavaScript             | 配置文件                          |
| ---------------- | ------------------- | ---------------------- | --------------------------------- |
| `selfClosing`    | `--self-closing`    | `selfClosing: true`    | `selfClosing.enabled: true`       |
| `contentNewline` | `--content-newline` | `contentNewline: true` | `contentNewline.enabled: true`    |
| `expandNonVoid`  | `--expand-non-void` | `expandNonVoid: true`  | `selfClosing.expandNonVoid: true` |

CLI 参数和 JavaScript 选项会覆盖配置文件中的 `enabled`。在开启了某条规则的配置文件中，`--no-self-closing` / `--no-content-newline`（或 `false`）可将其关闭。

## `closingBracketNewline`

等效于 `vue/html-closing-bracket-newline`。多行起始标签的结束括号单独占一行。属性能放在一行内的起始标签，`>` 保持在该行（`singleline: never`）。

## `oneAttributePerLine`

等效于 `vue/max-attributes-per-line` 的多行默认值。在属性跨越多行的起始标签中，每个属性单独占一行。

之所以需要这条规则，是因为如果属性保持原来的行位置，重新排序后可能会出现两个属性挤在同一行的情况。

<!-- prettier-ignore -->
```html
<!-- 输入 -->
<div class="a"
  id="x"></div>

<!-- 输出 -->
<div
  id="x"
  class="a"
></div>
```

## `selfClosing`

等效于 `vue/html-self-closing`。按元素种类分别设置为 `always`、`never` 或 `preserve`。`void` 始终生效；`enabled`（在内置 `vue` 配置文件中为关闭，见上文）只作用于 `normal` 和 `component`：

| 种类        | 示例         | 内置 `html` | 内置 `vue` |
| ----------- | ------------ | ----------- | ---------- |
| `void`      | `<br />`     | `always`    | `always`   |
| `normal`    | `<div />`    | –           | `always`   |
| `component` | `<MyComp />` | –           | `always`   |

- `normal` 和 `component` 仅在 Vue 模板中生效。在纯 HTML 中，`<div />` 是一个未闭合的起始标签。
- `--void-self-closing` / `--no-void-self-closing`（JavaScript：`voidSelfClosing: true` / `false`）可以在不修改配置文件的情况下将 `void` 设为 `always` / `never`。
- 仅含空白的内容视为空，`pre` 和 `textarea` 除外。
- SFC 的顶层块（`<template>`、`<script>`、`<style>`）从不处理。

### `expandNonVoid`

纯 HTML 会忽略非空元素的斜杠，因此 `<div />` 等同于 `<div>`：元素保持打开状态，并吞掉其后的所有内容。`expandNonVoid: true`（`--expand-non-void`，JavaScript `expandNonVoid: true`）会为这类元素补上结束标签。由于它会改变浏览器解析标记的方式，内置配置文件中默认关闭。

<!-- prettier-ignore -->
```html
<!-- 输入 -->
<div class="spacer" /><my-icon/>
<svg><path d="M0 0" /></svg>

<!-- 输出（--expand-non-void） -->
<div class="spacer"></div><my-icon></my-icon>
<svg><path d="M0 0" /></svg>
```

- 外部内容（`<svg>`、`<math>`、`<rdf:RDF>` 等带命名空间的 XML）保持不变：在那里自闭合是合法的（[限制](../start/limitations.md)）。
- Vue 模板不受影响；在 Vue 模板中请使用 `normal: never` / `component: never`。

:::warning 与 vue/html-self-closing 的差异
`void: always` 与 `vue/html-self-closing` 的默认值（`html.void: never`）不同。如果你保留该 ESLint 规则，请在其中设置 `html.void: always`，或在 annealer 配置文件中设置 `void: never`。
:::

## `contentNewline`

等效于 `vue/multiline-html-element-content-newline`。在内置配置文件中为关闭（见上文）。在你自己的配置文件中，省略该键或设置 `enabled: false` 即可关闭规则；`{}` 表示以默认值开启。在没有该键的配置文件上使用 `--content-newline` 时，同样使用默认值。

- 当元素的结束标签起始于比起始标签更靠后的行时，该元素是**多行**的（与 ESLint 规则的定义相同）。此时其内容的开头和结尾各有且仅有一个换行。
- `allowEmptyLines: true` 会保留空行，而不是将其合并。
- 空元素保持不变。
- `ignore` 列出保持不变的元素（连同其内部全部内容）。默认值即该规则的默认值：`pre`、`textarea` 以及行内元素（`a`、`span`、`strong` 等），在这些元素中添加的空白可能会显示出来。
- 未闭合的 HTML 元素（没有 `</li>` 的 `<li>`）由祖先元素的结束标签隐式闭合，不会被处理。
- 只会添加换行，不会重排缩进。新行采用多行起始标签的属性缩进；若无，则采用第一行更深层的内容缩进；若仍无，则采用标签缩进再加两个空格（如果标签使用制表符缩进，则加一个制表符）。

<!-- prettier-ignore -->
```vue
<!-- 输入 -->
<template>
  <div class="a"
    id="x"><MyComp></MyComp></div>
</template>

<!-- 输出（vue 配置文件，--self-closing --content-newline） -->
<template>
  <div
    id="x"
    class="a"
  >
    <MyComp />
  </div>
</template>
```

HTML 配置文件同样定义了这条规则。块级内容开头和结尾的空白不会被渲染，而 `ignore` 列表涵盖了可能会被渲染的元素。

将内容移到新行可能会改变嵌套标签的结束括号所对齐的缩进。因此，annealer 会反复格式化文档直到结果稳定（最多 4 轮，通常 2 轮）。
