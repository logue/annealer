> For AI agents: the complete documentation index is available at /annealer/ja/llms.txt, the full documentation bundle is available at /annealer/ja/llms-full.txt.

# 値の並べ替え

:::info 状況
どちらも実装済みです。静的な `style` 属性（オプトイン。[スタイルシート](/annealer/ja/guide/concepts/stylesheets.md#static-style-attributes-opt-in)を参照）と、差し替え可能なクラス順による `class` の値（[クラス順](/annealer/ja/guide/features/class-order.md)を参照）で、組み込みの `tailwind` 順があります。Bootstrap のプロファイルはまだ書いていません。このページには、その背景にある原則をまとめます。
:::

属性の並び順が 1 つの軸だとすれば、属性内の値の並び順はもう 1 つの軸です。これは Tailwind に限った需要ではありません。クラスベースの CSS フレームワーク（Bootstrap、Tailwind、プロジェクト独自のユーティリティ群）なら、属性と同じく差分を安定させるという理由で、決まったクラス順の恩恵を受けます。

## 原則

### 静的な値のみ

バインドされた属性（`:class`、`:style`、`v-bind:*`）が持つのは値ではなく式です。annealer はその中身を一切見ません。

これは、`:class` の式や `clsx()` のような呼び出しの中の文字列リテラルまでソートする `eslint-plugin-tailwindcss`（`classnames-order`）や `prettier-plugin-tailwindcss` とは異なる点です。

### 仕組みはライブラリ非依存、プロファイルはライブラリ固有

クラス順は、属性グループと同じくプロファイルのデータであり、コードの分岐ではありません。これが[ライブラリプロファイル](/annealer/ja/guide/design/library-profiles.md)の最初の具体例になります。

### クラス順を決めるのはライブラリ

フレームワークの正式なクラス順は、annealer ではなくそのフレームワークが決める設計事項です。annealer は仕組みとプロファイル形式を提供します。

- **Bootstrap**: 下書きのプロファイルを添えて、仕組みを上流に提案します（Bootstrap 6 は開発中）。並び順は Bootstrap 自身が定義し、保守します。
- **Tailwind**: 並び順はすでに Tailwind のエンジンが定義しています。annealer はそれを組み込みの YAML（`tailwind`）として提供します。Tailwind から生成し、Tailwind の結果と照合しています。`eslint-plugin-tailwindcss` の `classnames-order` の仕様に従いますが、重複は残します（下記参照）。

### プロファイル形式は公開された契約

サードパーティがプロファイルを書いて配布することを想定しているため、提案の前に `schemaVersion`、JSON Schema、形式のドキュメントを安定させる必要があります。annealer の外で配布されるプロファイルには、読み込む手段（たとえばパッケージに含まれるファイルを解決する `extends`）も必要です。

### 近似的で自己完結した順序

Tailwind の正式な順序は、各プロジェクトの Tailwind ビルド（v4 の `@theme` やカスタムユーティリティ、v3 の `tailwind.config.js`）から導かれるもので、Tailwind 自身にしか計算できません。そこで annealer は順序を YAML で定義します。

- 標準のユーティリティについては正式な順序に近い
- カスタムユーティリティは認識しない
- JavaScript ランタイムは不要

### 意味を変えない

クラスの並べ替えは CSS の意味を変えません。重複したクラスは残し（削除はリンターの役割です）、未知のクラスも決して取り除きません。

## 設計

- **クラスは、設定する CSS プロパティで順位付けします。** 比較は Tailwind と同じで、未知のクラス、バリアント、プロパティ順で最初に異なるプロパティ、プロパティの数、名前の順です。プロパティはライブラリに依存しないので、Bootstrap のプロファイルも同じ形式で書けます（`d-flex` は `display`、`mt-3` は `margin-top` を設定する）。
- **バリアント**（Tailwind の `md:`、`hover:`）は別の軸です。区切り文字と、順位付きのバリアントパターンのリストで表します。
- **パターン**は属性のパターンより表現力があります。任意の位置の `*` と値セット `{name}` を使え、最も具体的な一致が優先されます。`text-center`、`text-lg`、`text-red-500` はそれぞれ別のルールに解決されます。
- **クラス順は独立したプロファイルです。** 属性プロファイルの一部ではないので、どの属性プロファイルとも組み合わせられます（`--class-order`、`classOrder` オプション、プロファイルの `classOrder`）。

## 解決済みの論点

- 未知のクラスの置き場所: `unknown: start`（Tailwind の順）または `end`。
- `class` の値の中の空白は保持します。`eslint-plugin-tailwindcss` と同じく、クラスが既存の区切りの間を移動します。

## 未解決の論点

- 中置のブレークポイント（Bootstrap の `col-md-6`、`d-lg-none`）はまだ独立した軸ではありません。プロファイルではパターンでしか順位付けできません。
- 属性プロファイルをどう組み合わせるか（たとえば `extends: vue`）。これはライブラリ固有の属性ルールにも使えます。
