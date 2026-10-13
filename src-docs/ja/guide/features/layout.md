---
description: 複数行の開始タグ、自己終了要素、要素のコンテンツに関するレイアウト規則。eslint-plugin-vue の対応ルールと同等です。
---

# レイアウト

属性を並べ替えると、タグの元の行構成が崩れることがあります。プロファイルの `layout` セクションで、並べ替え後のタグのレイアウトを定めます。各規則は eslint-plugin-vue のルールに対応しているため、それらのルールを残しているプロジェクトでも結果が食い違いません。

どちらの組み込みプロファイルでも、以下の規則はすべて有効です。

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

`vue/html-self-closing` 相当です。要素の種類ごとに `always`、`never`、`preserve` のいずれかを指定します。

| 種類        | 例           | 組み込み `html` | 組み込み `vue` |
| ----------- | ------------ | --------------- | -------------- |
| `void`      | `<br />`     | `always`        | `always`       |
| `normal`    | `<div />`    | –               | `always`       |
| `component` | `<MyComp />` | –               | `always`       |

- `normal` と `component` は Vue テンプレートでのみ適用します。素の HTML では `<div />` は閉じられていない開始タグになるためです。
- 空白だけのコンテンツは空とみなします。ただし `pre` と `textarea` は除きます。
- SFC のトップレベルブロック（`<template>`、`<script>`、`<style>`）には手を加えません。

:::warning vue/html-self-closing との違い
`void: always` は `vue/html-self-closing` の既定（`html.void: never`）と異なります。その ESLint ルールを残す場合は、そちらで `html.void: always` にするか、annealer のプロファイルで `void: never` にしてください。
:::

## `contentNewline`

`vue/multiline-html-element-content-newline` 相当です。キーを省略すると無効、`{}` を指定すると既定値で有効になります。

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

<!-- 出力（vue プロファイル） -->
<template>
  <div
    id="x"
    class="a"
  >
    <MyComp />
  </div>
</template>
```

この規則は HTML プロファイルでも有効です。ブロックレベルのコンテンツの先頭と末尾の空白は表示されず、表示されうる要素は `ignore` リストで除外しているためです。

コンテンツを新しい行に移すと、入れ子のタグの閉じ括弧がそろえるインデントが変わることがあります。そのため annealer は、結果が安定するまで文書を繰り返し整形します（最大 4 回、通常は 2 回）。
