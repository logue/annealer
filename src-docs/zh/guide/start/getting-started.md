---
description: 通过 anyl CLI 或 JavaScript 运行 annealer，并选择内置或自定义配置文件。
---

# 快速开始

annealer 有三个入口，共用同一个 Rust 核心：

- **`anyl`**：命令行工具。
- **`annealer`（npm）**：基于 WebAssembly 的 JavaScript API。
- **`annealer`（Rust crate）**：纯函数 `format()`。

## CLI

从仓库构建 CLI（npm 包和 crate 尚未发布）：

```bash
cargo install --path crates/annealer
```

然后格式化文件：

```bash
anyl src/App.vue                 # 输出结果
anyl --write "src/**/*.vue"      # 直接覆盖
anyl --check index.html          # 有文件会被修改时以 1 退出
anyl --profile my-profile.yaml page.html
```

支持的输入：`.html`、`.htm`、`.vue`、`.css`、`.scss`、`.sass`、`.less`。语言由扩展名判断，除非指定 `--profile`，否则每种语言使用对应的内置配置文件。所有选项参见 [CLI](../../api/cli.md)。

## JavaScript

```ts
import { format } from 'annealer';

format('<a title="Home" href="/" id="home">Home</a>');
// => '<a id="home" title="Home" href="/">Home</a>'

format(source, { language: 'vue' });
format(source, { language: 'html', profile: yamlText });
```

`format()` 是同步的。WebAssembly 二进制文件已内嵌在包中，因此无需配置加载器或打包工具。参见 [JavaScript API](../../api/javascript.md)。

## 示例 {#example}

输入：

```html
<nav aria-label="Main" class="nav" id="nav">
  <a href="/docs" class="nav__link" aria-current="page">Docs</a>
</nav>
<input
  required
  placeholder="you@example.com"
  name="email"
  type="email"
  id="email"
  class="form__input"
  aria-describedby="email-help"
/>
```

使用 `html` 配置文件的输出：

```html
<nav id="nav" class="nav" aria-label="Main">
  <a class="nav__link" href="/docs" aria-current="page">Docs</a>
</nav>
<input
  id="email"
  class="form__input"
  placeholder="you@example.com"
  type="email"
  name="email"
  required
  aria-describedby="email-help"
/>
```

- `id` 和 `class` 排在最前，然后是补充（`placeholder`）、语义（`type`）、键值（`name`）和状态（`required`）属性，最后是 ARIA。
- 多行标签每行一个属性，结束括号单独占一行。
- 空元素 `<input>` 变为 `<input />`。

## 配置文件 {#profiles}

内置两个配置文件：

| 配置文件 | 用于                                       | 概要                                                  |
| -------- | ------------------------------------------ | ----------------------------------------------------- |
| `html`   | `.html`、`.css`、`.scss`、`.sass`、`.less` | 面向纯 HTML 的语义顺序。                              |
| `vue`    | `.vue`                                     | `vue/attributes-order` 的层级加上 annealer 的子排序。 |

如需自定义，可以从内置配置文件开始（JavaScript 中使用 `builtinProfile('vue')`，或参考 `crates/annealer/profiles/*.yaml`），然后通过 `--profile` 或 `profile` 选项传入 YAML。参见[配置文件参考](../../api/profile.md)。

## 与格式化工具和 linter 配合使用 {#with-formatters-and-linters}

请在格式化工具和 linter（oxfmt、Prettier、oxlint、Rslint 等）**之前**运行 annealer：

1. **annealer** 对属性、类和声明排序，并可能把开始标签拆成多行。
2. **格式化工具**随后决定排序后代码的最终空白和换行。
3. **linter** 最后运行，检查（或 `--fix`）格式化后的结果。

若顺序相反，annealer 会修改已格式化的代码（移动属性、拆分标签），结果不再是格式化后的状态，检查会一直失败。

### 用 `run-s` 串联 {#chain-with-run-s}

推荐用 [npm-run-all2](https://github.com/bcomnes/npm-run-all2) 的 `run-s` 而不是 `&&` 串联各步骤：

- 按书写顺序逐个运行，并在第一个失败的步骤处停止。
- 不依赖运行脚本的 shell，因此在所有操作系统上行为一致。
- 每个步骤都是独立的脚本，也可以单独运行（`pnpm run format:annealer`）。

```bash
pnpm add -D npm-run-all2
```

使用 oxfmt 和 oxlint：

```json
{
  "scripts": {
    "format": "run-s format:annealer format:oxfmt format:oxlint",
    "format:annealer": "anyl --write .",
    "format:oxfmt": "oxfmt",
    "format:oxlint": "oxlint --fix",
    "check": "run-s check:annealer check:oxfmt check:oxlint",
    "check:annealer": "anyl --check .",
    "check:oxfmt": "oxfmt --check",
    "check:oxlint": "oxlint"
  }
}
```

使用 Prettier 和 Rslint，并限定在 `src/`：

```json
{
  "scripts": {
    "format": "run-s format:annealer format:prettier format:rslint",
    "format:annealer": "anyl --write \"src/**/*.{vue,html,css,scss}\"",
    "format:prettier": "prettier --write .",
    "format:rslint": "rslint --fix",
    "check": "run-s check:annealer check:prettier check:rslint",
    "check:annealer": "anyl --check \"src/**/*.{vue,html,css,scss}\"",
    "check:prettier": "prettier --check .",
    "check:rslint": "rslint"
  }
}
```

说明：

- 请像上例一样显式写出步骤名。`run-s format:*` 也可以，但它按 package.json 中的书写顺序运行匹配的脚本，编辑时容易打乱顺序。
- 在 package.json 中给 glob 模式加引号（`\"src/**/*.vue\"`），让 `anyl` 自己展开。关于目录、glob、排除和忽略文件，参见[文件](../../api/cli.md#files)。
- 在 CI 中运行 `check`：只要有一个工具会做出修改，它就会失败。
- 请关闭对同一内容排序的规则和插件，否则工具之间会互相改写：使用[类顺序](../features/class-order.md)时关闭 `prettier-plugin-tailwindcss` 和 `eslint-plugin-tailwindcss`（`classnames-order`）；配置文件设置了 `stylesheet.declarationOrder` 时关闭 CSS 属性顺序规则。`vue/attributes-order` 与内置的 `vue` 配置文件一致，后者细化了它的层级。
