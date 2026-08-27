# Beanfmt - Beancount Formatter

[中文](#中文)

A fast [Beancount](https://beancount.github.io/) file formatter extension for Visual Studio Code, with CJK double-width character support and smart date-based sorting.

## Features

- **Column alignment** — automatically aligns currencies and cost annotations
- **CJK-aware** — correctly handles double-width CJK characters for alignment
- **Thousands separator** — add, remove, or keep commas in numbers
- **Decimal places** — keep, strip trailing zeros, or pad to a fixed width
- **Brace spacing** — control spaces inside cost braces `{ ... }`
- **Inline comment alignment** — align trailing `;` comments to a column
- **Smart date sorting** — sort entries by date (asc/desc) with `time` metadata intra-day ordering, timeless-entry positioning, and directive-type sort barriers
- **Fast** — powered by a WASM-compiled Rust core

## Usage

1. Open a `.bean` or `.beancount` file
2. Format via `Shift+Alt+F` (or `Shift+Option+F` on macOS), or enable format on save:

```jsonc
"[beancount]": {
    "editor.defaultFormatter": "cyrus-x.beanfmt",
    "editor.formatOnSave": true
}
```

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `beanfmt.indent` | `2` | Number of spaces for indentation |
| `beanfmt.currencyColumn` | `70` | Column to align currencies to |
| `beanfmt.costColumn` | `75` | Column to align costs/prices to |
| `beanfmt.inlineCommentColumn` | `0` | Column to align inline comments (`;`) to; `0` disables alignment |
| `beanfmt.thousandsSeparator` | `"keep"` | Thousands separator: `"add"`, `"remove"`, or `"keep"` |
| `beanfmt.decimalMode` | `"keep"` | Decimal places: `"keep"`, `"minimal"` (strip trailing zeros), or `"pad"` |
| `beanfmt.decimalPlaces` | `2` | Fraction width to pad to when `decimalMode` is `"pad"` |
| `beanfmt.spacesInBraces` | `false` | Add spaces inside cost braces |
| `beanfmt.fixedCJKWidth` | `true` | Treat CJK characters as double-width for alignment |
| `beanfmt.sort` | `"off"` | Sort entries by date: `"off"`, `"asc"`, or `"desc"` |
| `beanfmt.sortTimeless` | `"keep"` | Where to place timeless entries within a day: `"begin"`, `"end"`, or `"keep"` |
| `beanfmt.sortExclude` | `[]` | Directive types to exclude from sorting; excluded directives act as sort barriers |

> **Note:** These settings are ignored when a `.beanfmt.toml` or `beanfmt.toml` exists in the workspace — the config file is the source of truth. See the [project README](https://github.com/Xm798/beanfmt#configuration) for the config file format.

---

# 中文

一个快速的 [Beancount](https://beancount.github.io/) 文件格式化 VS Code 扩展，支持 CJK 双宽度字符对齐和智能日期排序。

## 功能特性

- **列对齐** — 自动对齐货币和成本标注
- **CJK 感知** — 正确处理中日韩双宽度字符的对齐
- **千位分隔符** — 添加、移除或保留数字中的逗号
- **小数位** — 保持不变、去除末尾多余的零，或补齐到固定位数
- **花括号空格** — 控制成本花括号内的空格 `{ ... }`
- **行内注释对齐** — 将行尾 `;` 注释对齐到指定列
- **智能日期排序** — 按日期排序条目（升序/降序），支持 `time` 元数据的日内排序、无时间条目定位以及按指令类型设置排序屏障
- **高性能** — 基于 Rust 编译为 WASM 的核心引擎

## 使用方法

1. 打开 `.bean` 或 `.beancount` 文件
2. 使用 `Shift+Alt+F`（macOS 上为 `Shift+Option+F`）格式化，或启用保存时自动格式化：

```jsonc
"[beancount]": {
    "editor.defaultFormatter": "cyrus-x.beanfmt",
    "editor.formatOnSave": true
}
```

## 配置项

| 配置 | 默认值 | 说明 |
|------|--------|------|
| `beanfmt.indent` | `2` | 缩进空格数 |
| `beanfmt.currencyColumn` | `70` | 货币对齐列 |
| `beanfmt.costColumn` | `75` | 成本/价格对齐列 |
| `beanfmt.inlineCommentColumn` | `0` | 行内注释（`;`）对齐列；`0` 表示不对齐 |
| `beanfmt.thousandsSeparator` | `"keep"` | 千位分隔符处理：`"add"` 添加、`"remove"` 移除、`"keep"` 保持 |
| `beanfmt.decimalMode` | `"keep"` | 小数位处理：`"keep"` 保持、`"minimal"` 去除末尾的零、`"pad"` 补齐 |
| `beanfmt.decimalPlaces` | `2` | `decimalMode` 为 `"pad"` 时补齐到的小数位数 |
| `beanfmt.spacesInBraces` | `false` | 成本花括号内添加空格 |
| `beanfmt.fixedCJKWidth` | `true` | 将 CJK 字符视为双宽度进行对齐 |
| `beanfmt.sort` | `"off"` | 按日期排序条目：`"off"`、`"asc"`、`"desc"` |
| `beanfmt.sortTimeless` | `"keep"` | 无时间条目在当天的位置：`"begin"`、`"end"`、`"keep"` |
| `beanfmt.sortExclude` | `[]` | 排除排序的指令类型；被排除的指令作为排序屏障 |

> **注意：** 当工作区存在 `.beanfmt.toml` 或 `beanfmt.toml` 时，以上配置项将被忽略——配置文件是唯一可信来源。配置文件格式见[项目 README](https://github.com/Xm798/beanfmt#configuration)。

## 许可证

MIT
