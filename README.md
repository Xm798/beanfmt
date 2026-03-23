# Husk

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

### CLI (from source)

```bash
cargo install --path .
```

### Python

```bash
pip install maturin
maturin develop --features python
```

### WASM

```bash
wasm-pack build --target nodejs --features wasm --no-default-features
```

### VSCode Extension

```bash
cd editors/code
npm install
npm run build:wasm
npm run compile
npm run package
```

Install the generated `.vsix` file:

```bash
code --install-extension editors/code/husk-beancount-formatter-0.1.0.vsix
```

> Note: `python` and `wasm` features are mutually exclusive.

## Usage

### CLI

```bash
# Format from stdin
cat ledger.beancount | husk

# Format a file (print to stdout)
husk ledger.beancount

# Format in-place
husk -w ledger.beancount

# Recursively format all included files in-place
husk --recursive -w ledger.beancount

# Custom alignment columns
husk --currency-column 60 --cost-column 65 ledger.beancount

# Add thousands separators and sort by date
husk --thousands add --sort ledger.beancount
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--indent <STR>` | 4 spaces | Indentation string for postings and metadata |
| `--currency-column <N>` | `70` | Target column for currency alignment |
| `--cost-column <N>` | `75` | Target column for cost/price alignment |
| `--thousands <MODE>` | `keep` | Thousands separator: `add`, `remove`, or `keep` |
| `--spaces-in-braces` | off | Add spaces inside cost braces `{ ... }` |
| `--no-fixed-cjk-width` | off | Disable CJK double-width alignment |
| `--sort` | off | Sort entries by date |
| `--recursive` | off | Follow and format `include`d files |
| `-w, --write` | off | Write output back to file (in-place) |

### Python

```python
import husk

# Format a string
output = husk.format(source, currency_column=60, sort=True)

# Format a file
output = husk.format_file("ledger.beancount")

# Reusable options
opts = husk.Options(currency_column=60, thousands_separator="add")
output = husk.format(source, options=opts)

# Recursive formatting — returns list of (path, content) tuples
results = husk.format_recursive("ledger.beancount")
```

### WASM

```javascript
import { format, format_default } from "husk";

// Format with default options
const output = format_default(source);

// Format with full options
const output = format(source, "    ", 70, 75, "keep", false, true, false);
```

### VSCode

Install the extension, then configure in `settings.json`:

```jsonc
"[beancount]": {
    "editor.defaultFormatter": "husk.husk-beancount-formatter",
    "editor.formatOnSave": true
}
```

Available settings:

| Setting | Default | Description |
|---------|---------|-------------|
| `husk.indent` | `"    "` | Indentation string |
| `husk.currencyColumn` | `70` | Currency alignment column |
| `husk.costColumn` | `75` | Cost/price alignment column |
| `husk.thousandsSeparator` | `"keep"` | `"add"`, `"remove"`, or `"keep"` |
| `husk.spacesInBraces` | `false` | Spaces inside cost braces |
| `husk.fixedCJKWidth` | `true` | CJK double-width alignment |
| `husk.sort` | `false` | Sort entries by date |

## License

MIT
