> For AI agents: the complete documentation index is available at /annealer/zh/llms.txt, the full documentation bundle is available at /annealer/zh/llms-full.txt.

# 样式表

annealer 不自行定义 CSS 属性顺序，而是使用 [malva](https://github.com/g-plane/malva) 的 `declarationOrder` 顺序。配置文件的 `stylesheet` 部分负责控制此功能。

**只改变顺序。** 声明会按原样移动。空白、缩进、引号、颜色大小写、数字、注释和空行都保持不变：格式化工作交给 Prettier 或 Oxfmt 等格式化工具。malva 本身是格式化工具，但 annealer 只用它确定属性名称的排序方式，不使用它输出的样式表。

支持以下输入：

- HTML 和 Vue 中的 `<style>` 块，包括 `<style lang="scss">`、`lang="less"` 和 `lang="sass"`。
- 独立的 `.css`、`.scss`、`.sass`（缩进语法）和 `.less` 文件。

如果配置文件没有 `stylesheet` 部分，样式表将保持不变。

## 声明顺序 \{#declaration-order}

两个内置配置文件都使用：

```yaml
stylesheet:
  declarationOrder: smacss
  vendorPrefix: start
```

- `declarationOrder`：`smacss`（重要的、影响布局流的属性在前）、`concentric`（从盒模型外侧向内）或 `alphabetical`。
- `declarationOrderGroupBy`：如何将声明划分为独立排序的连续段：`non-declaration-and-empty-line`（默认；空行也会结束一段）或 `non-declaration`（跨空行排序，但保留空行位置）。

声明不会跨越其他语句（嵌套规则、`@include`、Less mixin 调用等）或注释移动，因此注释及其标注的分组会保持完整。被[禁用指令](/annealer/zh/guide/features/disable-directives.md)覆盖的声明以及包含嵌套块的声明（Sass 嵌套属性）也会留在原处，并结束当前连续段。

```css
/* 输入 */
body {
  color: #222;
  margin: 0;
  font-family: system-ui, sans-serif;
  display: flex;
}

/* 输出（smacss） */
body {
  display: flex;
  margin: 0;
  color: #222;
  font-family: system-ui, sans-serif;
}
```

## 厂商前缀：`vendorPrefix: start` \{#vendor-prefixes-vendorprefix-start}

带厂商前缀的声明（`-webkit-…`、`-moz-…`）与标准属性分开处理。它们是方言，其中许多从未被标准化。使用 `start` 时，这些声明会按简单的字母顺序放在同一连续段的其他声明**之前**，不按属性分组。

为什么是 `start` 而不是 `end`：带前缀的属性通常是标准属性的别名，而后出现的声明优先。把前缀放在最后可能会让方言覆盖标准值（例如带两个值的 `-webkit-border-radius`）。`start` 让标准属性保持在最后，这与 Autoprefixer 的输出一致。

malva 没有这样的选项（它按去掉前缀后的名称排序），因此 annealer 会在相同的连续段中自行移动带前缀的声明。仅在设置了 `declarationOrder` 时该选项才生效。`alongside`（未设置时的默认值）会保留 malva 的顺序。

## 静态 `style` 属性（可选） \{#static-style-attributes-opt-in}

`stylesheet.styleAttribute: true` 还会对静态 `style` 属性中的声明排序。两个内置配置文件中都为关闭。

- 声明按原样移动；空格和缺少的末尾 `;` 都保持不变。
- 绑定的 `:style` 从不处理。
- 当值未加引号、包含模板语法、`{}`、`<` 或 `&`，或无法解析为普通声明时，保持原样。
