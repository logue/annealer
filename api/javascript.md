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

Throws an `Error` when the input or the profile is invalid.

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
};

type Meta = {
  version: string;
  date: string;
};
```

`Language`, `FormatOptions` and `Meta` are also exported as values: the extension-to-language map, the default options, and the build metadata.
