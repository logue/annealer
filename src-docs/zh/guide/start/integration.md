---
description: 将 annealer 与 Rslint 和 rs fmt（Prettier）、eslint-plugin-vue、oxlint 和 oxfmt 或 Biome 一起使用，并提供避免工具之间互相撤销输出的配置示例。
---

# 与其他工具配合使用

annealer 只负责排序：属性、类和声明。行宽和缩进交给你的格式化工具。本页介绍如何将它与常用工具链组合，使两者不会互相撤销对方的输出。

以下示例已使用这些版本验证：

- [Prettier](https://prettier.io/) 3.9（通过 `rs fmt`）
- [oxfmt](https://oxc.rs/docs/guide/usage/formatter.html) 0.74
- [oxlint](https://oxc.rs/docs/guide/usage/linter.html) 1.89
- [eslint-plugin-vue](https://eslint.vuejs.org/) 10.11，搭配 [eslint-config-prettier](https://npm.im/eslint-config-prettier) 10.1
- [Biome](https://biomejs.dev/) 2.6

## 运行顺序 {#run-order}

先运行 annealer，再运行格式化工具：

```bash
anyl --write src/App.vue   # 1. 排序
rs fmt src/App.vue         # 2. 格式化（Prettier、oxfmt、Biome 等）
```

- 这些格式化工具都不会对属性或声明排序，因此格式化工具会保留 annealer 的顺序。
- annealer 使用 malva 格式化样式表，而 malva 对某些值的布局方式与 Prettier、oxfmt 和 Biome 不同（参见[限制](./limitations.md#stylesheets)）。当格式化工具最后运行时，以它的布局为准，结果是稳定的：再次运行两者不会产生任何变化。
- 由于这些布局差异，单独运行 `anyl --check` 可能会在格式化工具刚写出的文件上失败。要在 CI 中检查顺序，请运行整个流程并检查是否有变化：

```bash
git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write
rs fmt                  # 或：oxfmt / biome format --write .
git diff --exit-code    # 顺序或格式不正确时失败
```

`anyl` 本身不会遍历目录或展开通配符，因此请像上面的 `git ls-files` 那样传入文件路径。

## 需要对齐的设置 {#settings-to-align}

annealer 的内置配置文件使用以下值。让格式化工具与之对齐，可以避免 `<style>` 块和样式表在两种风格之间来回切换，并让格式化工具产生的 diff 保持最小：

| 项目            | annealer（内置配置文件）               | Prettier / `rs fmt`                  | oxfmt                    | Biome                                            |
| --------------- | -------------------------------------- | ------------------------------------ | ------------------------ | ------------------------------------------------ |
| CSS 字符串引号  | 单引号（`quotes: prefer-single`）      | `singleQuote: true`                  | `singleQuote: true`      | `css.formatter.quoteStyle: "single"`             |
| 样式表缩进      | 2 个空格（malva 默认）                 | 默认                                 | 默认                     | `formatter.indentStyle: "space"`                 |
| 样式表行宽      | 80；可用 `--print-width` 修改          | 默认（80）                           | `--print-width 100`      | 默认（80）                                       |
| 空元素          | `<br />`（always）                     | 默认                                 | 默认                     | `html.formatter.selfCloseVoidElements: "always"` |
| 属性排序        | annealer                               | 无                                   | 无                       | 关闭 `useSortedAttributes`（必需）               |
| 声明排序        | annealer（`declarationOrder: smacss`） | 无                                   | 无                       | 关闭 `useSortedProperties`（必需）               |
| Tailwind 类排序 | `--class-order tailwind`（可选）       | 不使用 `prettier-plugin-tailwindcss` | 不使用 `sortTailwindcss` | 关闭 `useTailwindSortedClasses`                  |

如果想改用双引号，请修改配置文件而不是格式化工具：复制内置配置文件（`crates/annealer/profiles/*.yaml`）并设置 `stylesheet.malva.quotes: prefer-double`。

### 不要按字母顺序排序 {#no-alphabetical-sorting}

annealer 的存在意义在于以有含义的顺序排列属性和声明：先是标识，然后是元素是什么、做什么，接着是 ARIA、数据和事件。字母顺序会丢弃这些信息，因此任何按字母顺序对属性或声明排序的工具都会与 annealer 冲突，二者会不断改写对方的输出。请关闭以下设置：

| 工具              | 设置                                                                                |
| ----------------- | ----------------------------------------------------------------------------------- |
| eslint-plugin-vue | `vue/attributes-order`：保持 `alphabetical: false`（默认值）                        |
| Biome             | `assist.actions.source.useSortedAttributes` 和 `useSortedProperties`：`off`         |
| Stylelint         | 不要对 annealer 处理的文件使用 `stylelint-order` 的 `properties-alphabetical-order` |

### Tailwind 类：只选一个排序工具 {#tailwind-classes-pick-one-sorter}

annealer 的 `tailwind` 类顺序遵循 Tailwind 自身的顺序，与 `prettier-plugin-tailwindcss` 和 oxfmt 的 `sortTailwindcss` 相同。顺序一致，但是：

- 这些格式化工具默认会删除重复的类。除非传入 `--remove-duplicate-classes`，否则 annealer 会保留它们。
- 它们还可以对 `clsx()`/`cn()` 调用以及你指定的属性（`:class`）中的类排序。annealer 只对静态 `class` 属性排序。

请只使用其中一个。如果你更喜欢格式化工具的排序，请去掉 `--class-order`；对于 oxfmt，如果希望在重复项上与 annealer 行为一致，请设置 `preserveDuplicates: true`。

## Rslint 与 `rs fmt`（Rstack） {#rslint-and-rs-fmt-rstack}

Rslint 对 JavaScript 和 TypeScript 进行检查，没有 Vue 模板规则，因此与 annealer 没有任何重叠。`rs fmt` 基于 Prettier，所以 annealer 在它之前运行：

```ts title="rstack.config.ts"
import { define } from 'rstack';

define.lint(({ js, ts }) => [
  { ignores: ['dist/**'] },
  js.configs.recommended,
  ts.configs.recommendedTypeChecked
]);

define.fmt({
  // annealer 输出单引号的 CSS 字符串。
  singleQuote: true
  // 使用 `anyl --class-order tailwind` 时，不要添加 `prettier-plugin-tailwindcss`。
});

define.staged({
  // 先运行 annealer，再运行格式化工具。
  '*.{vue,html,css,scss,less}': ['anyl --write', 'rs fmt'],
  '*.sass': 'anyl --write',
  '*.{js,jsx,ts,tsx,mjs,cjs,mts,cts}': ['rs lint --fix', 'rs fmt']
});
```

```json title="package.json"
{
  "scripts": {
    "format": "git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write && rs fmt"
  }
}
```

staged 的通配符只应包含 annealer 支持的扩展名。其他任何文件都会报错（`unsupported file type`）。

## eslint-plugin-vue {#eslint-plugin-vue}

`vue/attributes-order`（包含在 `flat/recommended` 中）检查的层级与 annealer 的 `vue` 配置文件相同，因此 annealer 的输出能通过该规则。请将 `alphabetical` 保持为默认值 `false`（参见[不要按字母顺序排序](#no-alphabetical-sorting)）。使用 Prettier 时，请最后添加 `eslint-config-prettier`：它会关闭布局类规则（`vue/html-self-closing`、`vue/max-attributes-per-line` 等），由 Prettier 决定布局。

```js title="eslint.config.js"
import vue from 'eslint-plugin-vue';
import prettier from 'eslint-config-prettier';

export default [
  ...vue.configs['flat/recommended'],
  prettier,
  {
    rules: {
      // annealer 按语义对 "other attributes" 层级排序。
      'vue/attributes-order': ['warn', { alphabetical: false }]
    }
  }
];
```

- **展开：** annealer 从不会让属性跨越 `v-bind="object"` 移动，因为在 Vue 3 中后出现的值优先。`vue/attributes-order` 仍会检查跨越展开的顺序，其 `--fix` 也会让属性跨越展开移动（`title="t" v-bind="obj"` 变为 `v-bind="obj" title="t"`，于是 `title` 会覆盖 `obj.title`）。对于这样的标签，请手动排序属性，或添加 `eslint-disable` 注释。
- **annealer 不改写的规则：** `vue/v-on-event-hyphenation`（`@myEvent` → `@my-event`）和 `vue/v-slot-style`（`v-slot:item` → `#item`）。`eslint --fix` 会改写它们，annealer 不会改动其结果。

依次运行 annealer、Prettier 和 `eslint --fix`。对其结果运行 `anyl --check` 会通过。

## oxlint 与 oxfmt（Oxc） {#oxlint-and-oxfmt-oxc}

oxlint 的 `vue` 插件只检查 `<script>` 代码；它没有 `vue/attributes-order` 这样的模板规则。无需关闭任何规则：

```json title=".oxlintrc.json"
{
  "$schema": "./node_modules/oxlint/configuration_schema.json",
  "plugins": ["typescript", "unicorn", "oxc", "vue"],
  "categories": { "correctness": "error" },
  "ignorePatterns": ["dist/**"]
}
```

oxfmt 使用内置的 Prettier 格式化 `.vue` 和 `.html`，并原生格式化 CSS/SCSS/Less。它的默认值在引号（双引号）和行宽（100 而非 80）上与 annealer 不同。请在 oxfmt 中设置引号：

```json title=".oxfmtrc.json"
{
  "$schema": "./node_modules/oxfmt/configuration_schema.json",
  "singleQuote": true
}
```

至于行宽，请修改 annealer 而不是 oxfmt：传入 `--print-width 100`（JavaScript：`printWidth: 100`），让样式表和 `<style>` 块在与 oxfmt 相同的位置换行。如果你在 `.oxfmtrc.json` 中设置了 `printWidth`，请将相同的值传给 `--print-width`。当 annealer 对类排序时，请保持 `sortTailwindcss` 关闭；或者开启它并去掉 `--class-order`：

```json title=".oxfmtrc.json（由 oxfmt 对 Tailwind 类排序）"
{
  "singleQuote": true,
  "sortTailwindcss": {
    "stylesheet": "./src/app.css",
    "preserveDuplicates": true
  }
}
```

然后在 oxfmt 之前运行 annealer：

```bash
git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write --print-width 100
oxfmt
```

## Biome {#biome}

Biome 的默认值在四个方面与 annealer 冲突：制表符、双引号的 CSS 字符串、不带斜杠的 `<br>`，以及它自己的排序操作。在开启 HTML 完整支持（`.vue` 文件需要）的情况下，请使用：

```json title="biome.json"
{
  "$schema": "./node_modules/@biomejs/biome/configuration_schema.json",
  "formatter": { "indentStyle": "space", "indentWidth": 2 },
  "css": { "formatter": { "quoteStyle": "single" } },
  "html": {
    "experimentalFullSupportEnabled": true,
    "formatter": { "selfCloseVoidElements": "always" }
  },
  "assist": {
    "actions": {
      "source": {
        "useSortedAttributes": "off",
        "useSortedProperties": "off"
      }
    }
  },
  "linter": {
    "rules": { "nursery": { "useTailwindSortedClasses": "off" } }
  }
}
```

- **关闭 `useSortedAttributes` 和 `useSortedProperties`。** `useSortedAttributes` 会按字母顺序对 HTML 和 Vue 属性排序，`useSortedProperties` 会按其自身的顺序对 CSS 声明排序。两者都与 annealer 按语义的排序冲突，并会在每次 `biome check --write` 时撤销它（参见[不要按字母顺序排序](#no-alphabetical-sorting)）。它们默认是关闭的；请像上面那样显式设置为 `off`，以免共享配置将其开启。
- `useTailwindSortedClasses` 是 nursery 规则。当 annealer 对类排序时，请将其关闭。
- 如果你保留制表符，请在配置文件中设置 `stylesheet.malva.useTabs: true`，而不是修改 `indentStyle`。

然后在 Biome 之前运行 annealer：

```bash
git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write
biome check --write .
```
