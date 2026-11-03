> For AI agents: the complete documentation index is available at /annealer/zh/llms.txt, the full documentation bundle is available at /annealer/zh/llms-full.txt.

# 快速开始

annealer 有三个入口，共用同一个 Rust 核心：

- **`anyl`**：命令行工具。
- **`annealer`（npm）**：基于 WebAssembly 的 JavaScript API。
- **`annealer`（Rust crate）**：纯函数 `format()`。

当前版本为 **0.1.0-alpha.1**。

## 安装 \{#installation}

```bash
cargo install annealer --version 0.1.0-alpha.1   # anyl CLI
pnpm add -D annealer@0.1.0-alpha.1               # JavaScript API
```

预发布版本不会被 `^0.1.0` 这样的版本范围匹配，因此请明确指定版本。要从仓库的检出副本构建 CLI，请运行 `cargo install --path crates/annealer`。

## CLI

```bash
anyl src/App.vue                 # 输出结果
anyl --write src/**/*.vue        # 直接覆盖
anyl --check index.html          # 有文件会被修改时以 1 退出
anyl --profile my-profile.yaml page.html
```

支持的输入：`.html`、`.htm`、`.vue`、`.css`、`.scss`、`.sass`、`.less`。语言由扩展名判断，除非指定 `--profile`，否则每种语言使用对应的内置配置文件。所有选项参见 [CLI](/annealer/zh/api/cli.md)。

## JavaScript

```ts
import { format } from 'annealer';

format('<a title="Home" href="/" id="home">Home</a>');
// => '<a id="home" title="Home" href="/">Home</a>'

format(source, { language: 'vue' });
format(source, { language: 'html', profile: yamlText });
```

`format()` 是同步的。WebAssembly 二进制文件已内嵌在包中，因此无需配置加载器或打包工具。参见 [JavaScript API](/annealer/zh/api/javascript.md)。

## 示例 \{#example}

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

## 配置文件 \{#profiles}

内置两个配置文件：

| 配置文件   | 用于                                     | 概要                                          |
| ------ | -------------------------------------- | ------------------------------------------- |
| `html` | `.html`、`.css`、`.scss`、`.sass`、`.less` | 面向纯 HTML 的语义顺序。                             |
| `vue`  | `.vue`                                 | `vue/attributes-order` 的层级加上 annealer 的子排序。 |

如需自定义，可以从内置配置文件开始（JavaScript 中使用 `builtinProfile('vue')`，或参考 `crates/annealer/profiles/*.yaml`），然后通过 `--profile` 或 `profile` 选项传入你的 YAML。参见[配置文件参考](/annealer/zh/api/profile.md)。
