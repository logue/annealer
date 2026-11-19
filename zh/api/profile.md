> For AI agents: the complete documentation index is available at /annealer/zh/llms.txt, the full documentation bundle is available at /annealer/zh/llms-full.txt.

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
    match: [on*]

stylesheet:
  declarationOrder: smacss
  vendorPrefix: start
```

:::tip 编辑器支持
JSON Schema（draft-07）位于 `crates/annealer/schema/profile.schema.json`。配合上面的 modeline 和 [YAML 扩展](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-yaml)，VS Code 可以校验并补全配置文件。
:::

配置文件中任何位置的未知键都会报错，因此拼写错误不会被忽略。

## 顶层 \{#top-level}

| 键               | 类型                  | 必需 | 说明                                  |
| --------------- | ------------------- | -- | ----------------------------------- |
| `schemaVersion` | `1`                 | 是  | 配置文件格式版本。                           |
| `name`          | string              | 是  | 配置文件名称。                             |
| `description`   | string              |    | 供人阅读的简介。                            |
| `extends`       | string \| string\[] |    | 作为基础的配置文件。参见 [`extends`](#extends)。 |
| `normalize`     | object              |    | 属性名改写。                              |
| `layout`        | object              |    | 起始标签的布局规则。                          |
| `groups`        | array               | 是¹ | 按输出顺序排列的属性分组。                       |
| `stylesheet`    | object              |    | 样式表格式化。省略时样式表保持不变。                  |
| `classOrder`    | string \| object    |    | 静态 `class` 属性的类顺序。省略时类保持原有顺序。       |

¹ 使用 `extends` 时可省略；合并后的配置文件必须具备。

## `extends` \{#extends}

以其他配置文件为基础：内置配置文件名（`html`、`vue`）、相对于继承方文件的路径（仅 CLI），或它们的列表。

```yaml
schemaVersion: 1
name: my-html
extends: [html, ./bootstrap-data.yaml]
```

```yaml
# bootstrap-data.yaml：作为 mixin 使用的部分配置文件
schemaVersion: 1
name: bootstrap-data
groups:
  - name: bootstrap
    match: [data-bs-toggle, data-bs-target, data-bs-*]
    sort: listed
    before: data
```

配置文件按顺序合并，因此**后写的优先**：`extends` 的每一项覆盖其之前的项，配置文件自身覆盖所有项。

- **映射**（`normalize`、`layout`、`stylesheet` 等）按键合并。
- **其他值**（标量、列表、`classOrder`）被替换。
- **`groups` 按 `name` 合并。** 与继承分组同名的分组按键更新该分组；其 `match` 列表被替换而非追加，位置保持不变。新分组追加到末尾。
- **`before` / `after`** 指定要把分组（新分组或继承的分组）放在其旁边的分组名。可以指定此前合并的任何配置文件中的分组，因此 mixin 可以把自己的分组放到前面配置文件的分组之间。

继承链会先被展平：自身带有 `extends` 的基础配置文件会把它的基础展开在自己之前。校验只在合并结果上进行一次，因此基础配置文件可以是部分的（没有 `groups`、没有 fallback 分组）。直接或间接继承自身会报错。

JavaScript API 没有文件系统，因此 `extends` 只能指定内置配置文件。

## `normalize`

在分类之前应用的改写。

| 键                         | 类型      | 默认      | 说明                                                    |
| ------------------------- | ------- | ------- | ----------------------------------------------------- |
| `directiveShorthand`      | boolean | `false` | `v-bind:foo` → `:foo`，`v-on:click` → `@click`。        |
| `hyphenateComponentProps` | boolean | `false` | 组件标签上的 camelCase 属性名 → kebab-case。原生 SVG/MathML 元素除外。 |

## `layout`

详情参见[布局](/annealer/zh/guide/features/layout.md)。

| 键                                | 类型                                | 默认                    | 说明                                     |
| -------------------------------- | --------------------------------- | --------------------- | -------------------------------------- |
| `closingBracketNewline`          | boolean                           | `false`               | 将多行起始标签的结束括号放在单独一行。                    |
| `oneAttributePerLine`            | boolean                           | `false`               | 多行起始标签中每行一个属性。                         |
| `selfClosing.void`               | `always` \| `never` \| `preserve` | `preserve`            | 空元素：`<br />` 或 `<br>`。                 |
| `selfClosing.normal`             | `always` \| `never` \| `preserve` | `preserve`            | 内容为空的非空元素（`<div />`）。仅限 Vue 模板。        |
| `selfClosing.component`          | `always` \| `never` \| `preserve` | `preserve`            | 内容为空的组件（`<MyComp />`）。仅限 Vue 模板。       |
| `contentNewline`                 | object                            | 关闭                    | 让多行元素的内容独占一行。`{}` 表示以默认值开启。            |
| `contentNewline.ignore`          | string\[]                         | `pre`、`textarea`、行内元素 | 不处理的元素（连同其内部全部内容）。元素名不区分大小写匹配；组件名精确匹配。 |
| `contentNewline.allowEmptyLines` | boolean                           | `false`               | 保留内容前后的空行。                             |

## `groups`

分组按输出顺序列出。必须恰好有一个分组是 fallback，且分组名必须唯一。

| 键            | 类型                                     | 默认       | 说明                                             |
| ------------ | -------------------------------------- | -------- | ---------------------------------------------- |
| `name`       | string                                 | –        | 唯一的分组名。必需。                                     |
| `match`      | string\[]                              | –        | 模式：精确名称，或以 `*` 结尾的前缀。除非 `fallback: true`，否则必需。 |
| `sort`       | `source` \| `alphabetical` \| `listed` | `source` | 分组内的顺序：按书写顺序、按键名，或按 `match` 中的顺序。              |
| `fallback`   | boolean                                | `false`  | 接收所有未匹配任何模式的属性。                                |
| `spreadSafe` | boolean                                | `false`  | 此分组中的属性可以跨越 `v-bind="object"` 展开移动。仅用于编译期指令。   |
| `before`     | string                                 | –        | 将分组放在该分组之前。参见 [`extends`](#extends)。           |
| `after`      | string                                 | –        | 将分组放在该分组之后。参见 [`extends`](#extends)。           |

模式会与**规范化并转为小写后的键**匹配：`foo` 对应 `foo`/`:foo`/`v-bind:foo`，`@click` 对应 `v-on:click`，`#name` 对应 `v-slot:name`，修饰符会被去除。精确名称优先于任何前缀，较长的前缀优先于较短的前缀。参见[属性顺序](/annealer/zh/guide/concepts/attribute-order.md)。

## `stylesheet`

详情参见[样式表](/annealer/zh/guide/concepts/stylesheets.md)。

| 键                         | 类型                                                    | 默认                               | 说明                                                               |
| ------------------------- | ----------------------------------------------------- | -------------------------------- | ---------------------------------------------------------------- |
| `declarationOrder`        | `smacss` \| `concentric` \| `alphabetical`            | –                                | 属性顺序。未设置时不对声明排序。                                                 |
| `declarationOrderGroupBy` | `non-declaration` \| `non-declaration-and-empty-line` | `non-declaration-and-empty-line` | 如何将声明划分为分别排序的连续段。默认情况下，空行会结束一段；`non-declaration` 会跨空行排序，但保留空行位置。 |
| `vendorPrefix`            | `alongside` \| `start`                                | `alongside`                      | `start` 会将带厂商前缀的声明按字母顺序放在每段的开头。                                  |
| `styleAttribute`          | boolean                                               | `false`                          | 同时对静态 `style` 属性中的声明排序。                                          |

只会改变声明顺序；空白、引号和其他格式交由格式化工具处理。因此不接受 malva 的格式化选项。

## `classOrder` \{#classorder}

静态 `class` 属性中类的顺序（参见[类顺序](/annealer/zh/guide/features/class-order.md)）：内置类顺序名（`tailwind`、`bootstrap`），或内联书写的[类顺序配置文件](#class-order-profiles)。CLI 的 `--class-order` 和 JavaScript 的 `classOrder` 选项可覆盖它。

```yaml
classOrder: tailwind
```

## 类顺序配置文件 \{#class-order-profiles}

类顺序配置文件是一个独立的 YAML 文件，通过 `--class-order` 或 `classOrder` 传入，也可以内联写在配置文件的 `classOrder` 下。其 JSON Schema 为 `crates/annealer/schema/class-order.schema.json`；内置的类顺序为 [`crates/annealer/class-orders/tailwind.yaml`](https://github.com/logue/annealer/tree/main/crates/annealer/class-orders)。

| 键                     | 类型                  | 默认      | 说明                                                                      |
| --------------------- | ------------------- | ------- | ----------------------------------------------------------------------- |
| `schemaVersion`       | `1`                 | –       | 必需。格式版本。                                                                |
| `name`                | string              | –       | 必需。类顺序名称。                                                               |
| `description`         | string              | –       | 供人阅读的简介。                                                                |
| `unknown`             | `start` \| `end`    | `start` | 未匹配任何规则的类放在哪里（保持源顺序）。                                                   |
| `removeDuplicates`    | boolean             | `false` | 删除重复的类，保留第一个。                                                           |
| `important`           | string              | –       | 匹配前从工具类开头或结尾去除的标记（`!`）。                                                 |
| `arbitraryProperties` | boolean             | `false` | `[property:value]` 形式的类设置 `property`。                                   |
| `variants`            | object              | –       | `separator`（如 `:`）和 `order`：按等级排列的变体模式，等级低的在前。一个列表项中的模式具有相同等级。          |
| `propertyOrder`       | string\[]           | –       | 必需。按输出顺序排列的 CSS 属性。                                                     |
| `values`              | map of pattern list | –       | 模式中 `{name}` 的取值集合。`number` 为内置。                                        |
| `rules`               | array               | –       | 必需。`match`（模式）、`properties`（取自 `propertyOrder`）以及可选的 `count`（声明数量，若不同）。 |

带有未匹配任何模式的变体的类视为未知类。规则只能引用 `propertyOrder` 中列出的属性，否则会报错。
