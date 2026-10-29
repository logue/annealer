---
description: コマンドラインツール anyl のオプション、入力言語、終了コード。
---

# CLI

```text
anyl [OPTIONS] [FILES]...
```

各ファイルを整形して結果を出力します。ファイルを指定しなければ標準入力を読みます。

## オプション

| オプション                    | 説明                                                                                                                                                                                                                                                                                                                                  |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `-w`, `--write`               | ファイルをその場で上書きします。`--check` とは併用できません。                                                                                                                                                                                                                                                                        |
| `-c`, `--check`               | 変更が必要なファイルのパスを出力し、1 つでもあれば終了コード 1 で終了します。                                                                                                                                                                                                                                                         |
| `-p`, `--profile <PROFILE>`   | 組み込みプロファイル名（`html`、`vue`）またはプロファイル YAML ファイルのパス。既定は各ファイルの言語に対応する組み込みプロファイルです。                                                                                                                                                                                             |
| `--class-order <CLASS_ORDER>` | 静的な `class` 属性のクラスを並べ替えます。組み込みのクラス順の名前（`tailwind`）またはクラス順 YAML ファイルのパス。プロファイルの `classOrder` より優先されます。                                                                                                                                                                   |
| `--remove-duplicate-classes`  | 静的な `class` 属性で重複したクラスを、最初のものを残して削除します。`--class-order` の有無にかかわらず使えます。既定ではクラス順の `removeDuplicates` に従います（`tailwind` では無効）。`--keep-duplicate-classes` で残します。                                                                                                     |
| `--self-closing`              | Vue テンプレートの空要素と空のコンポーネントを自己終了にします（`layout.selfClosing.normal`/`component`、`vue/html-self-closing`）: `<div />`、`<MyComp />`。組み込みプロファイルでは無効です。`--no-self-closing` で無効にします。空要素（`<br />`）は `--void-self-closing` に従います。                                            |
| `--void-self-closing`         | 空要素を自己終了にします: `<input />`、`<meta />`（`layout.selfClosing.void: always`、組み込みプロファイルの設定）。`--no-void-self-closing` で `<input>`、`<meta>` にします（`never`）。                                                                                                                                             |
| `--expand-non-void`           | 素の HTML で、自己終了で書かれた空要素以外の要素に終了タグを付けます: `<div />` → `<div></div>`（`layout.selfClosing.expandNonVoid`）。HTML はスラッシュを無視するため、`<div />` は要素が閉じられません。`<svg>` と `<math>` の中の要素には手を加えません。組み込みプロファイルでは無効です。`--no-expand-non-void` で無効にします。 |
| `--content-newline`           | 複数行の要素のコンテンツを独立した行に置きます（`vue/multiline-html-element-content-newline`）。組み込みプロファイルでは無効です。`--no-content-newline` で無効にします。                                                                                                                                                             |
| `--print-width <PRINT_WIDTH>` | スタイルシートと `<style>` ブロックの行幅（`stylesheet.malva.printWidth`）。フォーマッターに合わせます。Prettier と Biome は 80、oxfmt は 100。既定はプロファイルの値（80）。                                                                                                                                                         |
| `-l`, `--language <LANGUAGE>` | 標準入力の言語: `html`（既定）、`vue`、`css`、`scss`、`sass`、`less`                                                                                                                                                                                                                                                                  |
| `-h`, `--help`                | ヘルプを表示します。                                                                                                                                                                                                                                                                                                                  |
| `-V`, `--version`             | バージョンを表示します。                                                                                                                                                                                                                                                                                                              |

## 言語

ファイルの言語は拡張子から判定します。

| 拡張子                            | 言語           | 既定のプロファイル                     |
| --------------------------------- | -------------- | -------------------------------------- |
| `.html`、`.htm`                   | `html`         | `html`                                 |
| `.vue`                            | `vue`          | `vue`                                  |
| `.css`、`.scss`、`.sass`、`.less` | スタイルシート | `html`（その `stylesheet` セクション） |

それ以外の拡張子（`.pug`、`.jade`、`.styl`、`.stylus` を含む）は、ファイルを読み込まずに未対応として報告します。Pug や Stylus の `<template>`・`<style>`（`lang="pug"`、`"jade"`、`"stylus"`、`"styl"`）を含むファイルもエラーになります。[制約事項](../guide/start/limitations.md#supported-input)を参照してください。

## 終了コード

| コード | 意味                                                                                                                                                     |
| ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `0`    | 成功。`--check` の場合は、変更が必要なファイルがない。                                                                                                   |
| `1`    | `--check` の場合に、変更が必要なファイルが 1 つ以上ある。                                                                                                |
| `2`    | エラー: ファイルを読めない、未対応のファイル形式、不正なプロファイル、不正な入力（Pug や Stylus のブロックを含む）、未知の `annealer-*` ディレクティブ。 |

複数のファイルを指定した場合は、すべてを処理してエラーを個別に報告し、1 つでも失敗があれば `2` で終了します。

## 例

```bash
anyl src/App.vue                          # 結果を出力
anyl --write src/**/*.vue                 # その場で上書き
anyl --check index.html                   # CI で使う
anyl --profile my-profile.yaml page.html  # 独自プロファイル
anyl --class-order tailwind src/**/*.vue  # Tailwind のクラスを並べ替える
anyl --class-order tailwind --remove-duplicate-classes src/**/*.vue  # 重複も削除する
anyl --print-width 100 src/**/*.vue  # oxfmt と同じく 100 で折り返す
anyl --self-closing --content-newline src/**/*.vue  # オプションのレイアウト規則も適用する
cat App.vue | anyl --language vue         # 標準入力
```
