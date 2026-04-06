#[cfg(all(feature = "python", feature = "wasm"))]
compile_error!("features \"python\" and \"wasm\" are mutually exclusive");

pub mod align;
#[cfg(feature = "file-config")]
pub mod config;
pub mod line;
pub mod normalize;
pub mod options;
#[cfg(any(feature = "cli", feature = "python"))]
pub mod recursive;
pub mod sort;

use align::{align_balance, align_open, align_posting, align_price};
use line::{Line, parse_line};
use normalize::{normalize_braces, normalize_comment, normalize_indent, normalize_thousands};
use options::{Options, SortOrder, SortableDirective};
use std::borrow::Cow;

pub fn format(input: &str, options: &Options) -> String {
    // Step 1: Sort if enabled
    let working: Cow<str> = match options.sort {
        SortOrder::Off => Cow::Borrowed(input),
        SortOrder::Asc => Cow::Owned(sort::sort_input(
            input,
            false,
            options.sort_timeless,
            &options.sort_exclude,
        )),
        SortOrder::Desc => Cow::Owned(sort::sort_input(
            input,
            true,
            options.sort_timeless,
            &options.sort_exclude,
        )),
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
                let price = price.map(|p| p.replace("- ", "-"));
                align_posting(
                    &options.indent_str(),
                    account,
                    number.as_deref(),
                    currency,
                    cost.as_deref(),
                    price.as_deref(),
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
                format!("{}{key}: {value}", options.indent_str().repeat(meta_depth))
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
                normalize_indent(&formatted, options.indent)
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

        let is_nonexcluded_entry = match parsed {
            Line::TransactionHeader { .. } => {
                !options.sort_exclude.contains(&SortableDirective::Transaction)
            }
            Line::Balance { .. } => !options.sort_exclude.contains(&SortableDirective::Balance),
            Line::Open { .. } => !options.sort_exclude.contains(&SortableDirective::Open),
            Line::Close { .. } => !options.sort_exclude.contains(&SortableDirective::Close),
            Line::Price { .. } => !options.sort_exclude.contains(&SortableDirective::Price),
            Line::DateDirective { keyword, .. } => {
                let directive = match keyword {
                    "pad" => Some(SortableDirective::Pad),
                    "note" => Some(SortableDirective::Note),
                    "document" => Some(SortableDirective::Document),
                    "event" => Some(SortableDirective::Event),
                    "custom" => Some(SortableDirective::Custom),
                    "query" => Some(SortableDirective::Query),
                    "commodity" => Some(SortableDirective::Commodity),
                    _ => None,
                };
                directive.map_or(true, |d| !options.sort_exclude.contains(&d))
            }
            _ => false,
        };
        if is_nonexcluded_entry
            && output_lines
                .last()
                .is_some_and(|l| !l.is_empty() && !l.starts_with(';'))
        {
            output_lines.push(String::new());
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
