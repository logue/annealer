> For AI agents: the complete documentation index is available at /annealer/ja/llms.txt, the full documentation bundle is available at /annealer/ja/llms-full.txt.

# プロファイルリファレンス

プロファイルは、属性順、レイアウト規則、スタイルシートのオプションを定める YAML ファイルです。組み込みプロファイルは [`crates/annealer/profiles/`](https://github.com/logue/annealer/tree/main/crates/annealer/profiles) にあります。

```yaml
# yaml-language-server: $schema=https://github.com/logue/annealer/raw/refs/heads/main/crates/annealer/schema/profile.schema.json
schemaVersion: 1
name: my-profile
description: 独自の属性順

normalize:
  directiveShorthand: true

layout:
  closingBracketNewline: true
  oneAttributePerLine: true
  selfClosing:
    void: always
  contentNewline: {}

groups:
  - name: global
    match: [id]
  - name: class
    match: [class]
  - name: other
    fallback: true
  - name: events
    match: ['on*']

stylesheet:
  declarationOrder: smacss
  vendorPrefix: start
```

:::tip エディター支援
JSON Schema（draft-07）は `crates/annealer/schema/profile.schema.json` にあります。上のモードラインと [YAML 拡張機能](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-yaml)を使えば、VS Code でプロファイルの検証と補完ができます。
:::

プロファイルのどこでも未知のキーはエラーになるため、タイプミスが見過ごされることはありません。

## トップレベル

| キー              | 型      | 必須 | 説明                                |
| --------------- | ------ | -- | --------------------------------- |
| `schemaVersion` | `1`    | ○  | プロファイル形式のバージョン                    |
| `name`          | 文字列    | ○  | プロファイル名                           |
| `description`   | 文字列    |    | 人間向けの概要                           |
| `normalize`     | オブジェクト |    | 属性名の書き換え                          |
| `layout`        | オブジェクト |    | 開始タグのレイアウト規則                      |
| `groups`        | 配列     | ○  | 出力順に並べた属性グループ                     |
| `stylesheet`    | オブジェクト |    | スタイルシートの整形。省略するとスタイルシートには手を加えません。 |

## `normalize`

分類の前に適用する書き換えです。

| キー                        | 型   | 既定      | 説明                                                                |
| ------------------------- | --- | ------- | ----------------------------------------------------------------- |
| `directiveShorthand`      | 真偽値 | `false` | `v-bind:foo` → `:foo`、`v-on:click` → `@click`                     |
| `hyphenateComponentProps` | 真偽値 | `false` | コンポーネントタグ上の camelCase の属性名 → kebab-case。SVG/MathML のネイティブ要素は除きます。 |

## `layout`

詳しくは[レイアウト](/annealer/ja/guide/features/layout.md)を参照してください。

| キー                               | 型                                 | 既定                       | 説明                                           |
| -------------------------------- | --------------------------------- | ------------------------ | -------------------------------------------- |
| `closingBracketNewline`          | 真偽値                               | `false`                  | 複数行の開始タグの閉じ括弧を独立した行に置く                       |
| `oneAttributePerLine`            | 真偽値                               | `false`                  | 複数行の開始タグで 1 行に 1 属性                          |
| `selfClosing.void`               | `always` \| `never` \| `preserve` | `preserve`               | 空要素: `<br />` または `<br>`                     |
| `selfClosing.normal`             | `always` \| `never` \| `preserve` | `preserve`               | 空の非空要素（`<div />`）。Vue テンプレートのみ               |
| `selfClosing.component`          | `always` \| `never` \| `preserve` | `preserve`               | 空のコンポーネント（`<MyComp />`）。Vue テンプレートのみ         |
| `contentNewline`                 | オブジェクト                            | 無効                       | 複数行の要素のコンテンツを独立した行に置く。`{}` で既定値のまま有効         |
| `contentNewline.ignore`          | 文字列の配列                            | `pre`、`textarea`、インライン要素 | 中身も含めて手を加えない要素。要素名は大文字小文字を区別せず、コンポーネント名は完全一致 |
| `contentNewline.allowEmptyLines` | 真偽値                               | `false`                  | コンテンツの前後の空行を残す                               |

## `groups`

グループは出力順に並べます。フォールバックのグループはちょうど 1 つで、グループ名は一意でなければなりません。

| キー           | 型                                      | 既定       | 説明                                                            |
| ------------ | -------------------------------------- | -------- | ------------------------------------------------------------- |
| `name`       | 文字列                                    | –        | 一意なグループ名。必須                                                   |
| `match`      | 文字列の配列                                 | –        | パターン: 完全一致の名前、または `*` で終わるプレフィックス。`fallback: true` 以外では必須     |
| `sort`       | `source` \| `alphabetical` \| `listed` | `source` | グループ内の順序: 書かれた順、キー順、`match` の順                                |
| `fallback`   | 真偽値                                    | `false`  | どのパターンにも一致しない属性を受け取る                                          |
| `spreadSafe` | 真偽値                                    | `false`  | このグループの属性は `v-bind="object"` のスプレッドをまたいで移動できる。コンパイル時ディレクティブ専用 |

パターンは**正規化して小文字にしたキー**と照合します。`foo`/`:foo`/`v-bind:foo` は `foo`、`@click` は `v-on:click`、`#name` は `v-slot:name` になり、修飾子は除去されます。完全一致はどのプレフィックスよりも優先され、長いプレフィックスは短いものより優先されます。[属性順](/annealer/ja/guide/concepts/attribute-order.md)を参照してください。

## `stylesheet`

詳しくは[スタイルシート](/annealer/ja/guide/concepts/stylesheets.md)を参照してください。

| キー                        | 型                                                     | 既定                | 説明                                                            |
| ------------------------- | ----------------------------------------------------- | ----------------- | ------------------------------------------------------------- |
| `declarationOrder`        | `smacss` \| `concentric` \| `alphabetical`            | –                 | プロパティの順序。指定しなければ宣言を並べ替えません。                                   |
| `declarationOrderGroupBy` | `non-declaration` \| `non-declaration-and-empty-line` | `non-declaration` | 宣言を、個別にソートする区間にどう分けるか                                         |
| `vendorPrefix`            | `alongside` \| `start`                                | `alongside`       | `start` にすると、ベンダープレフィックス付きの宣言を各区間の先頭にアルファベット順で置く              |
| `styleAttribute`          | 真偽値                                                   | `false`           | 静的な `style` 属性内の宣言も並べ替える                                      |
| `malva`                   | オブジェクト                                                | –                 | その他の [malva のオプション](https://malva.netlify.app/config/)をそのまま渡す |

`stylesheet.malva` には、`printWidth`、`useTabs`、`indentWidth`、`lineBreak`、`hexCase`、`hexColorLength`、`quotes`、`trailingComma`、`formatComments`、`*.preferSingleLine` 系など malva 自身のオプションを書けます。`declarationOrder` と `declarationOrderGroupBy` は `malva` ではなく `stylesheet` 直下に書きます。未知の malva のキーはエラーになります。
