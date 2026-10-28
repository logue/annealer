---
description: annealer の現在の制約事項。対応する入力、マークアップとレイアウト、クラス順、スタイルシート、プロファイルと CLI について。
---

# 制約事項

annealer は開発中です。このページでは、まだ対応していないことと、設計上行わないことをまとめます。詳しくは各項目のリンク先を参照してください。

## 状況 {#status}

- **未公開です。** npm パッケージと Rust クレートはまだレジストリにありません。CLI は `cargo install --path crates/annealer` でビルドします。
- **プロファイルの形式は、安定版と宣言するまで変わる可能性があります**（`schemaVersion: 1`）。

## 対応する入力 {#supported-input}

- HTML、Vue 3 SFC のテンプレート、CSS、SCSS、Sass（インデント構文）、Less。それ以外は未対応のファイル形式として報告します。
- **未対応:** JSX/TSX、Svelte、Astro、Angular のテンプレート、Markdown/MDX、Vue 2 の構文（`.sync`、`.native`）。Svelte は今後対応する予定です。
- **Pug と Stylus は未対応です。** `.pug`、`.jade`、`.styl`、`.stylus` のファイルは読み込まずに未対応のファイル形式として報告します。`lang="pug"`、`lang="jade"`、`lang="stylus"`、`lang="styl"` の `<template>` や `<style>` を含むファイルは、そのブロックを読み飛ばすのではなく、ファイル全体をエラー（`unsupported language`）にします。Pug をマークアップとして走査したり、他のブロックだけを整形したりしないためです。これらのファイルは annealer に渡すパスから除外してください。
- `.vue` ファイルの `<script>` とカスタムブロック（`<i18n>`）には手を加えません。

## マークアップ {#markup}

- **DOM を作りません。** 開始タグを字句解析で読むだけで、HTML の検証や壊れたマークアップの修正は行いません。非推奨のマークアップ（`<center>`、`bgcolor`）も解析はしますが、報告も変更もしません。
- **再整形しません。** 改行を追加・削除するのは、レイアウト規則が必要とする場所だけです。行のインデントを直したり、テキストを行幅で折り返したりはしません。それはフォーマッターの役割です（[他のツールとの併用](./integration.md)）。
- **スプレッドは境界です。** 属性は `v-bind="object"` をまたいで移動しません（コンパイル時ディレクティブの `spreadSafe` グループを除く）。そのため、スプレッドを含むタグは一部が並べ替えられないまま残ることがあり、`vue/attributes-order` はそれを報告します（[属性順](../concepts/attribute-order.md)、[eslint-plugin-vue](./integration.md#eslint-plugin-vue)）。
- **同名の属性のまとまりは元の順を保ちます。** 例外は `class` で、静的形式をバインド形式より前に置きます。`style` などの重複は、順序で動作が変わりうるため書かれたままにします。

## レイアウト {#layout}

- **空要素は常に自己終了にします**（`<br />`、`<hr />`、`<wbr />`）。これは `vue/html-self-closing` の既定（`html.void: never`）と異なります。そのルール側で `html.void: always` にするか、プロファイルで `void: never` にしてください。
- **空要素以外の自己終了（`<div />`、`<MyComp />`）と `contentNewline` は既定で無効です。** `--self-closing` / `--content-newline` で有効にします（[レイアウト](../features/layout.md)）。`<div />` は Vue テンプレートでのみ適用します。素の HTML では閉じられていない開始タグになるためです。
- `contentNewline` は改行を追加しますが、コンテンツのインデントは直しません。暗黙に閉じられる要素（`</li>` のない `<li>`）には手を加えません。

## クラス順 {#class-order}

- **静的な `class` 属性だけが対象です。** バインドされた `:class`、`clsx()`/`cn()` の呼び出し、テンプレート構文を含む値は読みません。
- **組み込みは Tailwind だけです。** Tailwind CSS 4.3 の既定のテーマから生成しています。独自のテーマの値や `@utility` のクラスは認識しません。独自の色はテーマの色と同じように並び、未知のクラスは先頭に置かれます（[クラス順](../features/class-order.md)）。
- **既定では重複を残します。** `prettier-plugin-tailwindcss` と oxfmt は既定で削除します。削除するには `--remove-duplicate-classes` を指定してください。
- Bootstrap のクラス順はまだありません。

## スタイルシート {#stylesheets}

- **スタイルシートは並べ替えだけでなく、malva で整形されます。** malva のレイアウトは、ところどころ Prettier、oxfmt、Biome と異なります。たとえば、カンマ区切りの値（`transition`、`font-family`）を、それらが複数行に分けるところで 1 行のまま残します。この違いをなくす malva のオプションはないため、フォーマッターが書いた直後のファイルに `anyl --check` を実行すると失敗することがあります。annealer はフォーマッターより先に実行してください（[実行順](./integration.md#run-order)）。
- **CSS の文字列は既定でシングルクォートです**（`quotes: prefer-single`）。ダブルクォートで書くフォーマッターと併用する場合は、どちらかに合わせてください（[合わせておく設定](./integration.md#settings-to-align)）。
- ベンダープレフィックス付きの宣言は、`declarationOrder` を指定したときだけ移動します。宣言の間にコメントがある並びには手を加えません（[スタイルシート](../concepts/stylesheets.md)）。
- 静的な `style` 属性を並べ替えるのは、`stylesheet.styleAttribute` を有効にしたときだけです。テンプレート構文やコメントを含む値、単純な宣言として解析できない値は対象外です。バインドされた `:style` には手を加えません。

## プロファイルと CLI {#profiles-and-the-cli}

- **プロファイルの合成（`extends`）はできません。** 組み込みプロファイルの設定を 1 つ変えるだけでも、プロファイル全体（`crates/annealer/profiles/*.yaml` または `builtinProfile()`）をコピーして編集する必要があります。
- **設定ファイルはありません。** `--profile`、`--class-order`、`--self-closing`、`--print-width` などのオプションは毎回指定します（`package.json` のスクリプトにまとめてください）。
- **`anyl` はファイルのパスだけを受け取ります。** ディレクトリをたどる、glob を展開する、ignore ファイルを読む、といったことはしません。シェルや `git ls-files` からパスを渡してください。
- エディター拡張、言語サーバー、Prettier プラグインはありません。annealer は CLI、Git フック、JavaScript API から実行してください。
