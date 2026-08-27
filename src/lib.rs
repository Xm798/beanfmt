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

use align::{align_balance, align_close, align_open, align_posting, align_price};
use line::{Line, parse_line};
use normalize::{
    normalize_amount, normalize_braces, normalize_comment_aligned, normalize_indent,
    normalize_number,
};
use options::{AmountScope, Options, SortOrder, SortableDirective};
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
    let raw_lines: Vec<&str> = working.lines().collect();
    let comment_semi_widths = comment_block_semi_widths(&raw_lines);
    let mut output_lines: Vec<String> = Vec::new();
    let mut meta_depth: usize = 1;
    let indent_str = options.indent_str();

    for (i, raw_line) in raw_lines.iter().copied().enumerate() {
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
                let number = number.map(|n| normalize_number(n, options));
                let cost = cost.map(|c| {
                    normalize_braces(&normalize_amount(c, options), options.spaces_in_braces)
                });
                let price = price.map(|p| {
                    let p = normalize_amount(p, options);
                    if p.contains("- ") {
                        Cow::Owned(p.replace("- ", "-"))
                    } else {
                        p
                    }
                });
                align_posting(
                    &indent_str,
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
                comment,
            } => {
                let number = normalize_number(number, options);
                align_balance(date, account, &number, currency, comment, options)
            }
            Line::Open {
                date,
                account,
                currencies,
                comment,
            } => align_open(date, account, currencies, comment, options),
            Line::Close {
                date,
                account,
                comment,
            } => align_close(date, account, comment, options),
            Line::Price {
                date,
                commodity,
                number,
                currency,
                comment,
            } => {
                let number = if options.amount_scope == AmountScope::All {
                    normalize_number(number, options)
                } else {
                    Cow::Borrowed(number)
                };
                align_price(date, commodity, &number, currency, comment, options)
            }
            Line::MetaItem {
                indent: _,
                key,
                value,
            } => {
                let value = normalize_braces(value, options.spaces_in_braces);
                format!("{}{key}: {value}", " ".repeat(options.indent * meta_depth))
            }
            Line::Comment { .. } => normalize_comment_aligned(raw_line, comment_semi_widths[i]),
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
            Line::TransactionHeader { .. } => !options
                .sort_exclude
                .contains(&SortableDirective::Transaction),
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
                directive.is_none_or(|d| !options.sort_exclude.contains(&d))
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

/// Leading whitespace and semicolon-run length of a comment line, or `None` if the
/// line does not begin (after indentation) with `;`. Mirrors the comment grammar of
/// `COMMENT_RE` but skips the full `parse_line` regex pass for this pre-scan.
fn comment_prefix(line: &str) -> Option<(&str, usize)> {
    let indent_len = line.len() - line.trim_start().len();
    let semis = line[indent_len..]
        .bytes()
        .take_while(|&b| b == b';')
        .count();
    (semis > 0).then(|| (&line[..indent_len], semis))
}

/// For each line, the semicolon-prefix width its comment block should align to.
/// A comment block is a maximal run of consecutive comment lines sharing the same
/// indentation; the width is the longest semicolon prefix in that run. Non-comment
/// lines get 0.
fn comment_block_semi_widths(lines: &[&str]) -> Vec<usize> {
    let mut widths = vec![0usize; lines.len()];
    let mut i = 0;
    while i < lines.len() {
        let Some((indent, semis)) = comment_prefix(lines[i]) else {
            i += 1;
            continue;
        };
        let mut max = semis;
        let mut j = i + 1;
        while let Some((ind, s)) = lines.get(j).and_then(|l| comment_prefix(l)) {
            if ind != indent {
                break;
            }
            max = max.max(s);
            j += 1;
        }
        for w in &mut widths[i..j] {
            *w = max;
        }
        i = j;
    }
    widths
}

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "python")]
mod python;
