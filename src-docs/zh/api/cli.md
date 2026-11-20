---
description: anyl 命令行工具的选项、输入语言和退出码。
---

# CLI

```text
anyl [OPTIONS] [FILE/DIR/GLOB]...
```

格式化每个文件并输出结果。未指定参数时从标准输入读取。参数的写法参见[文件](#files)。

## 文件 {#files}

与 Prettier 和 oxfmt 一样，每个参数可以是文件、目录或 glob 模式：

| 参数               | 含义                                                |
| ------------------ | --------------------------------------------------- |
| `index.html`       | 该文件本身。                                        |
| `src`、`.`         | 目录中所有受支持的文件（递归）。                    |
| `"src/**/*.vue"`   | 匹配 glob 的受支持文件。`*` 不跨目录，`**` 跨目录。 |
| `"!src/legacy/**"` | 从其他参数的结果中排除匹配的文件。                  |

glob 由 `anyl` 自身展开，因此在任何 shell 和 package.json scripts 中都能使用。请加引号，避免 shell 先行展开。

目录和 glob 会跳过隐藏文件和目录、`node_modules`，以及 `.gitignore` 或 `.anylignore` 中列出的文件（gitignore 语法，与 `.prettierignore` 一样在每个搜索到的目录中读取）。直接指定的文件始终会被格式化。

不匹配任何文件的参数会报错（退出码 `2`），除非指定 `--no-error-on-unmatched-pattern`。

## 选项

| 选项                              | 说明                                                                                                                              |
| --------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `-w`, `--write`                   | 直接覆盖文件。不能与 `--check` 同时使用。                                                                                         |
| `-c`, `--check`                   | 输出会被修改的文件路径；只要有一个文件会被修改，就以状态码 1 退出。                                                               |
| `-p`, `--profile <PROFILE>`       | 内置配置文件名（`html`、`vue`）或配置文件 YAML 的路径。默认使用与各文件语言对应的内置配置文件。                                   |
| `--class-order <CLASS_ORDER>`     | 对静态 `class` 属性中的类排序：内置类顺序名（`tailwind`、`bootstrap`）或类顺序 YAML 文件的路径。会覆盖配置文件中的 `classOrder`。 |
| `-l`, `--language <LANGUAGE>`     | 标准输入的语言：`html`（默认）、`vue`、`css`、`scss`、`sass`、`less`。                                                            |
| `--no-error-on-unmatched-pattern` | 参数不匹配任何文件时不报错。                                                                                                      |
| `-h`, `--help`                    | 显示帮助。                                                                                                                        |
| `-V`, `--version`                 | 显示版本。                                                                                                                        |

## 语言

文件的语言由扩展名决定：

| 扩展名                            | 语言   | 默认配置文件                   |
| --------------------------------- | ------ | ------------------------------ |
| `.html`, `.htm`                   | `html` | `html`                         |
| `.vue`                            | `vue`  | `vue`                          |
| `.css`, `.scss`, `.sass`, `.less` | 样式表 | `html`（其 `stylesheet` 部分） |

其他扩展名会被报告为不支持。

## 退出码

| 代码 | 含义                                                                                                         |
| ---- | ------------------------------------------------------------------------------------------------------------ |
| `0`  | 成功。使用 `--check` 时：没有文件会被修改。                                                                  |
| `1`  | 使用 `--check` 时：至少有一个文件会被修改。                                                                  |
| `2`  | 错误：文件无法读取、文件类型不支持、配置文件无效、输入无效、未知的 `annealer-*` 指令或不匹配任何文件的参数。 |

指定多个文件时，annealer 会格式化所有文件并逐一报告错误；只要有一个失败，就以 `2` 退出。

## 示例

```bash
anyl src/App.vue                          # 输出结果
anyl --write "src/**/*.vue"               # 直接覆盖
anyl --write . "!src/legacy/**"          # 目录中排除某个模式
anyl --check index.html                   # 在 CI 中使用
anyl --profile my-profile.yaml page.html  # 自定义配置文件
anyl --class-order tailwind "src/**/*.vue" # 对 Tailwind 类排序
cat App.vue | anyl --language vue         # 标准输入
```
