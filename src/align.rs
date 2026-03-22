use unicode_width::UnicodeWidthStr;

use crate::options::Options;

/// Calculate display width of a string, accounting for CJK double-width characters.
pub fn display_width(s: &str, fixed_cjk_width: bool) -> usize {
    if fixed_cjk_width {
        UnicodeWidthStr::width(s)
    } else {
        s.chars().count()
    }
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
        return format!("{indent}{account}");
    };

    let prefix_width = display_width(indent, cjk) + display_width(account, cjk);
    let num_width = display_width(number, cjk);
    // Currency starts at currency_column (1-indexed).
    // Layout: indent + account + spaces + number + " " + currency
    // display_width before currency = prefix_width + padding + num_width + 1 (space)
    // We want that to equal currency_column - 1 (0-indexed position).
    // padding = currency_column - 1 - prefix_width - num_width - 1
    let min_before = prefix_width + num_width + 1; // with 0 padding
    let padding = if options.currency_column > min_before + 2 {
        options.currency_column - 1 - min_before
    } else {
        2 // minimum 2 spaces between account and number
    };

    let mut result = format!("{indent}{account}{:padding$}{number} {currency}", "");

    // Align cost to cost_column if present
    if let Some(cost) = cost {
        let current_width = display_width(&result, cjk);
        let cost_padding = if options.cost_column > current_width + 1 {
            options.cost_column - current_width - 1
        } else {
            1
        };
        result = format!("{result}{:cost_padding$} {cost}", "");
    }

    if let Some(price) = price {
        result = format!("{result} {price}");
    }

    if let Some(comment) = comment {
        result = format!("{result} {comment}");
    }

    result
}

/// Align a balance directive so currency starts at `currency_column`.
pub fn align_balance(
    date: &str,
    account: &str,
    number: &str,
    currency: &str,
    options: &Options,
) -> String {
    let cjk = options.fixed_cjk_width;
    let prefix = format!("{date} balance {account}");
    let prefix_width = display_width(&prefix, cjk);
    let num_width = display_width(number, cjk);

    let min_before = prefix_width + num_width + 1;
    let padding = if options.currency_column > min_before + 2 {
        options.currency_column - 1 - min_before
    } else {
        2
    };

    format!("{prefix}{:padding$}{number} {currency}", "")
}

/// Align an open directive so currencies start at `currency_column`.
pub fn align_open(
    date: &str,
    account: &str,
    currencies: &str,
    options: &Options,
) -> String {
    if currencies.is_empty() {
        return format!("{date} open {account}");
    }

    let cjk = options.fixed_cjk_width;
    let prefix = format!("{date} open {account}");
    let prefix_width = display_width(&prefix, cjk);

    let padding = if options.currency_column > prefix_width + 2 {
        options.currency_column - 1 - prefix_width
    } else {
        2
    };

    format!("{prefix}{:padding$}{currencies}", "")
}

/// Align a price directive so currency starts at `currency_column`.
pub fn align_price(
    date: &str,
    commodity: &str,
    number: &str,
    currency: &str,
    options: &Options,
) -> String {
    let cjk = options.fixed_cjk_width;
    let prefix = format!("{date} price {commodity}");
    let prefix_width = display_width(&prefix, cjk);
    let num_width = display_width(number, cjk);

    let min_before = prefix_width + num_width + 1;
    let padding = if options.currency_column > min_before + 2 {
        options.currency_column - 1 - min_before
    } else {
        2
    };

    format!("{prefix}{:padding$}{number} {currency}", "")
}
