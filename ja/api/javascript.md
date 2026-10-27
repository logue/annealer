> For AI agents: the complete documentation index is available at /annealer/ja/llms.txt, the full documentation bundle is available at /annealer/ja/llms-full.txt.

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

このパッケージは Rust コアの WebAssembly ビルドと TypeScript ラッパーです。Node.js、ブラウザー、UMD ビルドで動きます。

WebAssembly バイナリ（約 1.5 MB、gzip 後 0.7 MB）は base64 でパッケージに埋め込まれています。アセットの読み込み設定は不要で、`format()` は同期関数です。

## `format(input, options?)`

```ts
function format(input: string, options?: FormatOptions): string;
```

属性（HTML、Vue）または宣言（CSS、SCSS、Sass、Less）を並べ替えます。

- `options.language`: 入力の言語。既定は `'html'`。
- `options.profile`: 組み込みプロファイル名（`'html'`、`'vue'`）またはプロファイルの YAML テキスト。既定は `language` に対応する組み込みプロファイル。
- `options.classOrder`: 組み込みのクラス順の名前（`'tailwind'`）またはクラス順の YAML テキスト。静的な `class` 属性のクラスを並べ替え、プロファイルの `classOrder` より優先されます。既定ではクラスは元の順のままです。[クラス順](/annealer/ja/guide/features/class-order.md)を参照してください。
- `options.removeDuplicateClasses`: `true` で静的な `class` 属性の重複したクラスを、最初のものを残して削除します。`classOrder` の有無にかかわらず使えます。`false` で残します。既定はクラス順の `removeDuplicates`（`tailwind` では無効、クラス順がなければ無効）。
- `options.printWidth`: スタイルシートと `<style>` ブロックの行幅（`stylesheet.malva.printWidth`、1 以上）。フォーマッターに合わせます。Prettier と Biome は 80、oxfmt は 100。既定はプロファイルの値（80）。
- `options.selfClosing`: `true` で Vue テンプレートの空要素と空のコンポーネントを自己終了にし（`layout.selfClosing.normal`/`component`: `<div />`、`<MyComp />`）、`false` で無効にします。既定はプロファイルの設定（組み込みプロファイルでは無効）。空要素（`<br />`）は常に `layout.selfClosing.void` に従います。
- `options.contentNewline`: `true` で複数行の要素のコンテンツを独立した行に置きます。プロファイルに `layout.contentNewline` がなければ規則の既定値を使います。`false` で無効にします。既定はプロファイルの設定（組み込みプロファイルでは無効）。[レイアウト](/annealer/ja/guide/features/layout.md)を参照してください。

入力やプロファイルが不正な場合は `Error` を投げます。

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

## `builtinClassOrder(name)`

```ts
function builtinClassOrder(name: 'tailwind'): string;
```

組み込みのクラス順の YAML テキストを返します。独自のクラス順の出発点として使えます。

## 型

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

`Language`、`FormatOptions`、`Meta` は値としてもエクスポートされます。それぞれ拡張子と言語の対応表、既定のオプション、ビルド情報です。
