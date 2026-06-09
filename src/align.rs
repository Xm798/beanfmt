use unicode_width::UnicodeWidthStr;

use crate::options::Options;

fn compute_padding(target_column: usize, content_width: usize, min_gap: usize) -> usize {
    if target_column > content_width + min_gap {
        target_column - 1 - content_width
    } else {
        min_gap
    }
}

/// Calculate display width of a string, accounting for CJK double-width characters.
pub fn display_width(s: &str, fixed_cjk_width: bool) -> usize {
    if fixed_cjk_width {
        UnicodeWidthStr::width(s)
    } else {
        s.chars().count()
    }
}

/// Append an inline comment to `result`, aligning its `;` to `inline_comment_column`
/// (1-indexed) when set. A column of 0 keeps the comment one space after the content.
/// When the content already reaches or passes the target column, a single space is used.
pub fn append_comment(result: String, comment: Option<&str>, options: &Options) -> String {
    let Some(comment) = comment else {
        return result;
    };
    if options.inline_comment_column == 0 {
        return format!("{result} {comment}");
    }
    let current = display_width(&result, options.fixed_cjk_width);
    let padding = compute_padding(options.inline_comment_column, current, 1);
    format!("{result}{:padding$}{comment}", "")
}

/// Right-pad a string with spaces to reach `target_width` display columns.
/// If already at or beyond target, append a single space.
pub fn pad_to_width(s: &str, target_width: usize, fixed_cjk_width: bool) -> String {
    let current = display_width(s, fixed_cjk_width);
    if current >= target_width {
        format!("{s} ")
    } else {
        let padding = target_width - current;
        format!("{s}{:padding$}", "")
    }
}

/// Align a posting line so currency starts at `currency_column`.
#[allow(clippy::too_many_arguments)]
pub fn align_posting(
    indent: &str,
    account: &str,
    number: Option<&str>,
    currency: Option<&str>,
    cost: Option<&str>,
    price: Option<&str>,
    comment: Option<&str>,
    options: &Options,
) -> String {
    let cjk = options.fixed_cjk_width;

    // Account-only posting (no amount)
    let (Some(number), Some(currency)) = (number, currency) else {
        return append_comment(format!("{indent}{account}"), comment, options);
    };

    let prefix_width = display_width(indent, cjk) + display_width(account, cjk);
    let num_width = display_width(number, cjk);
    // Currency starts at currency_column (1-indexed).
    // Layout: indent + account + spaces + number + " " + currency
    // display_width before currency = prefix_width + padding + num_width + 1 (space)
    // We want that to equal currency_column - 1 (0-indexed position).
    // padding = currency_column - 1 - prefix_width - num_width - 1
    let min_before = prefix_width + num_width + 1;
    let padding = compute_padding(options.currency_column, min_before, 2);

    let mut result = format!("{indent}{account}{:padding$}{number} {currency}", "");

    // Align cost to cost_column if present
    if let Some(cost) = cost {
        let current_width = display_width(&result, cjk);
        let cost_padding = compute_padding(options.cost_column, current_width + 1, 1);
        result = format!("{result}{:cost_padding$} {cost}", "");
    }

    if let Some(price) = price {
        result = format!("{result} {price}");
    }

    append_comment(result, comment, options)
}

/// Align a balance directive so currency starts at `currency_column`.
pub fn align_balance(
    date: &str,
    account: &str,
    number: &str,
    currency: &str,
    comment: Option<&str>,
    options: &Options,
) -> String {
    let cjk = options.fixed_cjk_width;
    let prefix = format!("{date} balance {account}");
    let prefix_width = display_width(&prefix, cjk);
    let num_width = display_width(number, cjk);

    let min_before = prefix_width + num_width + 1;
    let padding = compute_padding(options.currency_column, min_before, 2);

    let result = format!("{prefix}{:padding$}{number} {currency}", "");
    append_comment(result, comment, options)
}

/// Align an open directive so currencies start at `currency_column`.
pub fn align_open(
    date: &str,
    account: &str,
    currencies: &str,
    comment: Option<&str>,
    options: &Options,
) -> String {
    if currencies.is_empty() {
        return append_comment(format!("{date} open {account}"), comment, options);
    }

    let cjk = options.fixed_cjk_width;
    let prefix = format!("{date} open {account}");
    let prefix_width = display_width(&prefix, cjk);

    let padding = compute_padding(options.currency_column, prefix_width, 2);

    let result = format!("{prefix}{:padding$}{currencies}", "");
    append_comment(result, comment, options)
}

/// Assemble a close directive, aligning any inline comment.
pub fn align_close(date: &str, account: &str, comment: Option<&str>, options: &Options) -> String {
    append_comment(format!("{date} close {account}"), comment, options)
}

/// Align a price directive so currency starts at `currency_column`.
pub fn align_price(
    date: &str,
    commodity: &str,
    number: &str,
    currency: &str,
    comment: Option<&str>,
    options: &Options,
) -> String {
    let cjk = options.fixed_cjk_width;
    let prefix = format!("{date} price {commodity}");
    let prefix_width = display_width(&prefix, cjk);
    let num_width = display_width(number, cjk);

    let min_before = prefix_width + num_width + 1;
    let padding = compute_padding(options.currency_column, min_before, 2);

    let result = format!("{prefix}{:padding$}{number} {currency}", "");
    append_comment(result, comment, options)
}
