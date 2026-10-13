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
anyl --write src/**/*.vue        # その場で上書き
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
