---
description: annealer npm 包，包含 format()、init()、languageFromPath()、builtinProfile()、builtinClassOrder() 及导出的类型。
---

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
- `options.classOrder`：内置类顺序名（`'tailwind'`，或实验性、非官方的 `'bootstrap'`）或类顺序的 YAML 文本。对静态 `class` 属性中的类排序，并覆盖配置文件中的 `classOrder`。默认：类保持原有顺序。参见[类顺序](../guide/features/class-order.md)。

当输入或配置文件无效时抛出 `Error`。

```ts
format('<a title="Home" href="/" id="home">Home</a>');
// => '<a id="home" title="Home" href="/">Home</a>'

format(source, { language: 'vue' });
format(source, { language: 'html', profile: yamlText });
format(source, { language: 'vue', classOrder: 'tailwind' });
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
function builtinClassOrder(name: 'tailwind' | 'bootstrap'): string;
```

返回内置类顺序的 YAML 文本，可作为自定义类顺序的起点。

## 类型

```ts
type Language = 'html' | 'vue' | 'css' | 'scss' | 'sass' | 'less';

type FormatOptions = {
  language?: Language;
  profile?: string;
  classOrder?: string;
};

type Meta = {
  version: string;
  date: string;
};
```

`Language`、`FormatOptions` 和 `Meta` 同时也作为值导出：分别是扩展名到语言的映射、默认选项和构建元数据。
