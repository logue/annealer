---
description: annealer 的架构，以及实现过程中做出的设计决策及其理由。
---

# 设计决策

## 架构 {#architecture}

```text
annealer (core)  纯函数：format(input: &str, config: &Config)
                   -> Result<String, FormatError>
                 不做文件 I/O。可构建为 wasm32-unknown-unknown。
                 使用 malva 处理 CSS/SCSS/Sass/Less 的声明顺序。

anyl (CLI)       负责文件 I/O 和参数解析；调用核心。

annealer (npm)   核心的 WebAssembly 构建，外加 TypeScript 封装。
```

- **单个 crate。** 目前核心和 CLI 位于同一个 crate（`crates/annealer`）中。只有在真正需要时才会拆分为 workspace，以避免过早抽象。CLI 依赖的 `clap` 位于默认的 `cli` feature 之后，因此使用 `--no-default-features` 即可将库构建为 WebAssembly。
- **配置文件是 YAML，而不是代码。** HTML、Vue 以及未来的配置文件都是数据。每个配置文件都有 `schemaVersion`，因为该格式与 Rust 函数签名一样，都是公共 API。
- **严格校验。** 解析配置文件时会拒绝未知键，因此拼写错误会立即暴露。另提供 JSON Schema（draft-07）以支持编辑器。draft-07 无法表达的两条规则（恰好一个 fallback 分组、分组名唯一）仅由加载器检查。
- **malva 采用宽松的版本约束**（`0.16`），与其他项目中使用的库依赖策略一致。
- **npm 包内嵌 WebAssembly 二进制**（base64），而不是单独提供 `.wasm` 文件。代价是压缩前体积增加约 33%；换来的是 `format()` 是同步的，并且无需配置打包工具或加载器。

## 实现过程中的决策 {#decisions-made-during-implementation}

1. **HTML 配置文件的顺序。** 最初的表格没有为未匹配的属性以及 `alt`/`title` 留出位置。最终商定的顺序为 id → class → 补充（`alt`、`title`、`placeholder`）→ 语义（`type`、`href`、`src`、`srcset`）→ 键值（`name`、`value`、`content`）→ 尺寸（`width`、`height`）→ 其他（状态：`disabled`、`checked` 等）→ ARIA → data → 事件。Vue 配置文件在其其他属性层级内部使用相同的顺序。
2. **"静态先于绑定"仅适用于 `class`。** 对于其他名称，聚合簇保持源顺序，因为 Vue 合并 `style` 时后出现的声明优先，而对于其他重复项只保留一个值。
3. **`spreadSafe` 分组。** 编译期指令从不与 `v-bind` 展开对象合并，因此可以跨越展开移动。其他所有属性都保持在各自的片段内。
4. **多行标签中每行一个属性。** 否则重新排序可能会让两个属性挤在同一行。这与 `vue/max-attributes-per-line` 的多行默认值一致，可以通过 `layout.oneAttributePerLine` 关闭。
5. **Vue 第 8 层被拆分**为先其他指令、后其他属性，与 `vue/attributes-order`（`OTHER_DIRECTIVES` → `OTHER_ATTR`）一致。
6. **`void: always` 与 `vue/html-self-closing` 的默认值**（`html.void: never`）不同。保留该 ESLint 规则的项目需要让其中一方对齐。
7. **HTML 配置文件中同样定义了 `contentNewline`。** 块级内容开头和结尾的空白不会被渲染，而 `ignore` 列表涵盖了可能会被渲染的元素。
8. **带厂商前缀的声明排在最前**（`vendorPrefix: start`），这样标准属性最后声明并生效。参见[样式表](../concepts/stylesheets.md#vendor-prefixes-vendorprefix-start)。
9. **内容为空的元素和组件的自闭合以及 `contentNewline` 为可选功能。** 它们对标记的改写超出了排序所需，并且与不使用这些 eslint-plugin-vue 规则的配置（例如 Prettier）冲突。内置配置文件保留了这些设置，但设为 `enabled: false`；`--self-closing` / `--content-newline` 或对应的 JavaScript 选项可将其开启。空元素（`<br />`）始终自闭合，Prettier 也是这样输出的。

## 已解决的问题 {#resolved-questions}

- **`v-model` 与普通的 `value`**：从不聚合。键 `v-model` 与 `value` 不同。
- **配置文件的严格程度**：在解析时拒绝未知键。
- **malva 版本**：宽松范围 `0.16`。

## 待办事项 {#open-items}

- 待 malva 升级后，将 raffia 升级到 0.13（malva 0.16 仍使用 raffia 0.12）。
- Svelte 配置文件。
- 静态 `class` 属性的[值排序](./value-ordering.md)（Tailwind 近似顺序、Bootstrap 提案）。
- [库配置文件](./library-profiles.md)、配置文件组合（`extends`）以及按标签或按组件的覆盖。
- 除手写测试用例外，增加取自真实项目的测试用例。
