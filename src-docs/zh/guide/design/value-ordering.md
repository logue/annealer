---
description: 使用库专用的类顺序配置文件对静态 class 属性中的标记进行排序的原则，以及仍待解决的问题。
---

# 值排序

:::info 状态
两者均已实现：静态 `style` 属性（可选，参见[样式表](../concepts/stylesheets.md#static-style-attributes-opt-in)）和使用可替换类顺序的 `class` 值（参见[类顺序](../features/class-order.md)），并内置 `tailwind` 顺序。Bootstrap 配置文件尚未编写。本页记录其背后的原则。
:::

属性顺序是一个维度，属性内部值的顺序是另一个维度。这一需求并非 Tailwind 所特有：任何基于类的 CSS 框架（Bootstrap、Tailwind、项目自己的工具类集合）都能从固定的类顺序中受益，理由与属性相同——保持 diff 稳定。

## 原则 {#principles}

### 仅限静态值 {#static-values-only}

绑定的属性（`:class`、`:style`、`v-bind:*`）包含的是表达式，而不是值。annealer 从不查看其内部。

这与 `eslint-plugin-tailwindcss`（`classnames-order`）和 `prettier-plugin-tailwindcss` 不同，后两者还会对 `:class` 表达式和 `clsx()` 风格调用中的字符串字面量排序。

### 与库无关的机制，搭配库专用的配置文件 {#a-library-agnostic-mechanism-with-library-specific-profiles}

类顺序与属性分组一样，是普通的配置文件数据，而不是代码分支。这是[库配置文件](./library-profiles.md)的第一个具体案例。

### 类顺序归库所有 {#the-library-owns-its-class-order}

一个框架的标准类顺序是该框架的设计决策，而不是 annealer 的。annealer 提供机制和配置文件格式。

- **Bootstrap**：annealer 携带草案配置文件向上游提出这一机制（Bootstrap 6 正在开发中），由 Bootstrap 自己定义和维护顺序。
- **Tailwind**：顺序已由 Tailwind 的引擎定义。annealer 将其作为内置 YAML（`tailwind`）提供，由 Tailwind 生成并与之核对。它遵循 `eslint-plugin-tailwindcss` `classnames-order` 的规范，只是保留重复项（见下文）。

### 配置文件格式是公共契约 {#the-profile-format-is-a-public-contract}

预期会有第三方编写和分发配置文件。在提出提案之前，`schemaVersion`、JSON Schema 和格式文档必须稳定。在 annealer 之外分发的配置文件还需要一个加载途径（例如由 `extends` 解析包中附带的文件）。

### 近似且自包含的顺序 {#an-approximate-self-contained-order}

Tailwind 的官方顺序来源于每个项目自己的 Tailwind 构建（v4 的 `@theme` 和自定义工具类、v3 的 `tailwind.config.js`），只有 Tailwind 自身才能计算出来。annealer 则用 YAML 定义顺序：

- 对标准工具类而言接近官方顺序，
- 自定义工具类是未知的，
- 不需要 JavaScript 运行时。

### 绝不改变含义 {#never-change-meaning}

对类重新排序不会改变 CSS 的语义。重复的类会被保留（删除它们是 lint 的职责），未知的类也从不会被丢弃。

## 设计 {#design}

- **类按其设置的 CSS 属性排名**，采用 Tailwind 的比较方式：未知类、变体、属性顺序中第一个不同的属性、属性数量，最后是名称。属性与库无关，因此 Bootstrap 配置文件可以使用相同的格式（`d-flex` 设置 `display`，`mt-3` 设置 `margin-top`）。
- **变体**（Tailwind 的 `md:`、`hover:`）是一个独立的维度：一个分隔符和一份按等级排列的变体模式列表。
- **模式**比属性模式更丰富：`*` 可出现在任意位置，并支持 `{name}` 取值集合，最具体的匹配胜出。`text-center`、`text-lg` 和 `text-red-500` 会解析到不同的规则。
- **类顺序是独立的配置文件**，不属于属性配置文件，因此可以与任何属性配置文件组合：`--class-order`、`classOrder` 选项，或配置文件中的 `classOrder`。

## 已解决的问题 {#resolved-questions}

- 未知类放在哪里：`unknown: start`（Tailwind 的顺序）或 `end`。
- `class` 值内部的空白会被保留：类在现有的分隔符之间移动，与 `eslint-plugin-tailwindcss` 的做法相同。

## 待解决的问题 {#open-questions}

- 中缀断点（Bootstrap 的 `col-md-6`、`d-lg-none`）尚未成为独立的维度。配置文件只能通过模式对其排名。
- 属性配置文件如何组合（例如 `extends: vue`）。这同样服务于库专用的属性规则。
