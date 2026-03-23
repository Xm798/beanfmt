# Beanfmt - Beancount Formatter

[中文](#中文)

A fast [Beancount](https://beancount.github.io/) file formatter extension for Visual Studio Code, with CJK double-width character support.

## Features

- **Column alignment** — automatically aligns currencies and cost annotations
- **CJK-aware** — correctly handles double-width CJK characters for alignment
- **Thousands separator** — add, remove, or keep commas in numbers
- **Brace spacing** — control spaces inside cost braces `{ ... }`
- **Date sorting** — optionally sort entries by date
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
| `beanfmt.indent` | `4` | Number of spaces for indentation |
| `beanfmt.currencyColumn` | `70` | Column to align currencies to |
| `beanfmt.costColumn` | `75` | Column to align costs/prices to |
| `beanfmt.thousandsSeparator` | `"keep"` | Thousands separator: `"add"`, `"remove"`, or `"keep"` |
| `beanfmt.spacesInBraces` | `false` | Add spaces inside cost braces |
| `beanfmt.fixedCJKWidth` | `true` | Treat CJK characters as double-width for alignment |
| `beanfmt.sort` | `false` | Sort entries by date |

---

# 中文

一个快速的 [Beancount](https://beancount.github.io/) 文件格式化 VS Code 扩展，支持 CJK 双宽度字符对齐。

## 功能特性

- **列对齐** — 自动对齐货币和成本标注
- **CJK 感知** — 正确处理中日韩双宽度字符的对齐
- **千位分隔符** — 添加、移除或保留数字中的逗号
- **花括号空格** — 控制成本花括号内的空格 `{ ... }`
- **日期排序** — 可选按日期排序条目
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
| `beanfmt.indent` | `4` | 缩进空格数 |
| `beanfmt.currencyColumn` | `70` | 货币对齐列 |
| `beanfmt.costColumn` | `75` | 成本/价格对齐列 |
| `beanfmt.thousandsSeparator` | `"keep"` | 千位分隔符处理：`"add"` 添加、`"remove"` 移除、`"keep"` 保持 |
| `beanfmt.spacesInBraces` | `false` | 成本花括号内添加空格 |
| `beanfmt.fixedCJKWidth` | `true` | 将 CJK 字符视为双宽度进行对齐 |
| `beanfmt.sort` | `false` | 按日期排序条目 |

## 许可证

MIT
