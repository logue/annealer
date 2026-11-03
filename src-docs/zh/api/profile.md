---
description: annealer 配置文件 YAML 格式参考，列出每个键及其类型和默认值。
---

# 配置文件参考

配置文件是一个 YAML 文件，定义属性顺序、布局规则和样式表选项。内置配置文件位于 [`crates/annealer/profiles/`](https://github.com/logue/annealer/tree/main/crates/annealer/profiles)。

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

:::tip 编辑器支持
JSON Schema（draft-07）位于 `crates/annealer/schema/profile.schema.json`。配合上面的 modeline 和 [YAML 扩展](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-yaml)，VS Code 可以校验并补全配置文件。
:::

配置文件中任何位置的未知键都会报错，因此拼写错误不会被忽略。

## 顶层 {#top-level}

| 键              | 类型             | 必需 | 说明                                              |
| --------------- | ---------------- | ---- | ------------------------------------------------- |
| `schemaVersion` | `1`              | 是   | 配置文件格式版本。                                |
| `name`          | string           | 是   | 配置文件名称。                                    |
| `description`   | string           |      | 供人阅读的简介。                                  |
| `normalize`     | object           |      | 属性名改写。                                      |
| `layout`        | object           |      | 起始标签的布局规则。                              |
| `groups`        | array            | 是   | 按输出顺序排列的属性分组。                        |
| `stylesheet`    | object           |      | 样式表格式化。省略时样式表保持不变。              |
| `classOrder`    | string \| object |      | 静态 `class` 属性的类顺序。省略时类保持原有顺序。 |

## `normalize`

在分类之前应用的改写。

| 键                        | 类型    | 默认    | 说明                                                                   |
| ------------------------- | ------- | ------- | ---------------------------------------------------------------------- |
| `directiveShorthand`      | boolean | `false` | `v-bind:foo` → `:foo`，`v-on:click` → `@click`。                       |
| `hyphenateComponentProps` | boolean | `false` | 组件标签上的 camelCase 属性名 → kebab-case。原生 SVG/MathML 元素除外。 |

## `layout`

详情参见[布局](../guide/features/layout.md)。

| 键                               | 类型                              | 默认                        | 说明                                                                                                                                                      |
| -------------------------------- | --------------------------------- | --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `closingBracketNewline`          | boolean                           | `false`                     | 将多行起始标签的结束括号放在单独一行。                                                                                                                    |
| `oneAttributePerLine`            | boolean                           | `false`                     | 多行起始标签中每行一个属性。                                                                                                                              |
| `selfClosing.enabled`            | boolean                           | `true`                      | 是否应用 `normal` 和 `component`（`void` 和 `expandNonVoid` 始终应用）。内置 `vue` 配置文件设为 `false`；`--self-closing` 和 `selfClosing` 选项可覆盖它。 |
| `selfClosing.void`               | `always` \| `never` \| `preserve` | `preserve`                  | 空元素：`<br />` 或 `<br>`。`--void-self-closing` 和 `voidSelfClosing` 选项可覆盖它。                                                                     |
| `selfClosing.normal`             | `always` \| `never` \| `preserve` | `preserve`                  | 内容为空的非空元素（`<div />`）。仅限 Vue 模板。                                                                                                          |
| `selfClosing.component`          | `always` \| `never` \| `preserve` | `preserve`                  | 内容为空的组件（`<MyComp />`）。仅限 Vue 模板。                                                                                                           |
| `selfClosing.expandNonVoid`      | boolean                           | `false`                     | `<div />` → `<div></div>`。仅限纯 HTML；`<svg>` 和 `<math>` 的内容不受影响。`--expand-non-void` 和 `expandNonVoid` 选项可覆盖它。                         |
| `contentNewline`                 | object                            | 关闭                        | 让多行元素的内容独占一行。`{}` 表示以默认值开启。                                                                                                         |
| `contentNewline.enabled`         | boolean                           | `true`                      | 是否应用该规则。内置配置文件设为 `false`；`--content-newline` 和 `contentNewline` 选项可覆盖它。                                                          |
| `contentNewline.ignore`          | string[]                          | `pre`、`textarea`、行内元素 | 不处理的元素（连同其内部全部内容）。元素名不区分大小写匹配；组件名精确匹配。                                                                              |
| `contentNewline.allowEmptyLines` | boolean                           | `false`                     | 保留内容前后的空行。                                                                                                                                      |

## `groups`

分组按输出顺序列出。必须恰好有一个分组是 fallback，且分组名必须唯一。

| 键           | 类型                                   | 默认     | 说明                                                                   |
| ------------ | -------------------------------------- | -------- | ---------------------------------------------------------------------- |
| `name`       | string                                 | –        | 唯一的分组名。必需。                                                   |
| `match`      | string[]                               | –        | 模式：精确名称，或以 `*` 结尾的前缀。除非 `fallback: true`，否则必需。 |
| `sort`       | `source` \| `alphabetical` \| `listed` | `source` | 分组内的顺序：按书写顺序、按键名，或按 `match` 中的顺序。              |
| `fallback`   | boolean                                | `false`  | 接收所有未匹配任何模式的属性。                                         |
| `spreadSafe` | boolean                                | `false`  | 此分组中的属性可以跨越 `v-bind="object"` 展开移动。仅用于编译期指令。  |

模式会与**规范化并转为小写后的键**匹配：`foo`/`:foo`/`v-bind:foo` 对应 `foo`，`@click` 对应 `v-on:click`，`#name` 对应 `v-slot:name`，修饰符会被去除。精确名称优先于任何前缀，较长的前缀优先于较短的前缀。参见[属性顺序](../guide/concepts/attribute-order.md)。

## `stylesheet`

详情参见[样式表](../guide/concepts/stylesheets.md)。

| 键                        | 类型                                                  | 默认              | 说明                                                             |
| ------------------------- | ----------------------------------------------------- | ----------------- | ---------------------------------------------------------------- |
| `declarationOrder`        | `smacss` \| `concentric` \| `alphabetical`            | –                 | 属性顺序。未设置时不对声明排序。                                 |
| `declarationOrderGroupBy` | `non-declaration` \| `non-declaration-and-empty-line` | `non-declaration` | 如何将声明划分为分别排序的连续段。                               |
| `vendorPrefix`            | `alongside` \| `start`                                | `alongside`       | `start` 会将带厂商前缀的声明按字母顺序放在每段的开头。           |
| `styleAttribute`          | boolean                                               | `false`           | 同时对静态 `style` 属性中的声明排序。                            |
| `malva`                   | object                                                | –                 | 其他 [malva 选项](https://malva.netlify.app/config/)，原样传递。 |

`stylesheet.malva` 接受 malva 自身的选项，例如 `printWidth`、`useTabs`、`indentWidth`、`lineBreak`、`hexCase`、`hexColorLength`、`quotes`、`trailingComma`、`formatComments` 以及各个 `*.preferSingleLine` 选项。`declarationOrder` 和 `declarationOrderGroupBy` 属于 `stylesheet` 本身，而不属于 `malva`。未知的 malva 键会被拒绝。在 annealer 中，`quotes` 默认为 `prefer-single`（malva 自身的默认值是 `always-double`）；参见[引号](../guide/concepts/stylesheets.md#quotes)。`--print-width`（JavaScript：`printWidth`）可以在不修改配置文件的情况下覆盖 `printWidth`。

## `classOrder`

静态 `class` 属性中类的顺序（参见[类顺序](../guide/features/class-order.md)）：内置类顺序名（`tailwind`），或内联书写的[类顺序配置文件](#class-order-profiles)。CLI 的 `--class-order` 和 JavaScript 的 `classOrder` 选项可覆盖它。

```yaml
classOrder: tailwind
```

## 类顺序配置文件 {#class-order-profiles}

类顺序配置文件是一个独立的 YAML 文件，通过 `--class-order` 或 `classOrder` 传入，也可以内联写在配置文件的 `classOrder` 下。其 JSON Schema 为 `crates/annealer/schema/class-order.schema.json`；内置的类顺序为 [`crates/annealer/class-orders/tailwind.yaml`](https://github.com/logue/annealer/tree/main/crates/annealer/class-orders)。

| 键                    | 类型                | 默认    | 说明                                                                                                  |
| --------------------- | ------------------- | ------- | ----------------------------------------------------------------------------------------------------- |
| `schemaVersion`       | `1`                 | –       | 必需。格式版本。                                                                                      |
| `name`                | string              | –       | 必需。类顺序名称。                                                                                    |
| `description`         | string              | –       | 供人阅读的简介。                                                                                      |
| `unknown`             | `start` \| `end`    | `start` | 未匹配任何规则的类放在哪里（保持源顺序）。                                                            |
| `removeDuplicates`    | boolean             | `false` | 删除重复的类，保留第一个。`--remove-duplicate-classes` 可覆盖它。                                     |
| `important`           | string              | –       | 匹配前从工具类开头或结尾去除的标记（`!`）。                                                           |
| `arbitraryProperties` | boolean             | `false` | `[property:value]` 形式的类设置 `property`。                                                          |
| `variants`            | object              | –       | `separator`（如 `:`）和 `order`：按等级排列的变体模式，等级低的在前。一个列表项中的模式具有相同等级。 |
| `propertyOrder`       | string[]            | –       | 必需。按输出顺序排列的 CSS 属性。                                                                     |
| `values`              | map of pattern list | –       | 模式中 `{name}` 的取值集合。`number` 为内置。                                                         |
| `rules`               | array               | –       | 必需。`match`（模式）、`properties`（取自 `propertyOrder`）以及可选的 `count`（声明数量，若不同）。   |

带有未匹配任何模式的变体的类视为未知类。规则只能引用 `propertyOrder` 中列出的属性，否则会报错。
