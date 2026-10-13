> For AI agents: the complete documentation index is available at /annealer/ja/llms.txt, the full documentation bundle is available at /annealer/ja/llms-full.txt.

# JavaScript API

```ts
import { builtinProfile, format, init, languageFromPath } from 'annealer';
```

このパッケージは Rust コアの WebAssembly ビルドと TypeScript ラッパーです。Node.js、ブラウザー、UMD ビルドで動きます。

WebAssembly バイナリ（約 1.3 MB、gzip 後 0.65 MB）は base64 でパッケージに埋め込まれています。アセットの読み込み設定は不要で、`format()` は同期関数です。

## `format(input, options?)`

```ts
function format(input: string, options?: FormatOptions): string;
```

属性（HTML、Vue）または宣言（CSS、SCSS、Sass、Less）を並べ替えます。

- `options.language`: 入力の言語。既定は `'html'`。
- `options.profile`: 組み込みプロファイル名（`'html'`、`'vue'`）またはプロファイルの YAML テキスト。既定は `language` に対応する組み込みプロファイル。

入力やプロファイルが不正な場合は `Error` を投げます。

```ts
format('<a title="Home" href="/" id="home">Home</a>');
// => '<a id="home" title="Home" href="/">Home</a>'

format(source, { language: 'vue' });
format(source, { language: 'html', profile: yamlText });
```

## `init()`

```ts
function init(): Promise<void>;
```

WebAssembly モジュールを、処理をブロックせずに事前コンパイルします。省略可能で、呼ばなければ `format()` の初回呼び出し時に同期的にコンパイルします。

## `languageFromPath(path)`

```ts
function languageFromPath(path: string): Language | undefined;
```

ファイル名やパスから言語を推定します（`.html`、`.htm`、`.vue`、`.css`、`.scss`、`.sass`、`.less`）。それ以外の拡張子では `undefined` を返します。

## `builtinProfile(name)`

```ts
function builtinProfile(name: 'html' | 'vue'): string;
```

組み込みプロファイルの YAML テキストを返します。独自プロファイルを作る出発点として使えます。

```ts
const yaml = builtinProfile('vue').replace(
  'oneAttributePerLine: true',
  'oneAttributePerLine: false'
);
format(source, { language: 'vue', profile: yaml });
```

## 型

```ts
type Language = 'html' | 'vue' | 'css' | 'scss' | 'sass' | 'less';

type FormatOptions = {
  language?: Language;
  profile?: string;
};

type Meta = {
  version: string;
  date: string;
};
```

`Language`、`FormatOptions`、`Meta` は値としてもエクスポートされます。それぞれ拡張子と言語の対応表、既定のオプション、ビルド情報です。
