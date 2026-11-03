---
description: annealer 目前的限制，涵盖支持的输入、标记与布局、类顺序、样式表、配置文件和 CLI。
---

# 限制

annealer 目前是 alpha 版本（0.1.0-alpha.1）。本页列出它尚未支持的功能，以及按设计不做的事情。每一项都链接到包含详细信息的页面。

## 现状 {#status}

- **Alpha 阶段。** 在 1.0 之前，任何版本都可能对相同输入产生不同的输出，遵循 Tailwind 的内置 Tailwind 类顺序除外。请锁定确切版本，并在升级时检查 diff。
- **npm 包不含 CLI。** 它只提供 JavaScript API；请通过 Cargo 安装 `anyl`（[快速开始](./getting-started.md#installation)）。
- **配置文件格式仍可能变化**，直到宣布稳定为止（`schemaVersion: 1`）。

## 支持的输入 {#supported-input}

- HTML、Vue 3 SFC 模板，以及 CSS、SCSS、Sass（缩进语法）和 Less。其他内容会被报告为不支持的文件类型。
- **不支持：** JSX/TSX、Svelte、Astro、Angular 模板、Markdown/MDX，以及 Vue 2 语法（`.sync`、`.native`）。Svelte 计划在以后支持。
- **不支持 Pug 和 Stylus。** `.pug`、`.jade`、`.styl` 和 `.stylus` 文件会被报告为不支持的文件类型，且不会读取。带有 `lang="pug"`、`lang="jade"`、`lang="stylus"` 或 `lang="styl"` 的 `<template>` 或 `<style>` 会导致整个文件失败（`unsupported language`），而不是被跳过；这样 annealer 既不会把 Pug 当作标记来扫描，也不会让其他块只被格式化一半。请将此类文件从传给 annealer 的路径中排除。
- 在 `.vue` 文件中，`<script>` 和自定义块（`<i18n>`）保持不变。

## 标记 {#markup}

- **不构建 DOM。** annealer 使用词法分析器读取起始标签。它不校验 HTML，也不修复损坏的标记。已废弃的标记（`<center>`、`bgcolor`）会被解析，但从不报告或修改。
- **不重排。** annealer 只在布局规则需要的地方添加或删除换行。它从不重新缩进，也不按行宽折行；那是格式化工具的工作（[与其他工具配合使用](./integration.md)）。
- **展开是边界。** 属性不会跨越 `v-bind="object"` 移动，编译期指令（`spreadSafe` 分组）除外。因此带有展开的标签可能部分保持未排序，`vue/attributes-order` 会对此报告（[属性顺序](../concepts/attribute-order.md)、[eslint-plugin-vue](./integration.md#eslint-plugin-vue)）。
- **外部内容保持原样。** 内联 SVG（`<svg>`）、MathML（`<math>`）和 RDF（`<rdf:RDF>`）等带命名空间的 XML 元素，连同其内部全部内容，都不会被处理：不进行属性排序、类排序、自闭合、`contentNewline` 或 `<style>` 格式化。在纯 HTML 中，`<svg>` 和 `<math>` 不区分大小写匹配；在 Vue 模板中，`<Svg>` 是组件。
- **同名聚合保持源顺序**，`class` 除外：静态形式排在绑定形式之前。`style` 和其他重复项保持书写顺序，因为它们的顺序可能影响行为。

## 布局 {#layout}

- **空元素默认自闭合**（`<br />`、`<hr />`、`<wbr />`）。这与 `vue/html-self-closing` 的默认值（`html.void: never`）不同；请在该规则中设置 `html.void: always`，或传入 `--no-void-self-closing`（在配置文件中为 `void: never`）。
- **内容为空的元素和组件的自闭合（`<div />`、`<MyComp />`）以及 `contentNewline` 默认关闭。** 使用 `--self-closing` / `--content-newline` 开启（[布局](../features/layout.md)）。`<div />` 仅在 Vue 模板中应用，因为在纯 HTML 中它是一个未闭合的起始标签；在纯 HTML 中，`--expand-non-void` 会将其改写为 `<div></div>`。
- `contentNewline` 会添加换行，但不会重新缩进内容。隐式闭合的元素（没有 `</li>` 的 `<li>`）保持不变。

## 类顺序 {#class-order}

- **仅限静态 `class` 属性。** 绑定的 `:class`、`clsx()`/`cn()` 调用以及含模板语法的值从不读取。
- **仅内置 Tailwind**，由 Tailwind CSS 4.3 及默认主题生成。你自己的主题值和 `@utility` 类是未知的：自定义颜色会像主题颜色一样排序，未知类排在最前（[类顺序](../features/class-order.md)）。
- **默认保留重复项**，而 `prettier-plugin-tailwindcss` 和 oxfmt 会删除它们。开启 `--remove-duplicate-classes` 可删除重复项。
- 尚无 Bootstrap 类顺序。

## 样式表 {#stylesheets}

- **样式表会被 malva 重新格式化**，而不仅仅是排序。malva 的布局在某些地方与 Prettier、oxfmt 和 Biome 不同。例如，对于逗号分隔的值（`transition`、`font-family`），malva 保持在一行，而它们会拆成多行。没有任何 malva 选项能消除这一差异，因此 `anyl --check` 可能会在格式化工具刚写出的文件上失败。请在格式化工具之前运行 annealer（[运行顺序](./integration.md#run-order)）。
- **CSS 字符串默认使用单引号**（`quotes: prefer-single`）。如果格式化工具输出双引号，请让两者之一保持一致（[需要对齐的设置](./integration.md#settings-to-align)）。
- 带厂商前缀的声明仅在设置了 `declarationOrder` 时才会移动。声明之间夹有注释的连续段保持不变（[样式表](../concepts/stylesheets.md)）。
- 静态 `style` 属性仅在开启 `stylesheet.styleAttribute` 时排序，且当其包含模板语法、注释或无法解析为普通声明的值时不排序。绑定的 `:style` 从不处理。

## 配置文件与 CLI {#profiles-and-the-cli}

- **不支持配置文件组合（`extends`）。** 要修改内置配置文件的某个设置，需要复制整个配置文件（`crates/annealer/profiles/*.yaml` 或 `builtinProfile()`）并编辑副本。
- **没有项目级设置文件。** `--profile`、`--class-order`、`--self-closing` 和 `--print-width` 等选项需要在每次运行时传入（可写入 `package.json` 脚本）。
- **`anyl` 只接受文件路径。** 它不会遍历目录、展开通配符或读取忽略文件。请从 shell 或 `git ls-files` 传入路径。
- 没有编辑器扩展、语言服务器或 Prettier 插件。请通过 CLI、Git 钩子或 JavaScript API 运行 annealer。
