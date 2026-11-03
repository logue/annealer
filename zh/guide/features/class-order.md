> For AI agents: the complete documentation index is available at /annealer/zh/llms.txt, the full documentation bundle is available at /annealer/zh/llms-full.txt.

# 类顺序

annealer 可以对静态 `class` 属性中的类重新排序。顺序来自**类顺序配置文件**，这是一个独立于属性配置文件的 YAML 文件，因此可以替换为其他库的顺序。

该功能默认关闭。通过类顺序名称或文件开启：

```bash
anyl --class-order tailwind src/**/*.vue
anyl --class-order ./my-classes.yaml index.html
```

```ts
format(source, { language: 'vue', classOrder: 'tailwind' });
```

属性配置文件也可以通过 [`classOrder`](/annealer/zh/api/profile.md#classorder) 指定一个类顺序。`--class-order` 和 `classOrder` 选项会覆盖它。

## 内置：`tailwind` \{#built-in-tailwind}

内置的 `tailwind` 类顺序遵循 `eslint-plugin-tailwindcss`（`classnames-order`）和 `prettier-plugin-tailwindcss` 所采用的顺序。两者都从 Tailwind CSS 编译器获取该顺序。

<!-- prettier-ignore -->

```html
<!-- 输入 -->
<div class="p-3 border-gray-300 m-4 h-24 lg:p-4 flex border-2 lg:m-4 card"></div>

<!-- 输出 -->
<div class="card m-4 flex h-24 border-2 border-gray-300 p-3 lg:m-4 lg:p-4"></div>
```

annealer 并不运行 Tailwind。该顺序是数据，由 `scripts/generate-tailwind-order.mjs` 根据 Tailwind CSS 4.3 及其默认主题生成。同一脚本还会写出一个由 Tailwind 自身排序的、包含 3,000 个类列表的测试语料，annealer 的输出与其全部一致。

与运行 Tailwind 的区别：

- **你自己的主题和工具类是未知的。** 自定义颜色（`bg-brand-500`）会像主题颜色一样排序，因为未匹配其他规则的值会回退到颜色规则。自定义工具类（`@utility`）是未知类。
- **默认保留重复项。** `eslint-plugin-tailwindcss` 默认会删除它们（`removeDuplicates`）。删除类属于 lint 的职责，因此内置配置文件会保留它们。如需删除，请传入 `--remove-duplicate-classes`（JavaScript：`removeDuplicateClasses: true`），或使用带有 `removeDuplicates: true` 的类顺序。该选项在没有类顺序时同样有效：类保持原有顺序，只去掉重复项。
- **仅限静态 `class` 属性。** `:class` 绑定和 `clsx()` 风格的调用是表达式，annealer 从不读取它们。

## 类的排序方式 \{#how-classes-are-ordered}

1. 未匹配任何规则的类（`card`、`js-toggle`）排在最前，保持源顺序（`unknown: end` 会将它们放在最后）。
2. 没有变体的类排在前面。带变体的类随后，按其变体的等级（`variants.order`）排序。由最高等级决定，类似于比较位掩码。
3. 然后按类所设置的 CSS 属性排序：第一个不同的属性在 `propertyOrder` 中的位置。
4. 然后属性更多的类在前（`count`）。
5. 最后按名称排序，连续的数字按数值比较（`p-2` 在 `p-10` 之前）。

这正是 Tailwind 使用的比较方式。其中没有任何 Tailwind 特有的内容：任何工具类库都可以通过类所设置的属性来描述其类。

## 标记中会发生的变化 \{#what-changes-in-the-markup}

- 空白保持原位：类在现有的分隔符之间移动，因此多行类列表会保留其换行。
- 含模板语法（`{{ }}`、`{% %}`、`<?php ?>`、`${}`）的值以及没有引号的值保持原样。
- [禁用指令](/annealer/zh/guide/features/disable-directives.md)的作用方式与属性相同。

## 编写类顺序 \{#writing-a-class-order}

```yaml
# yaml-language-server: $schema=https://github.com/logue/annealer/raw/refs/heads/main/crates/annealer/schema/class-order.schema.json
schemaVersion: 1
name: my-utilities
unknown: end
propertyOrder: [display, margin, padding, color]
values:
  size: [sm, md, lg]
rules:
  - match: [d-*]
    properties: [display]
  - match: ['m-{number}', 'm-{size}', mx-auto]
    properties: [margin]
  - match: ['p-{number}']
    properties: [padding]
  - match: [text-*]
    properties: [color]
```

- `match` 模式：`*` 匹配任意长度的字符；`{name}` 匹配 `values` 中某个取值集合的一项。`{number}`（十进制数）为内置。
- 最具体的模式胜出：首先是精确名称，然后是字面字符最多的，再然后是取值集合最多的。
- 当去掉 `/` 之后修饰符（`bg-red-500/50`）的类匹配得更好时，该修饰符会被忽略。

所有键参见[类顺序参考](/annealer/zh/api/profile.md#class-order-profiles)。JavaScript API 中的 `builtinClassOrder('tailwind')` 会返回内置的 YAML，可作为起点。
