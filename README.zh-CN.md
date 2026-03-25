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
| `--indent <N>` | `4` | 缩进空格数 |
| `--currency-column <N>` | `70` | 货币对齐目标列 |
| `--cost-column <N>` | `75` | 成本/价格对齐目标列 |
| `--thousands <MODE>` | `keep` | 千分位分隔符：`add`（添加）、`remove`（移除）、`keep`（保持） |
| `--spaces-in-braces` / `--no-spaces-in-braces` | 关闭 | 在成本花括号内添加空格 `{ ... }` |
| `--fixed-cjk-width` / `--no-fixed-cjk-width` | 开启 | CJK 双宽度字符对齐 |
| `--sort [MODE]` / `--no-sort` | `off` | 按日期排序条目：`asc`（默认，单独使用 `--sort` 时）、`desc`、`off` |
| `--recursive` | 关闭 | 递归格式化 `include` 引入的文件 |
| `-w, --write` | 关闭 | 将输出写回文件（原地修改） |
| `--no-config` | 关闭 | 跳过配置文件加载 |

### 配置文件

Beanfmt 支持 TOML 配置文件，按以下优先级合并（低 → 高）：

1. 内置默认值
2. 全局配置：`$XDG_CONFIG_HOME/beanfmt/config.toml`（默认为 `~/.config/beanfmt/config.toml`）
3. 项目配置：`.beanfmt.toml` 或 `beanfmt.toml`（从当前目录向上查找）
4. CLI 参数（最高优先级）

示例 `.beanfmt.toml`：

```toml
indent = 2
currency_column = 60
cost_column = 65
thousands = "add"
spaces_in_braces = true
fixed_cjk_width = true
sort = "asc"    # "asc"、"desc"、"off" 或 true/false
sort_timeless = "keep"   # "begin"、"end" 或 "keep"
sort_exclude = ["open", "close"]  # 排除的指令类型作为排序屏障
```

所有字段均为可选，未指定的字段从下一优先级层继承。使用 `--no-config` 可跳过所有配置文件加载。完整配置参考见 [`beanfmt.toml`](beanfmt.toml)。

各入口对配置文件的支持：

| 入口 | 全局配置 | 项目配置 | 说明 |
|------|---------|---------|------|
| CLI | 自动加载 | 自动加载 | `--no-config` 可禁用 |
| Python | 不支持 | 通过 `config=True` 启用 | 也可用 `load_project_config()` 手动加载 |
| VSCode | 不支持（用户设置替代） | 自动加载（工作区内） | 显式设置覆盖配置文件 |

### Python

```python
import beanfmt

# 格式化字符串
output = beanfmt.format(source, currency_column=60, sort=True)

# 格式化文件
output = beanfmt.format_file("ledger.beancount")

# 使用项目配置（自动发现 .beanfmt.toml）
output = beanfmt.format_file("ledger.beancount", config=True)

# 指定配置文件路径
output = beanfmt.format_file("ledger.beancount", config="/path/to/.beanfmt.toml")

# 配置文件 + kwargs 覆盖（kwargs 优先级更高）
output = beanfmt.format_file("ledger.beancount", config=True, indent=8)

# 加载和检查项目配置
opts = beanfmt.load_project_config("/path/to/project/")
opts = beanfmt.parse_config('indent = 2\ncurrency_column = 80\n')

# 可复用的选项对象
opts = beanfmt.Options(currency_column=60, thousands_separator="add")
output = beanfmt.format(source, options=opts)

# 递归格式化 - 返回 (路径, 内容) 元组列表
results = beanfmt.format_recursive("ledger.beancount", config=True)
```

`format_file` 和 `format_recursive` 的 `config` 参数接受 `None`（默认，不加载配置）、`True`（从文件目录向上查找 `.beanfmt.toml`）、`False`（同 `None`）或配置文件路径字符串。kwargs 始终覆盖配置文件中的值。`config` 和 `options` 参数互斥。

### WASM

```javascript
import { format, format_default } from "beanfmt";

// 使用默认选项格式化
const output = format_default(source);

// 使用完整选项格式化
const output = format(source, 4, 70, 75, "keep", false, true, false);
```

### VSCode

安装扩展后，在 `settings.json` 中配置：

```jsonc
"[beancount]": {
    "editor.defaultFormatter": "beanfmt.beanfmt-beancount-formatter",
    "editor.formatOnSave": true
}
```

扩展会自动读取工作区中的 `.beanfmt.toml` 或 `beanfmt.toml`（从文件所在目录向上搜索至工作区根目录）。显式设置的 VSCode 配置项会覆盖配置文件中的值；未设置的项回退到配置文件，再回退到内置默认值。

可用设置：

| 设置项 | 默认值 | 说明 |
|--------|--------|------|
| `beanfmt.indent` | `4` | 缩进空格数 |
| `beanfmt.currencyColumn` | `70` | 货币对齐列 |
| `beanfmt.costColumn` | `75` | 成本/价格对齐列 |
| `beanfmt.thousandsSeparator` | `"keep"` | `"add"`（添加）、`"remove"`（移除）、`"keep"`（保持） |
| `beanfmt.spacesInBraces` | `false` | 花括号内添加空格 |
| `beanfmt.fixedCJKWidth` | `true` | CJK 双宽度字符对齐 |
| `beanfmt.sort` | `"off"` | 按日期排序：`"asc"`、`"desc"`、`"off"` |
| `beanfmt.sortTimeless` | `"keep"` | 无时间条目在日内的位置：`"begin"`、`"end"`、`"keep"` |
| `beanfmt.sortExclude` | `[]` | 排除的指令类型（作为排序屏障） |

## 许可证

MIT
