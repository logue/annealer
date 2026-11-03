---
description: annealer 如何对起始标签进行词法分析、将属性分类到分组、聚合同名属性、尊重 v-bind 展开，以及规范化属性名。
---

# 属性顺序

## 词法分析器：不构建 DOM {#tokenizer-no-dom}

annealer 不构建 DOM 或 XML 树。它找出每个起始标签（`<tagname` … `>` 或 `/>`），并将属性列表拆分为名称/值对，只保留引号风格。

这种方式对纯 HTML 和 Vue 的非 XML 语法（`@click`、`#default`、`v-bind:[dynamic]`）同样适用。它还能避免基于 DOM 或 XML 的解析带来的副作用，例如合并空白或将标签名和属性名转为小写。

原始文本元素、注释、`{{ }}` 插值、SFC 的 `<script>` 块、自定义块以及使用非 HTML 语言的 `<template lang="…">` 会被跳过。

## 分类 {#classification}

每个属性都按其**规范化键**进行分类：

| 书写形式                    | 键            |
| --------------------------- | ------------- |
| `foo`, `:foo`, `v-bind:foo` | `foo`         |
| `@click`, `v-on:click`      | `v-on:click`  |
| `#item`, `v-slot:item`      | `v-slot:item` |
| `@click.stop`, `:foo.camel` | 去除修饰符    |

配置文件是按输出顺序排列的**分组**列表。每个分组都有 `match` 模式：精确名称，或以 `*` 结尾的前缀。属性会进入模式与其**最具体地**匹配的分组：精确名称优先于任何前缀，较长的前缀优先于较短的前缀。未匹配任何模式的属性进入唯一的 **fallback** 分组。

分组内的顺序由 `sort` 决定：

- `source`：按书写顺序（默认）。
- `alphabetical`：按键名。
- `listed`：按 `match` 模式的顺序。

## HTML 配置文件 {#html-profile}

| #   | 分组             | 属性                                                                                                                                                          | 排序         |
| --- | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ |
| 1   | global           | `id`                                                                                                                                                          |              |
| 2   | class            | `class`                                                                                                                                                       |              |
| 3   | supplementary    | `alt`, `title`, `placeholder`                                                                                                                                 | listed       |
| 4   | semantic         | `type`, `href`, `src`, `srcset`                                                                                                                               | listed       |
| 5   | key-value        | `name`, `value`, `content`                                                                                                                                    | listed       |
| 6   | dimensions       | `width`, `height`                                                                                                                                             | listed       |
| 7   | other (fallback) | `disabled`、`checked`、`readonly` 等状态属性                                                                                                                  | source       |
| 8   | aria-role        | `role`                                                                                                                                                        |              |
| 9   | aria-name        | `aria-label`, `aria-labelledby`                                                                                                                               | listed       |
| 10  | aria-description | `aria-describedby`, `aria-description`, `aria-details`                                                                                                        | listed       |
| 11  | aria-property    | 其他 `aria-*`                                                                                                                                                 | source       |
| 12  | aria-state       | `aria-busy`, `aria-checked`, `aria-current`, `aria-disabled`, `aria-expanded`, `aria-grabbed`, `aria-hidden`, `aria-invalid`, `aria-pressed`, `aria-selected` | source       |
| 13  | data             | `data-*`                                                                                                                                                      | alphabetical |
| 14  | events           | `on*`                                                                                                                                                         | source       |

为什么是这个顺序：

- **补充属性紧跟在 `id`/`class` 之后。** 它们为人类读者描述元素。`id` 和 `class` 一起放在最前面，因为这是常见的惯用写法。
- **ARIA 放在靠后的位置。** 它为辅助技术提供 `alt`/`title` 无法表达的额外元数据，而不是元素的主要描述。
- **ARIA 遵循 WAI-ARIA**：role → 可访问名称 → 描述 → 属性 → 状态。
- **`data-*` 按字母顺序排列**，因为它没有语义上的顺序。

## Vue 配置文件 {#vue-profile}

Vue 配置文件遵循 eslint-plugin-vue 的 `vue/attributes-order` 的默认顺序，因此其输出始终满足该规则：

| #   | 层级        | 属性                                                 |
| --- | ----------- | ---------------------------------------------------- |
| 1   | 定义        | `is`, `v-is`                                         |
| 2   | 列表渲染    | `v-for`                                              |
| 3   | 条件        | `v-if`, `v-else-if`, `v-else`, `v-show`, `v-cloak`   |
| 4   | 渲染修饰    | `v-pre`, `v-once`                                    |
| 5   | 全局        | `id`                                                 |
| 6   | 唯一 / 插槽 | `ref`, `key`, `slot`, `v-slot*`                      |
| 7   | 双向绑定    | `v-model*`                                           |
| 8a  | 其他指令    | 其他 `v-*`                                           |
| 8b  | 其他属性    | HTML 配置文件中从 `class` 到 `data` 的分组（见上文） |
| 9   | 事件        | `v-on*`（`@…`）                                      |
| 10  | 内容        | `v-html`, `v-text`                                   |

第 8 层被拆分为先其他指令、后其他属性。这与 `vue/attributes-order` 中的 `OTHER_DIRECTIVES` → `OTHER_ATTR` 一致。HTML 的子排序在其他属性层级内部应用。

```vue
<!-- 输入 -->
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

<!-- 输出 -->
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

## 同名聚合 {#same-name-clustering}

由于分类使用规范化键，同一属性的静态形式和绑定形式会得到相同的键，从而保持在一起。某个键第一次出现的位置决定其聚合簇的位置。无需任何特殊处理逻辑。

- 对于 **`class`**，静态形式排在绑定形式之前（`class="item" :class="{ active }"`）。
- 对于**其他名称**，聚合簇保持源顺序。Vue 会合并 `style`，且后出现的声明优先；对于其他重复名称，Vue 只保留其中一个。重新排序可能改变行为。
- **`v-model` 从不与同名的普通属性聚合**（`value="x" v-model="y"`）。它的键是 `v-model` 而非 `value`，并且由于双向绑定是不同的概念，它保持在自己的层级中。

## `v-bind` 展开 {#v-bind-spreads}

`v-bind="object"` 是一个边界。属性从不会跨越展开移动，因为这可能改变对象合并时哪个值优先。

例外是标记为 `spreadSafe` 的分组。它们是编译期指令（`is`、`v-for`、`v-if`/…、`v-pre`/`v-once`、自定义指令），从不与展开对象合并，因此可以跨越展开移动。其他所有属性都保持在各自的片段内。

## 规范化 {#normalization}

Vue 配置文件启用了两种改写，在分类之前执行：

- `directiveShorthand`：`v-bind:foo` → `:foo`，`v-on:click` → `@click`。
- `hyphenateComponentProps`：组件标签上的 camelCase 属性名 → kebab-case（`:itemName` → `:item-name`）。原生 SVG 和 MathML 元素不受影响。
