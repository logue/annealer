> For AI agents: the complete documentation index is available at /annealer/ja/llms.txt, the full documentation bundle is available at /annealer/ja/llms-full.txt.

# annealer

属性を意味の順に

> HTML・Vue の属性とスタイルシートの宣言を、差し替え可能な YAML プロファイルに従って意味に基づき並べ替えます。

[はじめに](/guide/start/introduction) | [GitHub](https://github.com/logue/annealer)

## Features

- [🧭 **意味に基づく属性順**](/guide/concepts/attribute-order): 属性をアルファベット順や入力順ではなく、役割（定義・状態・意味の補足・ARIA・データ・イベント）でグループ化します。
- [🧩 **vue/attributes-order の上位互換**](/guide/concepts/attribute-order): Vue プロファイルは eslint-plugin-vue の既定順を引き継ぎ、同名クラスタリング、ARIA の細分化、素の HTML、スタイルシートに対応します。
- [🎨 **malva によるスタイルシート整形**](/guide/concepts/stylesheets): CSS・SCSS・Sass・Less のプロパティ順は malva に委譲し、ベンダープレフィックス付きの宣言は先頭に置きます。
- [📄 **差し替え可能な YAML プロファイル**](/api/profile): 並び順やレイアウト規則はコードではなくデータです。JSON Schema で検証され、未知のキーはエラーになります。
- [🦀 **Rust コア・CLI・WebAssembly**](/api/): ファイル I/O を持たない純粋な Rust 関数、anyl CLI、同期的な format() を備えた npm パッケージを提供します。
- [🛑 **無効化ディレクティブ**](/guide/features/disable-directives): annealer-disable / annealer-enable / annealer-disable-next-line コメントで、指定したコードを書いたとおりに保ちます。
