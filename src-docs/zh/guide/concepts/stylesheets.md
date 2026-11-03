---
description: annealer 如何通过 malva 对 CSS、SCSS、Sass 和 Less 中的声明排序、带厂商前缀的声明放在哪里，以及可选的 style 属性支持。
---

# 样式表

annealer 本身并不实现 CSS 属性排序，而是委托给 [malva](https://github.com/g-plane/malva)，后者已经提供了可配置的 `declarationOrder`。这由配置文件的 `stylesheet` 部分控制。

会被格式化的输入：

- HTML 和 Vue 中的 `<style>` 块，包括 `<style lang="scss">`、`lang="less"` 和 `lang="sass"`。
- 独立的 `.css`、`.scss`、`.sass`（缩进语法）和 `.less` 文件。

如果配置文件没有 `stylesheet` 部分，样式表将保持不变。

## 声明顺序 {#declaration-order}

两个内置配置文件都使用：

```yaml
stylesheet:
  declarationOrder: smacss
  vendorPrefix: start
```

- `declarationOrder`：`smacss`（重要的、影响布局流的属性在前）、`concentric`（从盒模型外侧向内）或 `alphabetical`。
- `declarationOrderGroupBy`：如何将声明划分为独立排序的连续段：`non-declaration`（默认）或 `non-declaration-and-empty-line`。

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

## 厂商前缀：`vendorPrefix: start` {#vendor-prefixes-vendorprefix-start}

带厂商前缀的声明（`-webkit-…`、`-moz-…`）被视为与标准属性分开的部分。它们是方言，而且其中许多从未被标准化。使用 `start` 时，它们会按简单的字母顺序放在同一连续段中其他声明的**前面**，不按属性分组。

为什么是 `start` 而不是 `end`：带前缀的属性通常是标准属性的别名，而后出现的声明优先。把前缀放在最后可能会让方言覆盖标准值（例如带两个值的 `-webkit-border-radius`）。`start` 让标准属性保持在最后，这与 Autoprefixer 的输出一致。

malva 没有这样的选项（它按去掉前缀后的名称排序）。annealer 会重新解析 malva 的输出，只移动带前缀的声明。连续段与 malva 的排序单元一致。声明之间夹有注释的连续段保持不变。该选项仅在设置了 `declarationOrder` 时生效。`alongside`（未设置时的默认值）保持 malva 的行为。

## 静态 `style` 属性（可选） {#static-style-attributes-opt-in}

`stylesheet.styleAttribute: true` 还会对静态 `style` 属性中的声明排序。两个内置配置文件中都为关闭。

- 值保持在一行。
- CSS 字符串使用与属性引号相反的引号。对于常见的双引号属性，即为 annealer 本就偏好的单引号。
- 缺少的末尾 `;` 保持缺少。
- 绑定的 `:style` 从不处理。
- 当值没有引号、包含模板语法、注释、`{}`、`<` 或 `&`，或无法解析为普通声明时，保持原样。

## 其他 malva 选项 {#other-malva-options}

其他 malva 选项放在 `stylesheet.malva` 下（例如 `printWidth`、`quotes`、`hexCase`）。未知键会被拒绝，因此拼写错误不会被悄悄忽略。参见[配置文件参考](../../api/profile.md#stylesheet)。

## 引号 {#quotes}

CSS 字符串使用单引号（`@use 'sass:math'`、`font-family: 'Noto Sans'`），与 Prettier 和 oxfmt 在 `singleQuote` 下的写法一致。当使用单引号需要转义时，字符串保持双引号（`"O'Brien"`）。这是 annealer 的默认值（`quotes: prefer-single`）；malva 自身的默认值是双引号。可设置 `stylesheet.malva.quotes` 进行修改。
