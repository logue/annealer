> For AI agents: the complete documentation index is available at /annealer/ja/llms.txt, the full documentation bundle is available at /annealer/ja/llms-full.txt.

# スタイルシート

annealer は CSS プロパティの順序を自前で定義せず、[malva](https://github.com/g-plane/malva) の `declarationOrder` の順序を使います。プロファイルの `stylesheet` セクションでこれを制御します。

**変わるのは順序だけです。** 宣言は書かれたとおりのまま移動します。空白・インデント・引用符・色の大文字小文字・数値・コメント・空行には手を加えません。整形は Prettier や Oxfmt などのフォーマッターの役割です。malva 自体はフォーマッターですが、annealer はプロパティ名の並び順を求めるためだけに使い、malva の出力は使いません。

対象:

- HTML と Vue の `<style>` ブロック（`<style lang="scss">`、`lang="less"`、`lang="sass"` を含む）
- 単体の `.css`、`.scss`、`.sass`（インデント構文）、`.less` ファイル

プロファイルに `stylesheet` セクションがなければ、スタイルシートには手を加えません。

## 宣言順

どちらの組み込みプロファイルも次の設定です。

```yaml
stylesheet:
  declarationOrder: smacss
  vendorPrefix: start
```

- `declarationOrder`: `smacss`（重要でレイアウトの流れに影響するプロパティが先）、`concentric`（ボックスモデルの外側から内側へ）、`alphabetical` のいずれか
- `declarationOrderGroupBy`: 宣言を、個別にソートする区間にどう分けるか。`non-declaration-and-empty-line`（既定。空行でも区間を区切る）または `non-declaration`（空行をまたいでソートし、空行はその位置に残る）

宣言は、他の文（ネストしたルール、`@include`、Less のミックスイン呼び出しなど）やコメントをまたいで移動しません。そのため、コメントとそれが見出しになっている宣言のまとまりは崩れません。[無効化ディレクティブ](/annealer/ja/guide/features/disable-directives.md)の対象になっている宣言や、ネストしたブロックを持つ宣言（Sass のネストしたプロパティ）もその位置に留まり、区間を区切ります。

```css
/* 入力 */
body {
  color: #222;
  margin: 0;
  font-family: system-ui, sans-serif;
  display: flex;
}

/* 出力（smacss） */
body {
  display: flex;
  margin: 0;
  color: #222;
  font-family: system-ui, sans-serif;
}
```

## ベンダープレフィックス: `vendorPrefix: start` \{#vendor-prefixes-vendorprefix-start}

ベンダープレフィックス付きの宣言（`-webkit-…`、`-moz-…`）は、標準プロパティとは別のものとして扱います。これらは方言であり、標準化されなかったものも多くあります。`start` を指定すると、同じ区間の他の宣言より**前**に、プロパティごとのグループ化をせず単純なアルファベット順で置きます。

`end` ではなく `start` にした理由: プレフィックス付きのプロパティは標準プロパティの別名であることが多く、後に書いた宣言が勝ちます。プレフィックス付きを最後に置くと、方言が標準の値を上書きしてしまうおそれがあります（例: 値を 2 つ取る `-webkit-border-radius`）。`start` なら Autoprefixer の出力と同じく、標準プロパティが最後になります。

malva にはこのオプションがありません（プレフィックスを除いた名前でソートします）。そのため annealer が同じ区間の中でプレフィックス付きの宣言を移動します。このオプションは `declarationOrder` を設定したときだけ有効です。`alongside`（未指定時の既定）は malva の順序のままです。

## 静的な `style` 属性（オプトイン） \{#static-style-attributes-opt-in}

`stylesheet.styleAttribute: true` にすると、静的な `style` 属性内の宣言も並べ替えます。どちらの組み込みプロファイルでも無効です。

- 宣言は書かれたとおりのまま移動します。空白や末尾の `;` の有無は変わりません。
- バインドされた `:style` には一切手を加えません。
- 引用符がない値、テンプレート構文・`{}`・`<`・`&` を含む値、通常の宣言として解析できない値は、書かれたとおりに残します。
