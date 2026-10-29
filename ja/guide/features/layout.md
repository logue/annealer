> For AI agents: the complete documentation index is available at /annealer/ja/llms.txt, the full documentation bundle is available at /annealer/ja/llms-full.txt.

# レイアウト

属性を並べ替えると、タグの元の行構成が崩れることがあります。プロファイルの `layout` セクションで、並べ替え後のタグのレイアウトを定めます。各規則は eslint-plugin-vue のルールに対応しているため、それらのルールを残しているプロジェクトでも結果が食い違いません。

どちらの組み込みプロファイルでも `closingBracketNewline`、`oneAttributePerLine`、`selfClosing.void`（`<br />`、`<hr />`、`<wbr />` など）は有効です。空要素以外の自己終了（`selfClosing.normal`/`component`）と `contentNewline` は並べ替えに必要な範囲を超えてマークアップを書き換えるため、組み込みプロファイルでは設定値だけを定義し、無効（`enabled: false`）にしています。必要なときに有効にしてください。

| 規則               | CLI                 | JavaScript             | プロファイル                            |
| ---------------- | ------------------- | ---------------------- | --------------------------------- |
| `selfClosing`    | `--self-closing`    | `selfClosing: true`    | `selfClosing.enabled: true`       |
| `contentNewline` | `--content-newline` | `contentNewline: true` | `contentNewline.enabled: true`    |
| `expandNonVoid`  | `--expand-non-void` | `expandNonVoid: true`  | `selfClosing.expandNonVoid: true` |

CLI のフラグと JavaScript のオプションは、プロファイルの `enabled` より優先されます。`--no-self-closing` / `--no-content-newline`（または `false`）で、規則を有効にしているプロファイルでも無効にできます。

## `closingBracketNewline`

`vue/html-closing-bracket-newline` 相当です。複数行の開始タグでは、閉じ括弧を独立した行に置きます。属性が 1 行に収まる開始タグでは、`>` をその行に残します（`singleline: never`）。

## `oneAttributePerLine`

`vue/max-attributes-per-line` の複数行時の既定に相当します。属性が複数行にわたる開始タグでは、1 行に 1 属性を置きます。

属性が元の行位置を保つと、並べ替えによって 1 行に 2 つの属性が並んでしまうことがあるため、この規則が必要です。

<!-- prettier-ignore -->

```html
<!-- 入力 -->
<div class="a"
  id="x"></div>

<!-- 出力 -->
<div
  id="x"
  class="a"
></div>
```

## `selfClosing`

`vue/html-self-closing` 相当です。要素の種類ごとに `always`、`never`、`preserve` のいずれかを指定します。`void` は常に適用し、`enabled`（組み込み `vue` プロファイルでは無効、上記参照）は `normal` と `component` だけを切り替えます。

| 種類          | 例            | 組み込み `html` | 組み込み `vue` |
| ----------- | ------------ | ----------- | ---------- |
| `void`      | `<br />`     | `always`    | `always`   |
| `normal`    | `<div />`    | –           | `always`   |
| `component` | `<MyComp />` | –           | `always`   |

- `normal` と `component` は Vue テンプレートでのみ適用します。素の HTML では `<div />` は閉じられていない開始タグになるためです。
- `--void-self-closing` / `--no-void-self-closing`（JavaScript では `voidSelfClosing: true` / `false`）で、プロファイルを編集せずに `void` を `always` / `never` にできます。
- 空白だけのコンテンツは空とみなします。ただし `pre` と `textarea` は除きます。
- SFC のトップレベルブロック（`<template>`、`<script>`、`<style>`）には手を加えません。

### `expandNonVoid`

素の HTML は空要素以外の要素のスラッシュを無視するため、`<div />` は `<div>` と同じ意味になります。要素は閉じられず、後に続く内容を取り込みます。`expandNonVoid: true`（`--expand-non-void`、JavaScript では `expandNonVoid: true`）で、こうした要素に終了タグを付けます。ブラウザーの解釈が変わるため、組み込みプロファイルでは無効です。

<!-- prettier-ignore -->

```html
<!-- 入力 -->
<div class="spacer" /><my-icon/>
<svg><path d="M0 0" /></svg>

<!-- 出力（--expand-non-void） -->
<div class="spacer"></div><my-icon></my-icon>
<svg><path d="M0 0" /></svg>
```

- 外部コンテンツ（`<svg>`、`<math>`、`<rdf:RDF>` などの名前空間付き XML）には手を加えません。そこでは自己終了が有効なためです（[制限事項](/annealer/ja/guide/start/limitations.md)）。
- Vue テンプレートには影響しません。Vue では `normal: never` / `component: never` を使ってください。

:::warning vue/html-self-closing との違い
`void: always` は `vue/html-self-closing` の既定（`html.void: never`）と異なります。その ESLint ルールを残す場合は、そちらで `html.void: always` にするか、annealer のプロファイルで `void: never` にしてください。
:::

## `contentNewline`

`vue/multiline-html-element-content-newline` 相当です。組み込みプロファイルでは無効です（上記参照）。独自のプロファイルでは、キーを省略するか `enabled: false` にすると無効、`{}` を指定すると既定値で有効になります。キーのないプロファイルに `--content-newline` を指定した場合も既定値が使われます。

- 終了タグが開始タグより後の行で始まる要素を**複数行**の要素とみなします（ESLint ルールの定義と同じ）。その要素のコンテンツは、ちょうど 1 つの改行で始まり、1 つの改行で終わるようにします。
- `allowEmptyLines: true` にすると、空行をまとめずに残します。
- 空の要素には手を加えません。
- `ignore` には、中身も含めて手を加えない要素を列挙します。既定はルールの既定と同じく `pre`、`textarea`、インライン要素（`a`、`span`、`strong` など）で、追加した空白が表示に影響しうる要素です。
- 閉じられていない HTML 要素（`</li>` のない `<li>`）は祖先の終了タグで暗黙に閉じられたものとし、手を加えません。
- 追加するのは改行だけで、インデントは整形し直しません。新しい行のインデントは、複数行の開始タグなら属性のインデント、そうでなければ最初の一段深いコンテンツ行、それもなければタグのインデントに空白 2 つ（タグがタブでインデントされていればタブ）を加えたものです。

<!-- prettier-ignore -->

```vue
<!-- 入力 -->
<template>
  <div class="a"
    id="x"><MyComp></MyComp></div>
</template>

<!-- 出力（vue プロファイル、--self-closing --content-newline） -->
<template>
  <div
    id="x"
    class="a"
  >
    <MyComp />
  </div>
</template>
```

この規則は HTML プロファイルにも定義しています。ブロックレベルのコンテンツの先頭と末尾の空白は表示されず、表示されうる要素は `ignore` リストで除外しているためです。

コンテンツを新しい行に移すと、入れ子のタグの閉じ括弧がそろえるインデントが変わることがあります。そのため annealer は、結果が安定するまで文書を繰り返し整形します（最大 4 回、通常は 2 回）。
