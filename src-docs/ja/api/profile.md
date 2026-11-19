---
description: annealer のプロファイル YAML 形式のリファレンス。すべてのキーの型と既定値。
---

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
    match: [on*]

stylesheet:
  declarationOrder: smacss
  vendorPrefix: start
```

:::tip エディター支援
JSON Schema（draft-07）は `crates/annealer/schema/profile.schema.json` にあります。上のモードラインと [YAML 拡張機能](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-yaml)を使えば、VS Code でプロファイルの検証と補完ができます。
:::

プロファイルのどこでも未知のキーはエラーになるため、タイプミスが見過ごされることはありません。

## トップレベル

| キー            | 型                     | 必須 | 説明                                                                |
| --------------- | ---------------------- | ---- | ------------------------------------------------------------------- |
| `schemaVersion` | `1`                    | ○    | プロファイル形式のバージョン                                        |
| `name`          | 文字列                 | ○    | プロファイル名                                                      |
| `description`   | 文字列                 |      | 人間向けの概要                                                      |
| `extends`       | 文字列 \| 文字列[]     |      | 土台にするプロファイル。[`extends`](#extends) を参照                |
| `normalize`     | オブジェクト           |      | 属性名の書き換え                                                    |
| `layout`        | オブジェクト           |      | 開始タグのレイアウト規則                                            |
| `groups`        | 配列                   | ○¹   | 出力順に並べた属性グループ                                          |
| `stylesheet`    | オブジェクト           |      | スタイルシートの整形。省略するとスタイルシートには手を加えません。  |
| `classOrder`    | 文字列 \| オブジェクト |      | 静的な `class` 属性のクラス順。省略するとクラスは元の順のままです。 |

¹ `extends` があるときは省略できます。マージ後のプロファイルには必要です。

## `extends` {#extends}

ほかのプロファイルを土台にします。組み込みプロファイルの名前（`html`、`vue`）、継承する側のファイルからの相対パス（CLI のみ）、またはそれらのリストを指定します。

```yaml
schemaVersion: 1
name: my-html
extends: [html, ./bootstrap-data.yaml]
```

```yaml
# bootstrap-data.yaml: ミックスインとして使う部分的なプロファイル
schemaVersion: 1
name: bootstrap-data
groups:
  - name: bootstrap
    match: [data-bs-toggle, data-bs-target, data-bs-*]
    sort: listed
    before: data
```

プロファイルは順にマージされるので、**後に書いたものが優先**されます。`extends` の各項目はそれより前の項目を上書きし、プロファイル自身はそのすべてを上書きします。

- **マッピング**（`normalize`、`layout`、`stylesheet` など）はキーごとにマージします。
- **それ以外の値**（スカラー、リスト、`classOrder`）は置き換えます。
- **`groups` は `name` でマージします。** 継承したグループと同じ名前のグループは、そのグループをキーごとに更新します。`match` は追加ではなく置き換えで、位置はそのままです。新しいグループは末尾に追加します。
- **`before` / `after`** には、新しいグループや継承したグループを隣に置く先のグループ名を指定します。それより前にマージされたどのプロファイルのグループでも指定できるので、ミックスインは前に書かれたプロファイルのグループの間に自分のグループを置けます。

継承の連鎖は先に平坦化されます。自身も `extends` を持つ土台は、その土台を自分の前に展開します。検証はマージ結果に対して 1 回だけ行うので、土台は部分的（`groups` やフォールバックのグループがない）でも構いません。直接・間接を問わず自分自身を継承するとエラーになります。

JavaScript API にはファイルシステムがないため、`extends` には組み込みプロファイルしか指定できません。

## `normalize`

分類の前に適用する書き換えです。

| キー                      | 型     | 既定    | 説明                                                                                            |
| ------------------------- | ------ | ------- | ----------------------------------------------------------------------------------------------- |
| `directiveShorthand`      | 真偽値 | `false` | `v-bind:foo` → `:foo`、`v-on:click` → `@click`                                                  |
| `hyphenateComponentProps` | 真偽値 | `false` | コンポーネントタグ上の camelCase の属性名 → kebab-case。SVG/MathML のネイティブ要素は除きます。 |

## `layout`

詳しくは[レイアウト](../guide/features/layout.md)を参照してください。

| キー                             | 型                                | 既定                              | 説明                                                                                     |
| -------------------------------- | --------------------------------- | --------------------------------- | ---------------------------------------------------------------------------------------- |
| `closingBracketNewline`          | 真偽値                            | `false`                           | 複数行の開始タグの閉じ括弧を独立した行に置く                                             |
| `oneAttributePerLine`            | 真偽値                            | `false`                           | 複数行の開始タグで 1 行に 1 属性                                                         |
| `selfClosing.void`               | `always` \| `never` \| `preserve` | `preserve`                        | 空要素: `<br />` または `<br>`                                                           |
| `selfClosing.normal`             | `always` \| `never` \| `preserve` | `preserve`                        | 空の非空要素（`<div />`）。Vue テンプレートのみ                                          |
| `selfClosing.component`          | `always` \| `never` \| `preserve` | `preserve`                        | 空のコンポーネント（`<MyComp />`）。Vue テンプレートのみ                                 |
| `contentNewline`                 | オブジェクト                      | 無効                              | 複数行の要素のコンテンツを独立した行に置く。`{}` で既定値のまま有効                      |
| `contentNewline.ignore`          | 文字列の配列                      | `pre`、`textarea`、インライン要素 | 中身も含めて手を加えない要素。要素名は大文字小文字を区別せず、コンポーネント名は完全一致 |
| `contentNewline.allowEmptyLines` | 真偽値                            | `false`                           | コンテンツの前後の空行を残す                                                             |

## `groups`

グループは出力順に並べます。フォールバックのグループはちょうど 1 つで、グループ名は一意でなければなりません。

| キー         | 型                                     | 既定     | 説明                                                                                                    |
| ------------ | -------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------- |
| `name`       | 文字列                                 | –        | 一意なグループ名。必須                                                                                  |
| `match`      | 文字列の配列                           | –        | パターン: 完全一致の名前、または `*` で終わるプレフィックス。`fallback: true` 以外では必須              |
| `sort`       | `source` \| `alphabetical` \| `listed` | `source` | グループ内の順序: 書かれた順、キー順、`match` の順                                                      |
| `fallback`   | 真偽値                                 | `false`  | どのパターンにも一致しない属性を受け取る                                                                |
| `spreadSafe` | 真偽値                                 | `false`  | このグループの属性は `v-bind="object"` のスプレッドをまたいで移動できる。コンパイル時ディレクティブ専用 |
| `before`     | 文字列                                 | –        | このグループの直前に置く。[`extends`](#extends) を参照                                                  |
| `after`      | 文字列                                 | –        | このグループの直後に置く。[`extends`](#extends) を参照                                                  |

パターンは**正規化して小文字にしたキー**と照合します。`foo`/`:foo`/`v-bind:foo` は `foo`、`@click` は `v-on:click`、`#name` は `v-slot:name` になり、修飾子は除去されます。完全一致はどのプレフィックスよりも優先され、長いプレフィックスは短いものより優先されます。[属性順](../guide/concepts/attribute-order.md)を参照してください。

## `stylesheet`

詳しくは[スタイルシート](../guide/concepts/stylesheets.md)を参照してください。

| キー                      | 型                                                    | 既定                             | 説明                                                                                                                                     |
| ------------------------- | ----------------------------------------------------- | -------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `declarationOrder`        | `smacss` \| `concentric` \| `alphabetical`            | –                                | プロパティの順序。指定しなければ宣言を並べ替えません。                                                                                   |
| `declarationOrderGroupBy` | `non-declaration` \| `non-declaration-and-empty-line` | `non-declaration-and-empty-line` | 宣言を、個別にソートする区間にどう分けるか。既定では空行で区間を区切る。`non-declaration` は空行をまたいでソートし、空行はその位置に残る |
| `vendorPrefix`            | `alongside` \| `start`                                | `alongside`                      | `start` にすると、ベンダープレフィックス付きの宣言を各区間の先頭にアルファベット順で置く                                                 |
| `styleAttribute`          | 真偽値                                                | `false`                          | 静的な `style` 属性内の宣言も並べ替える                                                                                                  |

変わるのは宣言の順序だけで、空白や引用符などの整形はフォーマッターに任せます。そのため malva の整形オプションは受け付けません。

## `classOrder`

静的な `class` 属性内のクラスの並び順です（[クラス順](../guide/features/class-order.md)を参照）。組み込みのクラス順の名前（`tailwind`、`bootstrap`）か、インラインで書いた[クラス順プロファイル](#class-order-profiles)を指定します。CLI の `--class-order` と JavaScript の `classOrder` オプションはこれより優先されます。

```yaml
classOrder: tailwind
```

## クラス順プロファイル {#class-order-profiles}

クラス順プロファイルは独立した YAML ファイルで、`--class-order` や `classOrder` で渡すか、プロファイルの `classOrder` の下にインラインで書きます。JSON Schema は `crates/annealer/schema/class-order.schema.json`、組み込みのものは [`crates/annealer/class-orders/tailwind.yaml`](https://github.com/logue/annealer/tree/main/crates/annealer/class-orders) です。

| キー                  | 型                       | 既定    | 説明                                                                                                    |
| --------------------- | ------------------------ | ------- | ------------------------------------------------------------------------------------------------------- |
| `schemaVersion`       | `1`                      | –       | 必須。形式のバージョン                                                                                  |
| `name`                | 文字列                   | –       | 必須。クラス順の名前                                                                                    |
| `description`         | 文字列                   | –       | 人間向けの概要                                                                                          |
| `unknown`             | `start` \| `end`         | `start` | どのルールにも一致しないクラスの置き場所（元の順を保つ）                                                |
| `removeDuplicates`    | 真偽値                   | `false` | 重複したクラスを、最初のものを残して削除する                                                            |
| `important`           | 文字列                   | –       | 照合の前にユーティリティの先頭か末尾から取り除く印（`!`）                                               |
| `arbitraryProperties` | 真偽値                   | `false` | `[property:value]` 形式のクラスを、`property` を設定するクラスとして扱う                                |
| `variants`            | オブジェクト             | –       | `separator`（例: `:`）と `order`（バリアントのパターンを順位の低い順に）。リストの項目は同じ順位になる  |
| `propertyOrder`       | 文字列の配列             | –       | 必須。CSS プロパティの出力順                                                                            |
| `values`              | パターンのリストのマップ | –       | パターン内の `{name}` 用の値セット。`number` は組み込み                                                 |
| `rules`               | 配列                     | –       | 必須。`match`（パターン）、`properties`（`propertyOrder` のもの）、任意の `count`（異なる場合の宣言数） |

どのパターンにも一致しないバリアントを持つクラスは未知のクラスです。ルールに書けるのは `propertyOrder` にあるプロパティだけで、それ以外はエラーになります。
