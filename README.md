# Beanfmt

[中文文档](README.zh-CN.md)

A fast [beancount](https://beancount.github.io/) file formatter with CJK double-width character support.

## Features

- Column-aligned currencies and costs with CJK-aware display width
- Thousands separator normalization (add, remove, or keep)
- Brace spacing control for cost annotations
- Date-based entry sorting
- Recursive formatting of `include`d files (with glob support)
- Multi-platform: CLI, Python library, WASM module, and VSCode extension

## Installation

### CLI

```bash
cargo install beanfmt
```

### Python

```bash
pip install beanfmt
```

### VSCode Extension

Search for `beanfmt` in the VSCode Marketplace, or install from the command line:

```bash
code --install-extension beanfmt.beanfmt-beancount-formatter
```

### From Source

```bash
cargo install --path .                 # CLI
maturin develop --features python      # Python (requires maturin)
```

> Note: `python` and `wasm` features are mutually exclusive.

## Usage

### CLI

```bash
# Format from stdin
cat ledger.beancount | beanfmt

# Format a file (print to stdout)
beanfmt ledger.beancount

# Format in-place
beanfmt -w ledger.beancount

# Recursively format all included files in-place
beanfmt --recursive -w ledger.beancount

# Custom alignment columns
beanfmt --currency-column 60 --cost-column 65 ledger.beancount

# Add thousands separators and sort by date
beanfmt --thousands add --sort ledger.beancount
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--indent <N>` | `4` | Number of spaces for indentation |
| `--currency-column <N>` | `70` | Target column for currency alignment |
| `--cost-column <N>` | `75` | Target column for cost/price alignment |
| `--thousands <MODE>` | `keep` | Thousands separator: `add`, `remove`, or `keep` |
| `--spaces-in-braces` / `--no-spaces-in-braces` | off | Add spaces inside cost braces `{ ... }` |
| `--fixed-cjk-width` / `--no-fixed-cjk-width` | on | CJK double-width alignment |
| `--sort [MODE]` / `--no-sort` | `off` | Sort entries by date: `asc` (default if bare `--sort`), `desc`, `off` |
| `--sort-timeless <POS>` | `begin` | Where to place timeless entries within a day: `begin`, `end` |
| `--sort-exclude <TYPES>` | (none) | Comma-separated directive types to exclude from sorting; excluded directives act as sort barriers. Values: `transaction`, `balance`, `open`, `close`, `price`, `pad`, `note`, `document`, `event`, `custom`, `query`, `commodity` |
| `--recursive` | off | Follow and format `include`d files |
| `-w, --write` | off | Write output back to file (in-place) |
| `--no-config` | off | Skip loading configuration files |

### Configuration File

Beanfmt supports TOML configuration files with a three-layer merge priority (low → high):

1. Built-in defaults
2. Global config: `$XDG_CONFIG_HOME/beanfmt/config.toml` (defaults to `~/.config/beanfmt/config.toml`)
3. Project config: `.beanfmt.toml` or `beanfmt.toml` (searched upward from the current directory)
4. CLI arguments (highest priority)

Example `.beanfmt.toml`:

```toml
indent = 2
currency_column = 60
cost_column = 65
thousands = "add"
spaces_in_braces = true
fixed_cjk_width = true
sort = "asc"    # "asc", "desc", "off", or true/false
sort_timeless = "begin"  # "begin" or "end"
sort_exclude = ["open", "close"]  # excluded types act as sort barriers
```

All fields are optional. Unspecified fields inherit from the next lower priority layer. Use `--no-config` to skip all configuration file loading. See [`beanfmt.toml`](beanfmt.toml) for a full reference with comments.

### Python

```python
import beanfmt

# Format a string
output = beanfmt.format(source, currency_column=60, sort=True)

# Format a file
output = beanfmt.format_file("ledger.beancount")

# Reusable options
opts = beanfmt.Options(currency_column=60, thousands_separator="add")
output = beanfmt.format(source, options=opts)

# Recursive formatting — returns list of (path, content) tuples
results = beanfmt.format_recursive("ledger.beancount")
```

### WASM

```javascript
import { format, format_default } from "beanfmt";

// Format with default options
const output = format_default(source);

// Format with full options
const output = format(source, 4, 70, 75, "keep", false, true, "off", "begin", undefined);
```

### VSCode

Install the extension, then configure in `settings.json`:

```jsonc
"[beancount]": {
    "editor.defaultFormatter": "beanfmt.beanfmt-beancount-formatter",
    "editor.formatOnSave": true
}
```

Available settings:

| Setting | Default | Description |
|---------|---------|-------------|
| `beanfmt.indent` | `4` | Number of spaces for indentation |
| `beanfmt.currencyColumn` | `70` | Currency alignment column |
| `beanfmt.costColumn` | `75` | Cost/price alignment column |
| `beanfmt.thousandsSeparator` | `"keep"` | `"add"`, `"remove"`, or `"keep"` |
| `beanfmt.spacesInBraces` | `false` | Spaces inside cost braces |
| `beanfmt.fixedCJKWidth` | `true` | CJK double-width alignment |
| `beanfmt.sort` | `"off"` | Sort entries by date: `"asc"`, `"desc"`, `"off"` |
| `beanfmt.sortTimeless` | `"begin"` | Timeless entry position within a day: `"begin"`, `"end"` |
| `beanfmt.sortExclude` | `[]` | Directive types to exclude from sorting (act as sort barriers) |

## License

MIT
