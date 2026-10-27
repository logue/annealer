---
description: 差し替え可能なクラス順プロファイルで静的な class 属性内のクラスを並べ替える機能と、eslint-plugin-tailwindcss と同じ並び順になる組み込みの Tailwind CSS 順。
---

# クラス順

annealer は静的な `class` 属性内のクラスを並べ替えられます。並び順は**クラス順プロファイル**で決まります。これは属性プロファイルとは別の YAML ファイルなので、別のライブラリの並び順に差し替えられます。

既定では無効です。クラス順の名前かファイルを指定すると有効になります。

```bash
anyl --class-order tailwind src/**/*.vue
anyl --class-order ./my-classes.yaml index.html
```

```ts
format(source, { language: 'vue', classOrder: 'tailwind' });
```

属性プロファイルの [`classOrder`](../../api/profile.md#classorder) で指定することもできます。`--class-order` と `classOrder` オプションはそれより優先されます。

## 組み込み: `tailwind`

組み込みの `tailwind` は、`eslint-plugin-tailwindcss`（`classnames-order`）と `prettier-plugin-tailwindcss` が適用する並び順に従います。どちらもこの順序を Tailwind CSS のコンパイラから得ています。

<!-- prettier-ignore -->
```html
<!-- 入力 -->
<div class="p-3 border-gray-300 m-4 h-24 lg:p-4 flex border-2 lg:m-4 card"></div>

<!-- 出力 -->
<div class="card m-4 flex h-24 border-2 border-gray-300 p-3 lg:m-4 lg:p-4"></div>
```

annealer は Tailwind を実行しません。並び順はデータで、`scripts/generate-tailwind-order.mjs` が Tailwind CSS 4.3 の既定テーマから生成しています。同じスクリプトが、Tailwind 自身がソートした 3,000 件のクラスリストをテスト用に書き出しており、annealer の出力はそのすべてと一致します。

Tailwind を実行する場合との違い:

- **プロジェクト独自のテーマやユーティリティは認識しません。** 独自の色（`bg-brand-500`）はテーマの色と同じように並びます。他のどのルールにも一致しない値は、色のルールに当てはまるためです。独自のユーティリティ（`@utility`）は未知のクラスになります。
- **既定では重複を残します。** `eslint-plugin-tailwindcss` は既定で重複を削除します（`removeDuplicates`）。クラスの削除はリンターの役割なので、組み込みプロファイルでは残します。削除するには `--remove-duplicate-classes`（JavaScript では `removeDuplicateClasses: true`）を指定するか、`removeDuplicates: true` のクラス順を使います。このオプションはクラス順なしでも使えます。その場合、クラスは元の順のまま、重複だけが削除されます。
- **静的な `class` 属性のみ。** `:class` のバインドや `clsx()` のような呼び出しは式なので、annealer は読みません。

## 並び順の決まり方

1. どのルールにも一致しないクラス（`card`、`js-toggle`）を先頭に、元の順のまま置きます（`unknown: end` なら末尾）。
2. バリアントのないクラスが先です。バリアント付きのクラスは、そのバリアントの順位（`variants.order`）で並びます。ビットマスクの比較と同じく、最も高い順位で決まります。
3. 次に、クラスが設定する CSS プロパティで比べます。最初に異なるプロパティの、`propertyOrder` での位置順です。
4. 次に、プロパティの多い方が先です（`count`）。
5. 最後に名前順です。連続した数字は数値として比べます（`p-2` は `p-10` より前）。

これは Tailwind が使っている比較そのものです。Tailwind 固有の要素はなく、どのユーティリティライブラリも、クラスが設定するプロパティでクラスを記述できます。

## マークアップ上の変化

- 空白は元の位置に残ります。クラスが既存の区切りの間を移動するだけなので、複数行のクラスリストも改行を保ちます。
- テンプレート構文（`{{ }}`、`{% %}`、`<?php ?>`、`${}`）を含む値や、引用符のない値はそのままです。
- [無効化ディレクティブ](./disable-directives.md)は属性と同じく適用されます。

## クラス順を書く

```yaml
# yaml-language-server: $schema=https://github.com/logue/annealer/raw/refs/heads/main/crates/annealer/schema/class-order.schema.json
schemaVersion: 1
name: my-utilities
unknown: end
propertyOrder: [display, margin, padding, color]
values:
  size: [sm, md, lg]
rules:
  - match: [d-*]
    properties: [display]
  - match: ['m-{number}', 'm-{size}', mx-auto]
    properties: [margin]
  - match: ['p-{number}']
    properties: [padding]
  - match: [text-*]
    properties: [color]
```

- `match` のパターン: `*` は任意の文字列に、`{name}` は `values` の値セットのいずれかに一致します。`{number}`（10 進数）は組み込みです。
- 最も具体的なパターンが優先されます。完全一致、次に文字どおりの部分が長いもの、次に値セットの多いものです。
- `/` の後の修飾子（`bg-red-500/50`）は、修飾子を除いた方がよく一致する場合は無視されます。

すべてのキーは[クラス順リファレンス](../../api/profile.md#class-order-profiles)を参照してください。JavaScript API の `builtinClassOrder('tailwind')` で組み込みの YAML を取得し、出発点にできます。
