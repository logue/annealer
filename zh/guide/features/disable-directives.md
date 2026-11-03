> For AI agents: the complete documentation index is available at /annealer/zh/llms.txt, the full documentation bundle is available at /annealer/zh/llms-full.txt.

# 禁用指令

有些代码必须保持其原有形式，例如第三方代码片段或有意保留的旧顺序。可以用注释让 annealer 跳过这些代码。

```html
<!-- annealer-disable -->
<!-- annealer-enable -->
<!-- annealer-disable-next-line: [reason] -->
```

```css
/* annealer-disable */
/* annealer-enable */
/* annealer-disable-next-line: [reason] */
```

## 规则 \{#rules}

- **仅限块注释。** 不支持 `//`，因为 CSS 没有行注释。
- **`annealer-disable`** 覆盖其后的所有内容，直到 `annealer-enable` 或文档末尾。放在文档开头时，文件保持不变。
- **`annealer-disable-next-line`** 覆盖从下一行开始的结构：
  - 一个起始标签（整个标签，即使跨越多行，以及 `<style>` 元素的内容），或
  - 一条 CSS 语句（一条声明或一整条规则）。
- **`:` 之后的原因是可选的**，annealer 会忽略它。
- **被覆盖的代码逐字保留并留在原位。** 在样式表中，即使相邻声明被排序，被覆盖的声明也保持在原处。
- **作用范围：** 标记注释作用于标记（以及整个 `<style>` 元素）。CSS 注释作用于其所在的样式表内部。
- **未知指令会报错。** 以 `annealer-` 开头但不是已知指令的注释（例如拼写错误 `annealer-disabel`）会使运行失败，而不是被忽略。

```html
<!-- annealer-disable-next-line: copied from the vendor's docs -->
<a href="/" id="vendor-link">Kept as written</a>
<a href="/" id="home">Reordered</a>
```

## 实现说明（样式表） \{#implementation-note-stylesheets}

无法复用 malva 自身的忽略注释（`malva-ignore`）。它不接受 `: reason` 形式，而且排序仍会移动被忽略的声明。因此，annealer 会在调用 malva 之前用内部标记来标记被覆盖的语句，并在之后将其移除。

Sass（缩进语法）没有 `;`，因此在 Sass 中这些标记会单独占一行。malva 在 Sass 中只认可单独占一行的忽略注释。
