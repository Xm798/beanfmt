#[cfg(all(feature = "python", feature = "wasm"))]
compile_error!("features \"python\" and \"wasm\" are mutually exclusive");

pub mod align;
pub mod line;
pub mod normalize;
pub mod options;
#[cfg(any(feature = "cli", feature = "python"))]
pub mod recursive;
pub mod sort;

use align::{align_balance, align_open, align_posting, align_price};
use line::{Line, parse_line};
use normalize::{normalize_braces, normalize_comment, normalize_indent, normalize_thousands};
use options::Options;
use std::borrow::Cow;

pub fn format(input: &str, options: &Options) -> String {
    // Step 1: Sort if enabled
    let working: Cow<str> = if options.sort {
        Cow::Owned(sort::sort_input(input))
    } else {
        Cow::Borrowed(input)
    };

    // Step 2: Parse, normalize, and align each line
    let mut output_lines: Vec<String> = Vec::new();
    let mut meta_depth: usize = 1;

    for raw_line in working.lines() {
        let parsed = parse_line(raw_line);
        let formatted = match parsed {
            Line::BlankLine => String::new(),
            Line::TransactionHeader { date, flag, rest } => {
                let rest = normalize_braces(rest, options.spaces_in_braces);
                if rest.is_empty() {
                    format!("{date} {flag}")
                } else {
                    format!("{date} {flag} {rest}")
                }
            }
            Line::Posting {
                indent: _,
                account,
                number,
                currency,
                cost,
                price,
                comment,
            } => {
                let number = number.map(|n| normalize_thousands(n, &options.thousands_separator));
                let cost = cost.map(|c| normalize_braces(c, options.spaces_in_braces));
                align_posting(
                    &options.indent,
                    account,
                    number.as_deref(),
                    currency,
                    cost.as_deref(),
                    price,
                    comment,
                    options,
                )
            }
            Line::Balance {
                date,
                account,
                number,
                currency,
            } => {
                let number = normalize_thousands(number, &options.thousands_separator);
                align_balance(date, account, &number, currency, options)
            }
            Line::Open {
                date,
                account,
                currencies,
            } => align_open(date, account, currencies, options),
            Line::Close { date, account } => {
                format!("{date} close {account}")
            }
            Line::Price {
                date,
                commodity,
                number,
                currency,
            } => {
                let number = normalize_thousands(number, &options.thousands_separator);
                align_price(date, commodity, &number, currency, options)
            }
            Line::MetaItem {
                indent: _,
                key,
                value,
            } => {
                let value = normalize_braces(value, options.spaces_in_braces);
                format!("{}{key}: {value}", options.indent.repeat(meta_depth))
            }
            Line::Comment { .. } => normalize_comment(raw_line),
            Line::DateDirective {
                date,
                keyword,
                rest,
            } => {
                if rest.is_empty() {
                    format!("{date} {keyword}")
                } else {
                    format!("{date} {keyword} {rest}")
                }
            }
            Line::BlockDirective { .. } | Line::Include { .. } | Line::Other(_) => {
                raw_line.to_string()
            }
        };

        // Normalize indent for indented lines (postings, meta, indented comments)
        let formatted = match parsed {
            Line::Posting { .. } | Line::MetaItem { .. } => formatted,
            Line::Comment { indent, .. } if !indent.is_empty() => {
                normalize_indent(&formatted, &options.indent)
            }
            _ => formatted,
        };

        // Track context for metadata indent depth
        match parsed {
            Line::Posting { .. } => meta_depth = 2,
            Line::TransactionHeader { .. } => meta_depth = 1,
            Line::MetaItem { .. } | Line::Comment { .. } => {}
            _ => meta_depth = 1,
        }

        output_lines.push(formatted);
    }

    let mut result = output_lines.join("\n");

    // Preserve trailing newline
    if input.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }

    result
}

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "python")]
mod python;
