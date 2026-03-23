# Beanfmt

快速的 [beancount](https://beancount.github.io/) 文件格式化工具，支持 CJK 双宽度字符对齐。

## 功能特性

- 货币和成本列对齐，支持 CJK 字符显示宽度计算
- 千分位分隔符处理（添加、移除或保持不变）
- 成本花括号内空格控制
- 按日期排序条目
- 递归格式化 `include` 引入的文件（支持 glob 模式）
- 多平台支持：CLI、Python 库、WASM 模块、VSCode 扩展

## 安装

### CLI

```bash
cargo install beanfmt
```

### Python

```bash
pip install beanfmt
```

### VSCode 扩展

在 VSCode 扩展商店搜索 `beanfmt`，或通过命令行安装：

```bash
code --install-extension beanfmt.beanfmt-beancount-formatter
```

### 从源码构建

```bash
cargo install --path .                 # CLI
maturin develop --features python      # Python（需要 maturin）
```

> 注意：`python` 和 `wasm` 特性互斥，不能同时启用。

## 使用方法

### CLI

```bash
# 从标准输入格式化
cat ledger.beancount | beanfmt

# 格式化文件（输出到标准输出）
beanfmt ledger.beancount

# 原地格式化
beanfmt -w ledger.beancount

# 递归格式化所有 include 的文件
beanfmt --recursive -w ledger.beancount

# 自定义对齐列
beanfmt --currency-column 60 --cost-column 65 ledger.beancount

# 添加千分位分隔符并按日期排序
beanfmt --thousands add --sort ledger.beancount
```

### 选项

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--indent <STR>` | 4 个空格 | 过账和元数据的缩进字符串 |
| `--currency-column <N>` | `70` | 货币对齐目标列 |
| `--cost-column <N>` | `75` | 成本/价格对齐目标列 |
| `--thousands <MODE>` | `keep` | 千分位分隔符：`add`（添加）、`remove`（移除）、`keep`（保持） |
| `--spaces-in-braces` | 关闭 | 在成本花括号内添加空格 `{ ... }` |
| `--no-fixed-cjk-width` | 关闭 | 禁用 CJK 双宽度对齐 |
| `--sort` | 关闭 | 按日期排序条目 |
| `--recursive` | 关闭 | 递归格式化 `include` 引入的文件 |
| `-w, --write` | 关闭 | 将输出写回文件（原地修改） |

### Python

```python
import beanfmt

# 格式化字符串
output = beanfmt.format(source, currency_column=60, sort=True)

# 格式化文件
output = beanfmt.format_file("ledger.beancount")

# 可复用的选项对象
opts = beanfmt.Options(currency_column=60, thousands_separator="add")
output = beanfmt.format(source, options=opts)

# 递归格式化 - 返回 (路径, 内容) 元组列表
results = beanfmt.format_recursive("ledger.beancount")
```

### WASM

```javascript
import { format, format_default } from "beanfmt";

// 使用默认选项格式化
const output = format_default(source);

// 使用完整选项格式化
const output = format(source, "    ", 70, 75, "keep", false, true, false);
```

### VSCode

安装扩展后，在 `settings.json` 中配置：

```jsonc
"[beancount]": {
    "editor.defaultFormatter": "beanfmt.beanfmt-beancount-formatter",
    "editor.formatOnSave": true
}
```

可用设置：

| 设置项 | 默认值 | 说明 |
|--------|--------|------|
| `beanfmt.indent` | `"    "` | 缩进字符串 |
| `beanfmt.currencyColumn` | `70` | 货币对齐列 |
| `beanfmt.costColumn` | `75` | 成本/价格对齐列 |
| `beanfmt.thousandsSeparator` | `"keep"` | `"add"`（添加）、`"remove"`（移除）、`"keep"`（保持） |
| `beanfmt.spacesInBraces` | `false` | 花括号内添加空格 |
| `beanfmt.fixedCJKWidth` | `true` | CJK 双宽度字符对齐 |
| `beanfmt.sort` | `false` | 按日期排序 |

## 许可证

MIT
