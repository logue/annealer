> For AI agents: the complete documentation index is available at /annealer/ja/llms.txt, the full documentation bundle is available at /annealer/ja/llms-full.txt.

# 設計判断

## アーキテクチャ

```text
annealer（コア）  純粋関数: format(input: &str, config: &Config)
                    -> Result<String, FormatError>
                  ファイル I/O なし。wasm32-unknown-unknown 向けにビルド可能。
                  CSS/SCSS/Sass/Less の宣言順には malva を使用。

anyl（CLI）       ファイル I/O と引数解析を担当し、コアを呼び出す。

annealer（npm）   コアの WebAssembly ビルドと TypeScript ラッパー。
```

- **クレートは 1 つ。** 現時点ではコアと CLI を 1 つのクレート（`crates/annealer`）に置きます。早すぎる抽象化を避けるため、実際に必要になった時点でワークスペースに分割します。CLI が使う `clap` は既定の `cli` フィーチャーの裏にあるため、`--no-default-features` でライブラリを WebAssembly 向けにビルドできます。
- **プロファイルはコードではなく YAML。** HTML、Vue、今後のプロファイルはすべてデータです。プロファイル形式は Rust の関数シグネチャと同じく公開 API なので、各プロファイルに `schemaVersion` を持たせます。
- **厳格な検証。** 未知のキーはプロファイルの解析時にエラーになり、タイプミスにすぐ気付けます。エディター支援のために JSON Schema（draft-07）を用意しています。draft-07 で表現できない 2 つの規則（フォールバックグループがちょうど 1 つ、グループ名が一意）はローダーでのみ検査します。
- **malva のバージョン指定は緩く**（`0.16`）。他のプロジェクトで採っているライブラリ依存の方針に合わせています。
- **npm パッケージは WebAssembly バイナリを埋め込みます。** 別の `.wasm` ファイルを同梱せず、base64 で埋め込みます。圧縮前のサイズが約 33% 増える代わりに、`format()` が同期関数になり、バンドラーやローダーの設定も不要になります。

## 実装中の判断

1. **HTML プロファイルの順序。** 当初の表には、どれにも一致しない属性や `alt`/`title` の置き場所がありませんでした。合意した順序は id → class → 補足（`alt`、`title`、`placeholder`）→ 意味（`type`、`href`、`src`、`srcset`）→ キーと値（`name`、`value`、`content`）→ 寸法（`width`、`height`）→ その他（状態: `disabled`、`checked` など）→ ARIA → data → イベント です。Vue プロファイルも「その他の属性」の階層内で同じ順序を使います。
2. **静的形式をバインド形式より前に置くのは `class` だけ。** その他の名前ではクラスタ内で書かれた順を保ちます。Vue は `style` を後の宣言が勝つ形でマージし、その他の重複では 1 つの値しか残さないためです。
3. **`spreadSafe` グループ。** コンパイル時ディレクティブは `v-bind` のスプレッドオブジェクトとマージされないため、スプレッドをまたいで移動してよいものとします。それ以外は自分のセグメント内に留まります。
4. **複数行のタグでは 1 行に 1 属性。** そうしないと、並べ替えで 1 行に 2 つの属性が並ぶことがあります。`vue/max-attributes-per-line` の複数行時の既定と同じで、`layout.oneAttributePerLine` で無効にできます。
5. **Vue の階層 8 を分割。** `vue/attributes-order` と同じく、その他のディレクティブ → その他の属性 の順にします（`OTHER_DIRECTIVES` → `OTHER_ATTR`）。
6. **`void: always` は `vue/html-self-closing` の既定（`html.void: never`）と異なります。** その ESLint ルールを残すプロジェクトでは、どちらかに合わせる必要があります。
7. **`contentNewline` は HTML プロファイルにも定義。** ブロックレベルのコンテンツの先頭と末尾の空白は表示されず、表示されうる要素は `ignore` リストで除外しています。
8. **ベンダープレフィックス付きの宣言は先頭に置く**（`vendorPrefix: start`）。標準プロパティが最後に宣言され、優先されるようにするためです。[スタイルシート](/annealer/ja/guide/concepts/stylesheets.md#vendor-prefixes-vendorprefix-start)を参照してください。
9. **空要素以外の自己終了と `contentNewline` はオプトイン。** 並べ替えに必要な範囲を超えてマークアップを書き換え、それらの eslint-plugin-vue ルールを使わない構成（Prettier など）と衝突するためです。組み込みプロファイルは設定値を `enabled: false` で保持し、`--self-closing` / `--content-newline` または対応する JavaScript オプションで有効にします。空要素（`<br />`）は Prettier と同じく常に自己終了にします。

## 解決済みの論点

- **`v-model` と通常の `value`**: クラスタにしません。キー `v-model` は `value` と異なります。
- **プロファイルの厳格さ**: 未知のキーは解析時にエラーになります。
- **malva のバージョン**: 緩い範囲指定 `0.16`。

## 残っている項目

- malva が raffia 0.13 に上がったら追従する（malva 0.16 はまだ raffia 0.12 を使用）。
- Svelte プロファイル
- 静的な `class` 属性の[値の並べ替え](/annealer/ja/guide/design/value-ordering.md)（Tailwind の近似順、Bootstrap への提案）
- [ライブラリプロファイル](/annealer/ja/guide/design/library-profiles.md)、プロファイルの合成（`extends`）、タグ・コンポーネント単位の上書き
- 手書きのフィクスチャに加えて、実際のプロジェクトから取ったフィクスチャ
