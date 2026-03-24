use wasm_bindgen::prelude::*;

use crate::options::{Options, SortOrder, ThousandsSeparator};

fn parse_thousands(s: &str) -> Result<ThousandsSeparator, JsError> {
    match s.to_ascii_lowercase().as_str() {
        "add" => Ok(ThousandsSeparator::Add),
        "remove" => Ok(ThousandsSeparator::Remove),
        "keep" => Ok(ThousandsSeparator::Keep),
        other => Err(JsError::new(&format!(
            "invalid thousands: {other:?}, expected \"add\", \"remove\", or \"keep\""
        ))),
    }
}

fn parse_sort(s: &str) -> Result<SortOrder, JsError> {
    s.parse().map_err(|msg: String| JsError::new(&msg))
}

/// Format a beancount document with full options.
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn format(
    input: &str,
    indent: usize,
    currency_column: usize,
    cost_column: usize,
    thousands: &str,
    spaces_in_braces: bool,
    fixed_cjk_width: bool,
    sort: &str,
) -> Result<String, JsError> {
    let options = Options {
        indent,
        currency_column,
        cost_column,
        thousands_separator: parse_thousands(thousands)?,
        spaces_in_braces,
        fixed_cjk_width,
        sort: parse_sort(sort)?,
    };

    Ok(crate::format(input, &options))
}

/// Format with default options (convenience function).
#[wasm_bindgen]
pub fn format_default(input: &str) -> String {
    crate::format(input, &Options::default())
}
