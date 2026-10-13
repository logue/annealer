> For AI agents: the complete documentation index is available at /annealer/ja/llms.txt, the full documentation bundle is available at /annealer/ja/llms-full.txt.

# 属性順

## 字句解析: DOM を構築しない

annealer は DOM や XML のツリーを構築しません。開始タグ（`<tagname` … `>` または `/>`）を見つけ、属性リストを名前と値の組に分割し、引用符の種類だけを記録します。

この方式は素の HTML でも、XML に準拠しない Vue の構文（`@click`、`#default`、`v-bind:[dynamic]`）でも同じように動きます。また、DOM や XML ベースの解析で起こる、空白のまとめ込みやタグ名・属性名の小文字化といった副作用も避けられます。

生テキスト要素、コメント、`{{ }}` の補間、SFC の `<script>` ブロック、カスタムブロック、HTML 以外の言語を指定した `<template lang="…">` は読み飛ばします。

## 分類

各属性は**正規化したキー**で分類します。

| 書き方                        | キー            |
| -------------------------- | ------------- |
| `foo`、`:foo`、`v-bind:foo`  | `foo`         |
| `@click`、`v-on:click`      | `v-on:click`  |
| `#item`、`v-slot:item`      | `v-slot:item` |
| `@click.stop`、`:foo.camel` | 修飾子を除去        |

プロファイルは、出力順に並んだ**グループ**のリストです。各グループは `match` パターン（完全一致の名前、または `*` で終わるプレフィックス）を持ちます。属性は、**最も具体的に**一致するパターンのグループに入ります。完全一致はどのプレフィックスよりも優先され、長いプレフィックスは短いものより優先されます。どれにも一致しない属性は、ただ 1 つの**フォールバック**グループに入ります。

グループ内の順序は `sort` で決まります。

- `source`: 書かれた順（既定）
- `alphabetical`: キーのアルファベット順
- `listed`: `match` パターンの並び順

## HTML プロファイル

| #  | グループ             | 属性                                                                                                                                                   | ソート          |
| -- | ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ |
| 1  | global           | `id`                                                                                                                                                 |              |
| 2  | class            | `class`                                                                                                                                              |              |
| 3  | supplementary    | `alt`、`title`、`placeholder`                                                                                                                          | listed       |
| 4  | semantic         | `type`、`href`、`src`、`srcset`                                                                                                                         | listed       |
| 5  | key-value        | `name`、`value`、`content`                                                                                                                             | listed       |
| 6  | dimensions       | `width`、`height`                                                                                                                                     | listed       |
| 7  | other（フォールバック）   | `disabled`、`checked`、`readonly` などの状態属性                                                                                                              | source       |
| 8  | aria-role        | `role`                                                                                                                                               |              |
| 9  | aria-name        | `aria-label`、`aria-labelledby`                                                                                                                       | listed       |
| 10 | aria-description | `aria-describedby`、`aria-description`、`aria-details`                                                                                                 | listed       |
| 11 | aria-property    | その他の `aria-*`                                                                                                                                        | source       |
| 12 | aria-state       | `aria-busy`、`aria-checked`、`aria-current`、`aria-disabled`、`aria-expanded`、`aria-grabbed`、`aria-hidden`、`aria-invalid`、`aria-pressed`、`aria-selected` | source       |
| 13 | data             | `data-*`                                                                                                                                             | alphabetical |
| 14 | events           | `on*`                                                                                                                                                | source       |

この順序にした理由:

- **補足属性は `id`/`class` の直後に置きます。** 人間の読み手に向けて要素を説明する属性だからです。`id` と `class` は慣用的に並べて書かれるため、先頭にまとめます。
- **ARIA は末尾近くに置きます。** `alt`/`title` では表現できない、支援技術向けの追加メタデータであり、要素の主たる説明ではないからです。
- **ARIA の中は WAI-ARIA に従います**: role → アクセシブルな名前 → 説明 → プロパティ → ステート
- **`data-*` はアルファベット順です。** 意味に基づく順序が存在しないためです。

## Vue プロファイル

Vue プロファイルは eslint-plugin-vue の `vue/attributes-order` の既定順に従うため、出力は常にそのルールを満たします。

| #  | 階層          | 属性                                             |
| -- | ----------- | ---------------------------------------------- |
| 1  | 定義          | `is`、`v-is`                                    |
| 2  | リストレンダリング   | `v-for`                                        |
| 3  | 条件          | `v-if`、`v-else-if`、`v-else`、`v-show`、`v-cloak` |
| 4  | レンダー修飾子     | `v-pre`、`v-once`                               |
| 5  | グローバル       | `id`                                           |
| 6  | 一意／スロット     | `ref`、`key`、`slot`、`v-slot*`                   |
| 7  | 双方向バインディング  | `v-model*`                                     |
| 8a | その他のディレクティブ | その他の `v-*`                                     |
| 8b | その他の属性      | HTML プロファイルの `class` から `data` までのグループ（上記）     |
| 9  | イベント        | `v-on*`（`@…`）                                  |
| 10 | コンテンツ       | `v-html`、`v-text`                              |

階層 8 は「その他のディレクティブ」→「その他の属性」の順に分かれています。これは `vue/attributes-order` の `OTHER_DIRECTIVES` → `OTHER_ATTR` に合わせたものです。HTML プロファイルの細分化は「その他の属性」の階層内で適用されます。

```vue
<!-- 入力 -->
<MyItem
  @click="select"
  :class="{ active }"
  class="item"
  v-for="item in items"
  :key="item.id"
  aria-label="Item"
  v-bind:title="item.name"
  :itemName="item.name"
/>

<!-- 出力 -->
<MyItem
  v-for="item in items"
  :key="item.id"
  class="item"
  :class="{ active }"
  :title="item.name"
  :item-name="item.name"
  aria-label="Item"
  @click="select"
/>
```

## 同名クラスタリング

分類は正規化したキーで行うため、同じ属性の静的形式とバインド形式は同じキーになり、まとまって並びます。キーが最初に現れた位置がそのクラスタの位置になります。特別な処理は必要ありません。

- **`class`** は静的形式をバインド形式より前に置きます（`class="item" :class="{ active }"`）。
- **その他の名前**はクラスタ内で書かれた順を保ちます。Vue は `style` を後の宣言が勝つ形でマージし、それ以外の重複した名前は一方しか残さないため、並べ替えると動作が変わるおそれがあるからです。
- **`v-model` は同名の通常属性とクラスタにしません**（`value="x" v-model="y"`）。キーは `value` ではなく `v-model` であり、双方向バインディングは別の概念なので独自の階層に残ります。

## `v-bind` スプレッド

`v-bind="object"` は境界として扱います。オブジェクトのマージ時にどの値が勝つかが変わるおそれがあるため、属性をスプレッドをまたいで移動しません。

例外は `spreadSafe` を指定したグループです。これはスプレッドオブジェクトとマージされることのないコンパイル時ディレクティブ（`is`、`v-for`、`v-if` など、`v-pre`/`v-once`、カスタムディレクティブ）なので、スプレッドをまたいで移動できます。それ以外はすべて自分のセグメント内に留まります。

## 正規化

Vue プロファイルでは 2 つの書き換えが有効で、分類の前に実行されます。

- `directiveShorthand`: `v-bind:foo` → `:foo`、`v-on:click` → `@click`
- `hyphenateComponentProps`: コンポーネントタグ上の camelCase の属性名 → kebab-case（`:itemName` → `:item-name`）。SVG や MathML のネイティブ要素は対象外です。
