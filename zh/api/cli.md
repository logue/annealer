> For AI agents: the complete documentation index is available at /annealer/zh/llms.txt, the full documentation bundle is available at /annealer/zh/llms-full.txt.

# CLI

```text
anyl [OPTIONS] [FILES]...
```

格式化每个文件并输出结果。未指定文件时从标准输入读取。

## 选项

| 选项                            | 说明                                                                                                                                                                                                  |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `-w`, `--write`               | 直接覆盖文件。不能与 `--check` 同时使用。                                                                                                                                                                          |
| `-c`, `--check`               | 输出会被修改的文件路径；只要有一个文件会被修改，就以状态码 1 退出。                                                                                                                                                                 |
| `-p`, `--profile <PROFILE>`   | 内置配置文件名（`html`、`vue`）或配置文件 YAML 的路径。默认使用与各文件语言对应的内置配置文件。                                                                                                                                            |
| `--class-order <CLASS_ORDER>` | 对静态 `class` 属性中的类进行排序：内置类顺序名（`tailwind`）或类顺序 YAML 文件的路径。会覆盖配置文件中的 `classOrder`。                                                                                                                     |
| `--remove-duplicate-classes`  | 删除静态 `class` 属性中重复的类，保留第一个。无论是否指定 `--class-order` 都有效。默认由类顺序的 `removeDuplicates` 决定（`tailwind` 中为关闭）。`--keep-duplicate-classes` 则保留重复项。                                                             |
| `--self-closing`              | 在 Vue 模板中让空元素和组件自闭合（`layout.selfClosing.normal`/`component`，`vue/html-self-closing`）：`<div />`、`<MyComp />`。内置配置文件中为关闭。`--no-self-closing` 将其关闭。空元素（`<br />`）遵循 `--void-self-closing`。              |
| `--void-self-closing`         | 让空元素自闭合：`<input />`、`<meta />`（`layout.selfClosing.void: always`，内置配置文件的设置）。`--no-void-self-closing` 则写成 `<input>`、`<meta>`（`never`）。                                                               |
| `--expand-non-void`           | 在纯 HTML 中，为以自闭合形式书写的非空元素补上结束标签：`<div />` → `<div></div>`（`layout.selfClosing.expandNonVoid`）。HTML 会忽略斜杠，所以 `<div />` 会让元素保持未闭合。`<svg>` 和 `<math>` 内部的元素不受影响。内置配置文件中为关闭。`--no-expand-non-void` 将其关闭。 |
| `--content-newline`           | 让多行元素的内容独占一行（`vue/multiline-html-element-content-newline`）。内置配置文件中为关闭。`--no-content-newline` 将其关闭。                                                                                                  |
| `--print-width <PRINT_WIDTH>` | 样式表和 `<style>` 块的行宽（`stylesheet.malva.printWidth`）。请与所用格式化工具保持一致：Prettier 和 Biome 为 80，oxfmt 为 100。默认使用配置文件中的值（80）。                                                                                 |
| `-l`, `--language <LANGUAGE>` | 标准输入的语言：`html`（默认）、`vue`、`css`、`scss`、`sass`、`less`。                                                                                                                                                |
| `-h`, `--help`                | 显示帮助。                                                                                                                                                                                               |
| `-V`, `--version`             | 显示版本。                                                                                                                                                                                               |

## 语言

文件的语言由扩展名决定：

| 扩展名                               | 语言     | 默认配置文件                    |
| --------------------------------- | ------ | ------------------------- |
| `.html`, `.htm`                   | `html` | `html`                    |
| `.vue`                            | `vue`  | `vue`                     |
| `.css`, `.scss`, `.sass`, `.less` | 样式表    | `html`（其 `stylesheet` 部分） |

其他扩展名（包括 `.pug`、`.jade`、`.styl` 和 `.stylus`）会被报告为不支持，且不会读取文件。包含 Pug 或 Stylus 编写的 `<template>` 或 `<style>`（`lang="pug"`、`"jade"`、`"stylus"`、`"styl"`）的文件同样会报错；参见[限制](/annealer/zh/guide/start/limitations.md#supported-input)。

## 退出码

| 代码  | 含义                                                                    |
| --- | --------------------------------------------------------------------- |
| `0` | 成功。使用 `--check` 时：没有文件会被修改。                                           |
| `1` | 使用 `--check` 时：至少有一个文件会被修改。                                           |
| `2` | 错误：文件无法读取、文件类型不支持、配置文件无效、输入无效（包括 Pug 或 Stylus 块）或未知的 `annealer-*` 指令。 |

指定多个文件时，annealer 会格式化所有文件并逐一报告错误；只要有一个失败，就以 `2` 退出。

## 示例

```bash
anyl src/App.vue                          # 输出结果
anyl --write src/**/*.vue                 # 直接覆盖
anyl --check index.html                   # 在 CI 中使用
anyl --profile my-profile.yaml page.html  # 自定义配置文件
anyl --class-order tailwind src/**/*.vue  # 对 Tailwind 类排序
anyl --class-order tailwind --remove-duplicate-classes src/**/*.vue  # 同时删除重复类
anyl --print-width 100 src/**/*.vue  # 与 oxfmt 一样，样式表在 100 处换行
anyl --self-closing --content-newline src/**/*.vue  # 同时应用可选的布局规则
cat App.vue | anyl --language vue         # 标准输入
```
