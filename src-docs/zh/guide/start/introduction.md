---
description: annealer 是什么、为什么属性顺序很重要、它与 vue/attributes-order 的关系，以及其适用范围与非目标。
---

# 简介

annealer 是一个按语义对 HTML 和 Vue 模板属性进行排序的格式化工具。它同样会对样式表（CSS、SCSS、Sass、Less）中的声明进行排序。排序规则由可替换的 YAML 配置文件定义，而不是硬编码在程序中。

```html
<!-- 输入 -->
<a title="Home" href="/" id="home">Home</a>

<!-- 输出（html 配置文件） -->
<a id="home" title="Home" href="/">Home</a>
```

:::warning Alpha
当前版本为 annealer 0.1.0-alpha.1。格式化规则和默认值在不同版本之间仍可能变化（内置的 Tailwind 类顺序遵循 Tailwind，属于例外），配置文件格式在宣布稳定之前也可能变化。
:::

## 为什么属性顺序很重要 {#why-attribute-order-matters}

### 顺序是代码审查问题，而非外观问题

当多个贡献者长期编辑同一个组件时，属性会被添加、删除和改写。如果没有固定的规则，属性列表会被打乱，diff 中会混入实际上没有变化的行。基于规则的稳定顺序可以让 diff 只包含真正的改动，从而降低审查成本。

### 属性承载着不同的语义分量

HTML 属性并不是一组无序的字符串：

- 有些定义元素**是什么**（`is`、组件名）。
- 有些控制其**状态或行为**（`v-if`、`v-model`、`disabled`、`width`/`height`）。
- 有些为人类读者**补充含义**（`title`、`alt`）。
- 有些将含义**传递**给辅助技术等外部系统（`aria-*`）。

按字母顺序排序或保持书写顺序都会掩盖这种结构。annealer 对属性进行排序，使元素的语义结构在源代码中一目了然。

这种分组并非随意的偏好。WAI-ARIA 规范本身就将 `aria-*` 属性分为*属性（properties）*（使用过程中不变）和*状态（states）*（会变化）。annealer 采用了这一划分。

### 样式表同样值得这样对待

CSS 属性也有各自的角色：布局、外观、依赖状态的表现。"按语义排序"的原则同样适用于声明顺序。

## 与 `vue/attributes-order` 的关系 {#relation-to-vue-attributes-order}

Vue.js 风格指南已经为属性定义了优先级列表，eslint-plugin-vue 将其实现为 `vue/attributes-order`。annealer 的 Vue 配置文件以相同的默认顺序为起点，因此不再使用 ESLint 的项目也能得到相同的结果。

在此基础上，annealer 增加了：

- **同名聚合**：同一属性的静态形式和绑定形式保持在一起（`class` 紧挨 `:class`）。
- **ARIA 子排序**：role → name → description → property → state，遵循 WAI-ARIA 的属性/状态划分。
- **纯 HTML 支持**，适用于不使用 Vue 模板的项目。
- **样式表声明顺序**，交由 [malva](https://github.com/g-plane/malva) 处理。

annealer 并不是 `vue/attributes-order` 的竞争对手。它继承并扩展了后者的设计。对于用 rslint 或 oxlint 取代 ESLint 的团队，它让这项功能得以延续。

## 为什么要做一个新工具 {#why-a-new-tool}

我们评估了 Biome、rslint、oxlint、oxfmt 和 markup_fmt。它们都不能按语义对 Vue/HTML 属性排序：

| 工具                                 | 现状                                                                                                                   |
| ------------------------------------ | ---------------------------------------------------------------------------------------------------------------------- |
| Biome（`useSortedAttributes`，HTML） | 分类固定且未文档化；不会像 Vue 风格指南那样放置 `id` 等非指令属性；与一个已知的回归问题（属性重复/丢失）共用基础设施。 |
| rslint                               | 没有支持 Vue 的计划（维护者的决定）。                                                                                  |
| oxlint                               | Vue 规则仅关注正确性；没有属性顺序规则。                                                                               |
| oxfmt                                | 按设计无法处理模板标记。                                                                                               |
| markup_fmt                           | 可以格式化 Vue/Svelte/Astro 模板，但没有对属性排序的选项。                                                             |

CSS 属性顺序是个例外：malva 已经提供了可配置的 `declarationOrder`，所以 annealer 直接委托给它。**annealer 新增的是基于语义的属性分组，其余部分都不是从零构建的。**

## 适用范围 {#scope}

- 对 HTML 以及 Vue 3 SFC `<template>` 块中的属性排序。
- 属性规范化：`v-bind:` → `:`，`v-on:` → `@`，组件 prop 的 camelCase → kebab-case。
- 多行标签的布局规则，相当于 `vue/html-closing-bracket-newline`、`vue/max-attributes-per-line`（多行）、`vue/html-self-closing`（空元素始终自闭合，其余可选），以及可选的 `vue/multiline-html-element-content-newline`。
- 通过 malva 对 `<style>` 块和独立文件中的 CSS/SCSS/Sass/Less 声明排序。
- 可选：对静态 `style` 属性中的声明排序。
- 标记和样式表注释中的[禁用指令](../features/disable-directives.md)。
- 可选：对静态 `class` 属性进行[类排序](../features/class-order.md)，内置 Tailwind 顺序。
- 计划中：[库配置文件](../design/library-profiles.md)（Bootstrap 等）。
- Svelte 不在 0.1 的范围内。基于配置文件的设计应当允许日后在不重新设计的情况下加入。

## 非目标 {#non-goals}

- 重排文本或缩进。行宽和缩进交给外部的格式化工具（`rs fmt`、Prettier 等）。
- Vue 2 语法（`.sync`、`.native` 等）。
- 通用的 HTML/CSS 校验。
- 处理已废弃或过时的标记（`<xmp>`、`<center>`、`<font>`、`<marquee>`、`bgcolor` 等）。annealer 从不对其发出警告、删除或替换。它只在周围文档需要的范围内解析这些标记（例如 `<xmp>` 的内容是原始文本，因此其中的标签保持不变）。检测它们是 linter 或校验器的工作。

## 下一步 {#next-steps}

- [快速开始](./getting-started.md)：通过 CLI 或 JavaScript 运行 annealer。
- [与其他工具配合使用](./integration.md)：Rslint/`rs fmt`、oxlint/oxfmt 和 Biome 的配置。
- [限制](./limitations.md)：annealer 目前尚不支持的功能。
- [属性顺序](../concepts/attribute-order.md)：属性如何被分类和排序。
