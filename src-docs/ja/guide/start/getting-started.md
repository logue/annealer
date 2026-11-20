---
description: anyl CLI または JavaScript から annealer を実行し、組み込みプロファイルまたは独自プロファイルを選ぶ方法。
---

# 使い方

annealer には、同じ Rust コアを使う 3 つの入口があります。

- **`anyl`**: コマンドラインツール
- **`annealer`（npm）**: WebAssembly を使う JavaScript API
- **`annealer`（Rust クレート）**: 純粋関数 `format()`

## CLI

リポジトリから CLI をビルドします（npm パッケージとクレートは未公開です）。

```bash
cargo install --path crates/annealer
```

ファイルを整形します。

```bash
anyl src/App.vue                 # 結果を出力
anyl --write "src/**/*.vue"      # その場で上書き
anyl --check index.html          # 変更が必要なファイルがあれば終了コード 1
anyl --profile my-profile.yaml page.html
```

対応する入力は `.html`、`.htm`、`.vue`、`.css`、`.scss`、`.sass`、`.less` です。言語は拡張子から判定し、`--profile` を指定しない限り、言語に対応する組み込みプロファイルを使います。すべてのオプションは [CLI](../../api/cli.md) を参照してください。

## JavaScript

```ts
import { format } from 'annealer';

format('<a title="Home" href="/" id="home">Home</a>');
// => '<a id="home" title="Home" href="/">Home</a>'

format(source, { language: 'vue' });
format(source, { language: 'html', profile: yamlText });
```

`format()` は同期関数です。WebAssembly バイナリはパッケージに埋め込まれているため、ローダーやバンドラーの設定は不要です。[JavaScript API](../../api/javascript.md) を参照してください。

## 例

入力:

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

`html` プロファイルでの出力:

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

- 先頭は `id` と `class`、続いて補足（`placeholder`）、意味（`type`）、キーと値（`name`）、状態（`required`）の属性、最後に ARIA です。
- 複数行のタグは 1 行に 1 属性になり、閉じ括弧は独立した行に置かれます。
- 空要素 `<input>` は `<input />` になります。

## プロファイル

組み込みプロファイルは 2 つです。

| プロファイル | 用途                                       | 概要                                                          |
| ------------ | ------------------------------------------ | ------------------------------------------------------------- |
| `html`       | `.html`、`.css`、`.scss`、`.sass`、`.less` | 素の HTML 向けの意味に基づく順序                              |
| `vue`        | `.vue`                                     | `vue/attributes-order` の階層に annealer の細分化を加えたもの |

カスタマイズするときは、組み込みプロファイル（JavaScript では `builtinProfile('vue')`、または `crates/annealer/profiles/*.yaml`）をもとに YAML を書き、`--profile` または `profile` オプションで渡します。[プロファイルリファレンス](../../api/profile.md)を参照してください。

## フォーマッター・リンターとの併用 {#with-formatters-and-linters}

annealer は、フォーマッターやリンター（oxfmt、Prettier、oxlint、Rslint など）より**前に**実行してください。

1. **annealer** が属性・クラス・宣言を並べ替えます。開始タグを複数行に分けることもあります。
2. **フォーマッター**が、並べ替えたコードの最終的な空白と改行を決めます。
3. **リンター**を最後に実行し、整形済みの結果を検査（または `--fix`）します。

逆の順で実行すると、整形済みのコードを annealer が変更する（属性の移動、タグの分割）ため、結果が整形済みでなくなり、チェックが失敗し続けます。

### `run-s` でつなぐ {#chain-with-run-s}

各段階は `&&` ではなく、[npm-run-all2](https://github.com/bcomnes/npm-run-all2) の `run-s` でつなぐことを推奨します。

- 書いた順に 1 つずつ実行し、最初に失敗した段階で止まります。
- スクリプトを実行するシェルに依存しないので、どの OS でも同じように動きます。
- 各段階が独立したスクリプトになるので、単独でも実行できます（`pnpm run format:annealer`）。

```bash
pnpm add -D npm-run-all2
```

oxfmt と oxlint の場合:

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

Prettier と Rslint で、対象を `src/` に絞る場合:

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

補足:

- 段階は上の例のように名前で明示してください。`run-s format:*` でも動きますが、一致したスクリプトを package.json に書かれた順に実行するため、編集の際に順序が崩れやすくなります。
- package.json ではグロブパターンをクォートし（`\"src/**/*.vue\"`）、`anyl` 自身に展開させてください。ディレクトリ、グロブ、除外、無視ファイルについては[ファイル](../../api/cli.md#files)を参照してください。
- CI では `check` を実行します。どれか 1 つのツールが変更を加えそうな時点で失敗します。
- 同じものを並べ替えるルールやプラグインは無効にしてください。ツール同士が書き換え合います。[クラス順](../features/class-order.md)を使うなら `prettier-plugin-tailwindcss` と `eslint-plugin-tailwindcss`（`classnames-order`）、プロファイルに `stylesheet.declarationOrder` があるなら CSS プロパティ順のルールが該当します。`vue/attributes-order` は、その段階を細分化した組み込みの `vue` プロファイルと矛盾しません。
