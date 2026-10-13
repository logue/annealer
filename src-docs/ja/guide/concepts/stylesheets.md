---
description: annealer が malva を通じて CSS・SCSS・Sass・Less の宣言を並べ替える方法、ベンダープレフィックス付き宣言の位置、オプトインの style 属性対応。
---

# スタイルシート

annealer は CSS プロパティの並べ替えを自前で実装せず、設定可能な `declarationOrder` をすでに持つ [malva](https://github.com/g-plane/malva) に委譲します。プロファイルの `stylesheet` セクションでこれを制御します。

整形の対象:

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
- `declarationOrderGroupBy`: 宣言を、個別にソートする区間にどう分けるか。`non-declaration`（既定）または `non-declaration-and-empty-line`

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

## ベンダープレフィックス: `vendorPrefix: start` {#vendor-prefixes-vendorprefix-start}

ベンダープレフィックス付きの宣言（`-webkit-…`、`-moz-…`）は、標準プロパティとは別のものとして扱います。これらは方言であり、標準化されなかったものも多くあります。`start` を指定すると、同じ区間の他の宣言より**前**に、プロパティごとのグループ化をせず単純なアルファベット順で置きます。

`end` ではなく `start` にした理由: プレフィックス付きのプロパティは標準プロパティの別名であることが多く、後に書いた宣言が勝ちます。プレフィックス付きを最後に置くと、方言が標準の値を上書きしてしまうおそれがあります（例: 値を 2 つ取る `-webkit-border-radius`）。`start` なら Autoprefixer の出力と同じく、標準プロパティが最後になります。

malva にはこのオプションがありません（プレフィックスを除いた名前でソートします）。annealer は malva の出力を再解析し、プレフィックス付きの宣言だけを移動します。区間は malva のソート単位と一致します。宣言の間にコメントがある区間には手を加えません。このオプションは `declarationOrder` を設定したときだけ有効です。`alongside`（未指定時の既定）は malva の動作のままです。

## 静的な `style` 属性（オプトイン） {#static-style-attributes-opt-in}

`stylesheet.styleAttribute: true` にすると、静的な `style` 属性内の宣言も並べ替えます。どちらの組み込みプロファイルでも無効です。

- 値は 1 行のままです。
- CSS の文字列には、属性の引用符と反対の引用符を使います。
- 末尾の `;` がなければ、ないままにします。
- バインドされた `:style` には一切手を加えません。
- 引用符がない値、テンプレート構文・コメント・`{}`・`<`・`&` を含む値、通常の宣言として解析できない値は、書かれたとおりに残します。

## その他の malva オプション

その他の malva オプション（`printWidth`、`quotes`、`hexCase` など）は `stylesheet.malva` に書きます。未知のキーはエラーになるため、タイプミスが黙って無視されることはありません。[プロファイルリファレンス](../../api/profile.md#stylesheet)を参照してください。
