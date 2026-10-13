> For AI agents: the complete documentation index is available at /annealer/ja/llms.txt, the full documentation bundle is available at /annealer/ja/llms-full.txt.

# 値の並べ替え

:::info 状況
静的な `style` 属性は実装済みです（オプトイン。[スタイルシート](/annealer/ja/guide/concepts/stylesheets.md#static-style-attributes-opt-in)を参照）。`class` の値の並べ替えは**計画段階**で、まだ実装していません。このページには合意済みの原則をまとめます。
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
- **Tailwind**: 並び順はすでに Tailwind のエンジンが定義しています。annealer はその近似を YAML の組み込みプロファイルとして提供できます。

### プロファイル形式は公開された契約

サードパーティがプロファイルを書いて配布することを想定しているため、提案の前に `schemaVersion`、JSON Schema、形式のドキュメントを安定させる必要があります。annealer の外で配布されるプロファイルには、読み込む手段（たとえばパッケージに含まれるファイルを解決する `extends`）も必要です。

### 近似的で自己完結した順序

Tailwind の正式な順序は、各プロジェクトの Tailwind ビルド（v4 の `@theme` やカスタムユーティリティ、v3 の `tailwind.config.js`）から導かれるもので、Tailwind 自身にしか計算できません。そこで annealer は順序を YAML で定義します。

- 標準のユーティリティについては正式な順序に近い
- カスタムユーティリティは認識しない
- JavaScript ランタイムは不要

### 意味を変えない

クラスの並べ替えは CSS の意味を変えません。重複したクラスは残し（削除はリンターの役割です）、未知のクラスも決して取り除きません。

## 素案（設計中）

- `class` の値を空白で分割し、各トークンをグループパターンで分類し（属性グループと同じ考え方）、グループ順、次にグループの `sort` モードで並べます。同順位なら元の順を保ちます。
- バリアントとブレークポイントには別の軸が必要です。
  - Tailwind は**接頭辞**（`md:`、`hover:`、`dark:`）を使います。ソートキーはベースのクラスとバリアントの順序です。
  - Bootstrap は**中置**（`col-md-6`、`d-lg-none`、`mt-sm-3`）を使います。ブレークポイントは名前の途中から取り出します。
- 末尾の `*` だけでは足りません。たとえば Bootstrap の `text-center`（配置）と `text-primary`（色）は同じ接頭辞を持ちます。値のパターン構文は属性のパターンより表現力が必要です。

## 未解決の論点

- 未知のクラスをどこに置くか。Tailwind の Prettier プラグインはカスタムクラスを先頭に置きます。annealer では `start`/`end` オプションを用意するかもしれません。
- `class` の値の中の空白（複数行にわたるクラスリスト）を半角スペース 1 つに正規化するか、保持するか。
- プロファイルをどう組み合わせるか（たとえば `extends: vue` と `bootstrap` のクラス順）。これはライブラリ固有の属性ルールにも使えます。
