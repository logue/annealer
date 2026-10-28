> For AI agents: the complete documentation index is available at /annealer/llms.txt, the full documentation bundle is available at /annealer/llms-full.txt.

# JavaScript API

```ts
import {
  builtinClassOrder,
  builtinProfile,
  format,
  init,
  languageFromPath
} from 'annealer';
```

The package is a WebAssembly build of the Rust core with a TypeScript wrapper. It works in Node.js, in browsers and as a UMD build.

The WebAssembly binary (about 1.5 MB, 0.7 MB gzipped) is embedded in the package as base64. No asset loading needs to be configured, and `format()` is synchronous.

## `format(input, options?)`

```ts
function format(input: string, options?: FormatOptions): string;
```

Reorders attributes (HTML, Vue) or declarations (CSS, SCSS, Sass, Less).

- `options.language`: the input language. Default: `'html'`.
- `options.profile`: a built-in profile name (`'html'`, `'vue'`) or the YAML text of a profile. Default: the built-in profile for `language`.
- `options.classOrder`: a built-in class order name (`'tailwind'`) or the YAML text of a class order. Orders the classes of static `class` attributes and overrides the profile's `classOrder`. Default: classes keep their order. See [Class order](/annealer/guide/features/class-order.md).
- `options.removeDuplicateClasses`: `true` drops repeated classes in static `class` attributes, keeping the first, with or without `classOrder`; `false` keeps them. Default: the class order's `removeDuplicates` (off in `tailwind`, and off without a class order).
- `options.printWidth`: line width of stylesheets and `<style>` blocks (`stylesheet.malva.printWidth`), at least 1. Match your formatter: 80 for Prettier and Biome, 100 for oxfmt. Default: the profile's (80).
- `options.selfClosing`: `true` self-closes empty elements and components in Vue templates (`layout.selfClosing.normal`/`component`: `<div />`, `<MyComp />`); `false` turns that off. Default: the profile's setting (off in the built-in profiles). Void elements (`<br />`) always follow `layout.selfClosing.void`.
- `options.contentNewline`: `true` puts the content of multiline elements on lines of its own, with the rule's defaults when the profile has no `layout.contentNewline`; `false` turns it off. Default: the profile's setting (off in the built-in profiles). See [Layout](/annealer/guide/features/layout.md).

Throws an `Error` when the input or the profile is invalid, including when the input has a `<template>` or `<style>` in Pug or Stylus (`lang="pug"`, `"jade"`, `"stylus"`, `"styl"`), which are not supported.

```ts
format('<a title="Home" href="/" id="home">Home</a>');
// => '<a id="home" title="Home" href="/">Home</a>'

format(source, { language: 'vue' });
format(source, { language: 'html', profile: yamlText });
format(source, { language: 'vue', classOrder: 'tailwind' });
format(source, { classOrder: 'tailwind', removeDuplicateClasses: true });
format(source, { language: 'vue', selfClosing: true, contentNewline: true });
```

## `init()`

```ts
function init(): Promise<void>;
```

Compiles the WebAssembly module ahead of time without blocking. Optional: `format()` compiles it synchronously on first use.

## `languageFromPath(path)`

```ts
function languageFromPath(path: string): Language | undefined;
```

Infers the language from a file name or path (`.html`, `.htm`, `.vue`, `.css`, `.scss`, `.sass`, `.less`). Returns `undefined` for other extensions.

## `builtinProfile(name)`

```ts
function builtinProfile(name: 'html' | 'vue'): string;
```

Returns the YAML text of a built-in profile, as a starting point for a custom one.

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

Returns the YAML text of a built-in class order, as a starting point for a custom one.

## Types

```ts
type Language = 'html' | 'vue' | 'css' | 'scss' | 'sass' | 'less';

type FormatOptions = {
  language?: Language;
  profile?: string;
  classOrder?: string;
  removeDuplicateClasses?: boolean;
  printWidth?: number;
  selfClosing?: boolean;
  contentNewline?: boolean;
};

type Meta = {
  version: string;
  date: string;
};
```

`Language`, `FormatOptions` and `Meta` are also exported as values: the extension-to-language map, the default options, and the build metadata.
