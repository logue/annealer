> For AI agents: the complete documentation index is available at /annealer/zh/llms.txt, the full documentation bundle is available at /annealer/zh/llms-full.txt.

# JavaScript API

```ts
import { builtinClassOrder, builtinProfile, format, init, languageFromPath } from 'annealer';
```

该包是 Rust 核心的 WebAssembly 构建版本，外加一层 TypeScript 封装。它可以在 Node.js、浏览器中使用，也提供 UMD 构建。

WebAssembly 二进制文件（约 1.5 MB，gzip 后 0.7 MB）以 base64 形式内嵌在包中。无需配置任何资源加载，`format()` 是同步的。

## `format(input, options?)`

```ts
function format(input: string, options?: FormatOptions): string;
```

对属性（HTML、Vue）或声明（CSS、SCSS、Sass、Less）进行排序。

- `options.language`：输入的语言。默认：`'html'`。
- `options.profile`：内置配置文件名（`'html'`、`'vue'`）或配置文件的 YAML 文本。默认：与 `language` 对应的内置配置文件。
- `options.classOrder`：内置类顺序名（`'tailwind'`）或类顺序的 YAML 文本。对静态 `class` 属性中的类排序，并覆盖配置文件中的 `classOrder`。默认：类保持原有顺序。参见[类顺序](/annealer/zh/guide/features/class-order.md)。
- `options.removeDuplicateClasses`：`true` 会删除静态 `class` 属性中重复的类并保留第一个，无论是否指定 `classOrder`；`false` 则保留重复项。默认：类顺序的 `removeDuplicates`（`tailwind` 中为关闭，没有类顺序时也为关闭）。
- `options.printWidth`：样式表和 `<style>` 块的行宽（`stylesheet.malva.printWidth`），至少为 1。请与所用格式化工具保持一致：Prettier 和 Biome 为 80，oxfmt 为 100。默认：配置文件中的值（80）。
- `options.selfClosing`：`true` 会在 Vue 模板中让空元素和组件自闭合（`layout.selfClosing.normal`/`component`：`<div />`、`<MyComp />`）；`false` 将其关闭。默认：配置文件的设置（内置配置文件中为关闭）。空元素（`<br />`）遵循 `voidSelfClosing`。
- `options.voidSelfClosing`：`true` 会让空元素自闭合（`<input />`、`<meta />`；`layout.selfClosing.void: always`）；`false` 则去掉斜杠（`<input>`、`<meta>`；`never`）。默认：配置文件的设置（内置配置文件中为 `always`）。
- `options.expandNonVoid`：`true` 会为纯 HTML 中以自闭合形式书写的非空元素补上结束标签（`<div />` → `<div></div>`；`layout.selfClosing.expandNonVoid`）。HTML 会忽略斜杠，所以 `<div />` 会让元素保持未闭合。`<svg>` 和 `<math>` 内部的元素不受影响。`false` 将其关闭。默认：配置文件的设置（内置配置文件中为关闭）。
- `options.contentNewline`：`true` 会让多行元素的内容独占一行；配置文件中没有 `layout.contentNewline` 时使用该规则的默认值；`false` 将其关闭。默认：配置文件的设置（内置配置文件中为关闭）。参见[布局](/annealer/zh/guide/features/layout.md)。

当输入或配置文件无效时抛出 `Error`，包括输入中含有以 Pug 或 Stylus 编写的 `<template>` 或 `<style>`（`lang="pug"`、`"jade"`、`"stylus"`、`"styl"`）的情况，因为这些语言不受支持。

```ts
format('<a title="Home" href="/" id="home">Home</a>');
// => '<a id="home" title="Home" href="/">Home</a>'

format(source, { language: 'vue' });
format(source, { language: 'html', profile: yamlText });
format(source, { language: 'vue', classOrder: 'tailwind' });
format(source, { classOrder: 'tailwind', removeDuplicateClasses: true });
format(source, { language: 'vue', selfClosing: true, contentNewline: true });
format(source, { expandNonVoid: true, voidSelfClosing: false });
```

## `init()`

```ts
function init(): Promise<void>;
```

以非阻塞方式提前编译 WebAssembly 模块。此调用是可选的：`format()` 会在首次使用时同步编译。

## `languageFromPath(path)`

```ts
function languageFromPath(path: string): Language | undefined;
```

根据文件名或路径推断语言（`.html`、`.htm`、`.vue`、`.css`、`.scss`、`.sass`、`.less`）。其他扩展名返回 `undefined`。

## `builtinProfile(name)`

```ts
function builtinProfile(name: 'html' | 'vue'): string;
```

返回内置配置文件的 YAML 文本，可作为自定义配置文件的起点。

```ts
const yaml = builtinProfile('vue').replace(
  'oneAttributePerLine: true',
  'oneAttributePerLine: false'
);
format(source, { language: 'vue', profile: yaml });
```

## `builtinClassOrder(name)`

```ts
function builtinClassOrder(name: 'tailwind'): string;
```

返回内置类顺序的 YAML 文本，可作为自定义类顺序的起点。

## 类型

```ts
type Language = 'html' | 'vue' | 'css' | 'scss' | 'sass' | 'less';

type FormatOptions = {
  language?: Language;
  profile?: string;
  classOrder?: string;
  removeDuplicateClasses?: boolean;
  printWidth?: number;
  selfClosing?: boolean;
  voidSelfClosing?: boolean;
  expandNonVoid?: boolean;
  contentNewline?: boolean;
};

type Meta = {
  version: string;
  date: string;
};
```

`Language`、`FormatOptions` 和 `Meta` 同时也作为值导出：分别是扩展名到语言的映射、默认选项和构建元数据。
