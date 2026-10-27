> For AI agents: the complete documentation index is available at /annealer/ja/llms.txt, the full documentation bundle is available at /annealer/ja/llms-full.txt.

# 他のツールとの併用

annealer が行うのは並べ替え（属性、クラス、宣言）だけで、行幅やインデントはフォーマッターに任せます。このページでは、よく使われるツールチェーンと組み合わせて、どちらのツールも相手の出力を打ち消さないようにする方法を示します。

例は次のバージョンで確認しました。

- [Prettier](https://prettier.io/) 3.9（`rs fmt` 経由）
- [oxfmt](https://oxc.rs/docs/guide/usage/formatter.html) 0.74
- [oxlint](https://oxc.rs/docs/guide/usage/linter.html) 1.89
- [eslint-plugin-vue](https://eslint.vuejs.org/) 10.11（[eslint-config-prettier](https://npm.im/eslint-config-prettier) 10.1 と併用）
- [Biome](https://biomejs.dev/) 2.6

## 実行順 \{#run-order}

annealer を先に、フォーマッターを後に実行します。

```bash
anyl --write src/App.vue   # 1. 並べ替え
rs fmt src/App.vue         # 2. 整形（Prettier、oxfmt、Biome など）
```

- これらのフォーマッターはどれも属性や宣言を並べ替えないので、annealer の順序はそのまま残ります。
- annealer はスタイルシートを malva で整形しますが、malva は一部の値を Prettier、oxfmt、Biome と異なるレイアウトで書きます（[制約事項](/annealer/ja/guide/start/limitations.md#stylesheets)を参照）。フォーマッターを最後に実行すればそのレイアウトが優先され、結果は安定します。もう一度両方を実行しても何も変わりません。
- フォーマッターが書いた直後のファイルに対して `anyl --check` を単独で実行すると、このレイアウトの違いのために失敗することがあります。CI で順序を検査するときは、パイプラインを実行して差分の有無を見てください。

```bash
git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write
rs fmt                  # または: oxfmt / biome format --write .
git diff --exit-code    # 順序か整形が崩れていれば失敗する
```

`anyl` はディレクトリをたどらず、glob も展開しません。上の `git ls-files` のように、ファイルのパスを渡してください。

## 合わせておく設定 \{#settings-to-align}

annealer の組み込みプロファイルは次の値を使います。フォーマッター側をこれに合わせると、`<style>` ブロックやスタイルシートが 2 つの書き方の間で行き来せず、フォーマッターによる差分も小さくなります。

| 項目                 | annealer（組み込みプロファイル）                 | Prettier / `rs fmt`              | oxfmt                | Biome                                            |
| ------------------ | ------------------------------------ | -------------------------------- | -------------------- | ------------------------------------------------ |
| CSS の文字列の引用符       | シングル（`quotes: prefer-single`）        | `singleQuote: true`              | `singleQuote: true`  | `css.formatter.quoteStyle: "single"`             |
| スタイルシートのインデント      | スペース 2 つ（malva の既定）                  | 既定のまま                            | 既定のまま                | `formatter.indentStyle: "space"`                 |
| スタイルシートの行幅         | 80（`--print-width` で変更）              | 既定のまま（80）                        | `--print-width 100`  | 既定のまま（80）                                        |
| 空要素                | `<br />`（常時）                         | 既定のまま                            | 既定のまま                | `html.formatter.selfCloseVoidElements: "always"` |
| 属性の並べ替え            | annealer                             | なし                               | なし                   | `useSortedAttributes` を無効（必須）                    |
| 宣言の並べ替え            | annealer（`declarationOrder: smacss`） | なし                               | なし                   | `useSortedProperties` を無効（必須）                    |
| Tailwind のクラスの並べ替え | `--class-order tailwind`（オプトイン）      | `prettier-plugin-tailwindcss` なし | `sortTailwindcss` なし | `useTailwindSortedClasses` を無効                   |

ダブルクォートを使いたい場合は、フォーマッターではなくプロファイルを変更します。組み込みプロファイル（`crates/annealer/profiles/*.yaml`）をコピーし、`stylesheet.malva.quotes: prefer-double` を指定してください。

### アルファベット順に並べ替えない \{#no-alphabetical-sorting}

annealer の目的は、属性や宣言を意味のある順に並べることです。識別子を先頭に、要素が何でありどう振る舞うかを表す属性、ARIA、データ属性、イベントの順に並べます。アルファベット順はその意味を捨ててしまうため、属性や宣言をアルファベット順に並べ替えるツールは annealer の考え方と相容れず、互いの出力を書き換え続けます。次の設定を無効にしてください。

| ツール               | 設定                                                                           |
| ----------------- | ---------------------------------------------------------------------------- |
| eslint-plugin-vue | `vue/attributes-order`: `alphabetical: false`（既定）のままにする                      |
| Biome             | `assist.actions.source.useSortedAttributes` と `useSortedProperties`: `off`   |
| Stylelint         | annealer が並べるファイルに `stylelint-order` の `properties-alphabetical-order` を使わない |

### Tailwind のクラス: 並べ替えはどれか 1 つで \{#tailwind-classes-pick-one-sorter}

annealer の `tailwind` クラス順は、`prettier-plugin-tailwindcss` や oxfmt の `sortTailwindcss` と同じく Tailwind 公式の順序に従います。順序は同じですが、次の違いがあります。

- これらのフォーマッターは、既定で重複したクラスを削除します。annealer は `--remove-duplicate-classes` を指定しない限り残します。
- これらは `clsx()`/`cn()` の呼び出しや、指定した属性（`:class`）の中も並べ替えられます。annealer が並べ替えるのは静的な `class` 属性だけです。

両方ではなく、どちらか一方を使ってください。フォーマッター側で並べ替える場合は `--class-order` を外します。oxfmt で重複を annealer と同じく残したい場合は `preserveDuplicates: true` を指定します。

## Rslint と `rs fmt`（Rstack） \{#rslint-and-rs-fmt-rstack}

Rslint は JavaScript と TypeScript を検査するリンターで、Vue テンプレートのルールはないため、annealer と重なる部分はありません。`rs fmt` は Prettier をもとにしているので、annealer をその前に実行します。

```ts title="rstack.config.ts"
import { define } from 'rstack';

define.lint(({ js, ts }) => [
  { ignores: ['dist/**'] },
  js.configs.recommended,
  ts.configs.recommendedTypeChecked
]);

define.fmt({
  // annealer は CSS の文字列をシングルクォートで書く。
  singleQuote: true
  // `anyl --class-order tailwind` を使う場合は `prettier-plugin-tailwindcss` を追加しない。
});

define.staged({
  // annealer が先、フォーマッターが後。
  '*.{vue,html,css,scss,less}': ['anyl --write', 'rs fmt'],
  '*.sass': 'anyl --write',
  '*.{js,jsx,ts,tsx,mjs,cjs,mts,cts}': ['rs lint --fix', 'rs fmt']
});
```

```json title="package.json"
{
  "scripts": {
    "format": "git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write && rs fmt"
  }
}
```

staged の glob には、annealer が対応する拡張子だけを含めてください。それ以外のファイルはエラー（`unsupported file type`）になります。

## eslint-plugin-vue \{#eslint-plugin-vue}

`vue/attributes-order`（`flat/recommended` に含まれる）は、annealer の `vue` プロファイルと同じ階層を検査するので、annealer の出力はこのルールを満たします。`alphabetical` は既定の `false` のままにしてください（[アルファベット順に並べ替えない](#no-alphabetical-sorting)を参照）。Prettier と併用する場合は、最後に `eslint-config-prettier` を追加します。レイアウトのルール（`vue/html-self-closing`、`vue/max-attributes-per-line` など）が無効になり、レイアウトは Prettier が決めます。

```js title="eslint.config.js"
import vue from 'eslint-plugin-vue';
import prettier from 'eslint-config-prettier';

export default [
  ...vue.configs['flat/recommended'],
  prettier,
  {
    rules: {
      // annealer は「その他の属性」の階層を意味で並べる。
      'vue/attributes-order': ['warn', { alphabetical: false }]
    }
  }
];
```

- **スプレッド:** Vue 3 では後に書いた方が優先されるため、annealer は `v-bind="object"` をまたいで属性を動かしません。`vue/attributes-order` はスプレッドをまたいだ順序も検査し、`--fix` では属性をまたいで動かします（`title="t" v-bind="obj"` が `v-bind="obj" title="t"` になり、`title` が `obj.title` を上書きするようになります）。このようなタグは手で並べ替えるか、`eslint-disable` コメントを付けてください。
- **annealer が書き換えないもの:** `vue/v-on-event-hyphenation`（`@myEvent` → `@my-event`）と `vue/v-slot-style`（`v-slot:item` → `#item`）。`eslint --fix` が書き換え、annealer はその結果に手を加えません。

annealer、Prettier、`eslint --fix` の順に実行します。その結果に対して `anyl --check` は通ります。

## oxlint と oxfmt（Oxc） \{#oxlint-and-oxfmt-oxc}

oxlint の `vue` プラグインが検査するのは `<script>` のコードだけで、`vue/attributes-order` のようなテンプレートのルールはありません。無効にする必要があるものはありません。

```json title=".oxlintrc.json"
{
  "$schema": "./node_modules/oxlint/configuration_schema.json",
  "plugins": ["typescript", "unicorn", "oxc", "vue"],
  "categories": { "correctness": "error" },
  "ignorePatterns": ["dist/**"]
}
```

oxfmt は `.vue` と `.html` を同梱の Prettier で、CSS/SCSS/Less をネイティブに整形します。既定値は引用符（ダブル）と行幅（80 ではなく 100）が annealer と異なります。引用符は oxfmt 側で設定します。

```json title=".oxfmtrc.json"
{
  "$schema": "./node_modules/oxfmt/configuration_schema.json",
  "singleQuote": true
}
```

行幅は oxfmt ではなく annealer 側を合わせます。`--print-width 100`（JavaScript では `printWidth: 100`）を指定すると、スタイルシートと `<style>` ブロックが oxfmt と同じ位置で折り返されます。`.oxfmtrc.json` で `printWidth` を指定している場合は、同じ値を `--print-width` に渡してください。annealer がクラスを並べ替える場合は `sortTailwindcss` を無効のままにします。oxfmt に並べ替えさせる場合は、有効にして `--class-order` を外します。

```json title=".oxfmtrc.json（oxfmt が Tailwind のクラスを並べ替える場合）"
{
  "singleQuote": true,
  "sortTailwindcss": {
    "stylesheet": "./src/app.css",
    "preserveDuplicates": true
  }
}
```

そのうえで、annealer を oxfmt より先に実行します。

```bash
git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write --print-width 100
oxfmt
```

## Biome

Biome の既定値は、4 つの点で annealer と衝突します。タブ、ダブルクォートの CSS 文字列、スラッシュのない `<br>`、そして Biome 自身の並べ替えアクションです。HTML の完全サポート（`.vue` ファイルに必要）を有効にしたうえで、次のように設定します。

```json title="biome.json"
{
  "$schema": "./node_modules/@biomejs/biome/configuration_schema.json",
  "formatter": { "indentStyle": "space", "indentWidth": 2 },
  "css": { "formatter": { "quoteStyle": "single" } },
  "html": {
    "experimentalFullSupportEnabled": true,
    "formatter": { "selfCloseVoidElements": "always" }
  },
  "assist": {
    "actions": {
      "source": {
        "useSortedAttributes": "off",
        "useSortedProperties": "off"
      }
    }
  },
  "linter": {
    "rules": { "nursery": { "useTailwindSortedClasses": "off" } }
  }
}
```

- **`useSortedAttributes` と `useSortedProperties` は無効にしてください。** `useSortedAttributes` は HTML と Vue の属性をアルファベット順に、`useSortedProperties` は CSS の宣言を独自の順に並べ替えます。どちらも annealer の意味による並べ替えと衝突し、`biome check --write` のたびに annealer の順序を崩します（[アルファベット順に並べ替えない](#no-alphabetical-sorting)を参照）。既定では無効ですが、共有設定などで有効にならないよう、上のように明示的に `off` を指定します。
- `useTailwindSortedClasses` は nursery のルールです。annealer がクラスを並べ替える場合は無効にします。
- タブを使い続ける場合は、`indentStyle` を変える代わりに、プロファイルで `stylesheet.malva.useTabs: true` を指定してください。

そのうえで、annealer を Biome より先に実行します。

```bash
git ls-files -z '*.vue' '*.html' '*.css' '*.scss' '*.less' | xargs -0 anyl --write
biome check --write .
```
