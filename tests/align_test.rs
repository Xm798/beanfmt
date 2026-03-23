use beanfmt::align::*;
use beanfmt::options::Options;

fn opts(currency_col: usize, cost_col: usize) -> Options {
    Options {
        currency_column: currency_col,
        cost_column: cost_col,
        fixed_cjk_width: true,
        ..Options::default()
    }
}

// --- display_width ---

#[test]
fn display_width_ascii() {
    assert_eq!(display_width("hello", true), 5);
}

#[test]
fn display_width_cjk() {
    // Each CJK character occupies 2 columns
    assert_eq!(display_width("日本語", true), 6);
    assert_eq!(display_width("A日B", true), 4);
}

#[test]
fn display_width_fixed_cjk_false() {
    // When fixed_cjk_width is false, each char counts as 1
    assert_eq!(display_width("日本語", false), 3);
    assert_eq!(display_width("A日B", false), 3);
}

// --- pad_to_width ---

#[test]
fn pad_to_width_ascii() {
    let result = pad_to_width("hello", 10, true);
    assert_eq!(result, "hello     ");
    assert_eq!(result.len(), 10);
}

#[test]
fn pad_to_width_cjk() {
    // "日本" is 4 display columns, pad to 10 needs 6 spaces
    let result = pad_to_width("日本", 10, true);
    assert_eq!(display_width(&result, true), 10);
    assert!(result.starts_with("日本"));
    assert!(result.ends_with("      ")); // 6 spaces
}

#[test]
fn pad_to_width_already_wider() {
    let result = pad_to_width("very long string", 5, true);
    assert_eq!(result, "very long string ");
}

// --- align_posting ---

#[test]
fn align_posting_basic() {
    let opts = opts(50, 55);
    let result = align_posting(
        "    ",
        "Assets:Bank:Checking",
        Some("100.00"),
        Some("USD"),
        None,
        None,
        None,
        &opts,
    );
    // Currency "USD" should start at column 50
    let currency_pos = display_width(&result[..result.find("USD").unwrap()], true) + 1;
    assert_eq!(currency_pos, 50);
}

#[test]
fn align_posting_cjk_account() {
    let opts = opts(50, 55);
    // CJK account takes more display columns, alignment should compensate
    let result_cjk = align_posting(
        "    ",
        "Assets:銀行:普通",
        Some("100.00"),
        Some("USD"),
        None,
        None,
        None,
        &opts,
    );
    let result_ascii = align_posting(
        "    ",
        "Assets:Bank:Normal",
        Some("100.00"),
        Some("USD"),
        None,
        None,
        None,
        &opts,
    );
    // Both should have currency at column 50
    let cjk_pos = display_width(&result_cjk[..result_cjk.find("USD").unwrap()], true) + 1;
    let ascii_pos = display_width(&result_ascii[..result_ascii.find("USD").unwrap()], true) + 1;
    assert_eq!(cjk_pos, 50);
    assert_eq!(ascii_pos, 50);
}

#[test]
fn align_posting_account_only() {
    let opts = opts(50, 55);
    let result = align_posting(
        "    ",
        "Assets:Bank:Checking",
        None,
        None,
        None,
        None,
        None,
        &opts,
    );
    assert_eq!(result, "    Assets:Bank:Checking");
    // No trailing spaces
    assert!(!result.ends_with(' '));
}

#[test]
fn align_posting_with_cost_and_price() {
    let opts = opts(50, 60);
    let result = align_posting(
        "    ",
        "Assets:Stock",
        Some("10"),
        Some("AAPL"),
        Some("{150.00 USD}"),
        Some("@ 155.00 USD"),
        None,
        &opts,
    );
    assert!(result.contains("AAPL"));
    assert!(result.contains("{150.00 USD}"));
    assert!(result.contains("@ 155.00 USD"));
    // Verify cost starts at cost_column (consistent with currency_column convention)
    let cost_start = result.find("{150.00 USD}").unwrap();
    let before_cost = &result[..cost_start];
    let cost_pos = display_width(before_cost, true) + 1;
    assert_eq!(cost_pos, 60, "cost should start at cost_column={}", 60);
}

#[test]
fn align_posting_with_comment() {
    let opts = opts(50, 55);
    let result = align_posting(
        "    ",
        "Expenses:Food",
        Some("25.00"),
        Some("USD"),
        None,
        None,
        Some("; lunch"),
        &opts,
    );
    assert!(result.ends_with("; lunch"));
}

// --- align_balance ---

#[test]
fn align_balance_basic() {
    let opts = opts(50, 55);
    let result = align_balance("2024-01-01", "Assets:Bank", "1000.00", "USD", &opts);
    let currency_pos = display_width(&result[..result.find("USD").unwrap()], true) + 1;
    assert_eq!(currency_pos, 50);
}

// --- align_open ---

#[test]
fn align_open_basic() {
    let opts = opts(50, 55);
    let result = align_open("2024-01-01", "Assets:Bank:Checking", "USD,EUR", &opts);
    let currencies_pos = display_width(&result[..result.find("USD,EUR").unwrap()], true) + 1;
    assert_eq!(currencies_pos, 50);
}

#[test]
fn align_open_no_currencies() {
    let opts = opts(50, 55);
    let result = align_open("2024-01-01", "Assets:Bank:Checking", "", &opts);
    assert_eq!(result, "2024-01-01 open Assets:Bank:Checking");
}

// --- align_price ---

#[test]
fn align_price_basic() {
    let opts = opts(50, 55);
    let result = align_price("2024-01-01", "USD", "6.89", "CNY", &opts);
    let currency_pos = display_width(&result[..result.find("CNY").unwrap()], true) + 1;
    assert_eq!(currency_pos, 50);
}

// --- Edge cases ---

#[test]
fn align_posting_negative_number() {
    let opts = opts(50, 55);
    let result = align_posting(
        "    ",
        "Assets:Bank:Checking",
        Some("-500.00"),
        Some("USD"),
        None,
        None,
        None,
        &opts,
    );
    assert!(result.contains("-500.00"));
    let currency_pos = display_width(&result[..result.find("USD").unwrap()], true) + 1;
    assert_eq!(currency_pos, 50);
}

#[test]
fn align_posting_long_account_exceeds_column() {
    let opts = opts(30, 35);
    // Account is very long, exceeding currency_column
    let result = align_posting(
        "    ",
        "Assets:VeryLongBankName:VeryLongSubAccount",
        Some("100.00"),
        Some("USD"),
        None,
        None,
        None,
        &opts,
    );
    // Should still have at least 2 spaces between account and number
    let account_end = "    Assets:VeryLongBankName:VeryLongSubAccount";
    assert!(result.starts_with(account_end));
    let after_account = &result[account_end.len()..];
    // Count leading spaces before the number
    let spaces = after_account.len() - after_account.trim_start().len();
    assert!(spaces >= 2, "expected at least 2 spaces, got {spaces}");
}
