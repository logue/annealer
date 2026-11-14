> For AI agents: the complete documentation index is available at /annealer/zh/llms.txt, the full documentation bundle is available at /annealer/zh/llms-full.txt.

# 布局

对属性重新排序可能会破坏标签原有的行布局。配置文件的 `layout` 部分定义排序之后如何布局标签。每条规则都对应一条 eslint-plugin-vue 规则，因此保留这些规则的项目能得到一致的结果。

两个内置配置文件都开启了这些规则。

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

等效于 `vue/html-self-closing`。按元素种类分别设置为 `always`、`never` 或 `preserve`：

| 种类          | 示例           | 内置 `html` | 内置 `vue` |
| ----------- | ------------ | --------- | -------- |
| `void`      | `<br />`     | `always`  | `always` |
| `normal`    | `<div />`    | –         | `always` |
| `component` | `<MyComp />` | –         | `always` |

- `normal` 和 `component` 仅在 Vue 模板中生效。在纯 HTML 中，`<div />` 是一个未闭合的起始标签。
- 仅含空白的内容视为空，`pre` 和 `textarea` 除外。
- SFC 的顶层块（`<template>`、`<script>`、`<style>`）从不处理。

:::warning 与 vue/html-self-closing 的差异
`void: always` 与 `vue/html-self-closing` 的默认值（`html.void: never`）不同。如果你保留该 ESLint 规则，请在其中设置 `html.void: always`，或在 annealer 配置文件中设置 `void: never`。
:::

## `contentNewline`

等效于 `vue/multiline-html-element-content-newline`。省略该键即可关闭规则。`{}` 表示以默认值开启。

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

<!-- 输出（vue 配置文件） -->
<template>
  <div
    id="x"
    class="a"
  >
    <MyComp />
  </div>
</template>
```

HTML 配置文件同样开启了该规则。块级内容开头和结尾的空白不会被渲染，而 `ignore` 列表涵盖了可能会显示空白的元素。

将内容移到新行可能会改变嵌套标签的结束括号所对齐的缩进。因此，annealer 会反复格式化文档直到结果稳定（最多 4 轮，通常 2 轮）。
