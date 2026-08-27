use beanfmt::normalize::*;
use beanfmt::options::{DecimalMode, Options, ThousandsSeparator};

// normalize_indent tests

#[test]
fn indent_posting_line() {
    let result = normalize_indent("  Expenses:Food  100 USD", 4);
    assert_eq!(result, "    Expenses:Food  100 USD");
}

#[test]
fn indent_tab_indent() {
    let result = normalize_indent("\tExpenses:Food  100 USD", 4);
    assert_eq!(result, "    Expenses:Food  100 USD");
}

#[test]
fn indent_non_indented_passthrough() {
    let result = normalize_indent("2024-01-01 open Assets:Bank", 4);
    assert_eq!(result, "2024-01-01 open Assets:Bank");
}

#[test]
fn indent_blank_passthrough() {
    assert_eq!(normalize_indent("", 4), "");
    assert_eq!(normalize_indent("   ", 4), "   ");
}

// normalize_comment tests

#[test]
fn comment_no_space_after_semicolon() {
    assert_eq!(normalize_comment(";hello"), "; hello");
}

#[test]
fn comment_extra_spaces() {
    assert_eq!(normalize_comment(";    hello world"), "; hello world");
}

#[test]
fn comment_double_semicolon_narration() {
    assert_eq!(normalize_comment("  ;;  narration"), "  ;; narration");
}

#[test]
fn comment_already_correct() {
    assert_eq!(normalize_comment("; hello"), "; hello");
}

#[test]
fn comment_empty_content() {
    assert_eq!(normalize_comment(";"), ";");
    assert_eq!(normalize_comment(";;"), ";;");
    assert_eq!(normalize_comment(";   "), ";");
}

#[test]
fn comment_triple_semicolon_not_mangled() {
    assert_eq!(normalize_comment(";;; B"), ";;; B");
    assert_eq!(normalize_comment(";;;;deep"), ";;;; deep");
}

#[test]
fn comment_aligned_pads_semicolons() {
    // Pad the semicolon prefix to the target width before the single content space.
    assert_eq!(normalize_comment_aligned("; C", 3), ";   C");
    assert_eq!(normalize_comment_aligned(";; A", 3), ";;  A");
    assert_eq!(normalize_comment_aligned(";;; B", 3), ";;; B");
}

#[test]
fn comment_aligned_empty_content_no_trailing() {
    assert_eq!(normalize_comment_aligned(";", 3), ";");
    assert_eq!(normalize_comment_aligned("  ;;", 3), "  ;;");
}

// normalize_thousands Add tests

#[test]
fn thousands_add_large_number() {
    assert_eq!(
        normalize_thousands("1234567.89", &ThousandsSeparator::Add),
        "1,234,567.89"
    );
}

#[test]
fn thousands_add_small_number() {
    assert_eq!(normalize_thousands("999", &ThousandsSeparator::Add), "999");
}

#[test]
fn thousands_add_negative() {
    assert_eq!(
        normalize_thousands("-50000", &ThousandsSeparator::Add),
        "-50,000"
    );
}

// normalize_thousands Remove tests

#[test]
fn thousands_remove_strips_commas() {
    assert_eq!(
        normalize_thousands("1,234,567.89", &ThousandsSeparator::Remove),
        "1234567.89"
    );
}

// normalize_thousands Keep tests

#[test]
fn thousands_keep_unchanged() {
    assert_eq!(
        normalize_thousands("1,234", &ThousandsSeparator::Keep),
        "1,234"
    );
    assert_eq!(
        normalize_thousands("1234", &ThousandsSeparator::Keep),
        "1234"
    );
}

// normalize_braces tests

#[test]
fn braces_add_spaces() {
    assert_eq!(normalize_braces("{100 USD}", true), "{ 100 USD }");
}

#[test]
fn braces_disabled_no_change() {
    assert_eq!(normalize_braces("{100 USD}", false), "{100 USD}");
}

#[test]
fn braces_empty_unchanged() {
    assert_eq!(normalize_braces("{}", true), "{}");
}

#[test]
fn braces_total_cost_double() {
    assert_eq!(normalize_braces("{{100 USD}}", true), "{{ 100 USD }}");
}

// normalize_thousands boundary tests

#[test]
fn thousands_add_exactly_four_digits() {
    assert_eq!(
        normalize_thousands("1000", &ThousandsSeparator::Add),
        "1,000"
    );
}

#[test]
fn thousands_add_six_digits() {
    assert_eq!(
        normalize_thousands("123456", &ThousandsSeparator::Add),
        "123,456"
    );
}

#[test]
fn thousands_add_positive_sign() {
    assert_eq!(
        normalize_thousands("+50000", &ThousandsSeparator::Add),
        "+50,000"
    );
}

#[test]
fn thousands_add_integer_no_decimal() {
    assert_eq!(
        normalize_thousands("1234567", &ThousandsSeparator::Add),
        "1,234,567"
    );
}

#[test]
fn thousands_add_zero() {
    assert_eq!(normalize_thousands("0", &ThousandsSeparator::Add), "0");
}

#[test]
fn thousands_add_existing_wrong_commas() {
    assert_eq!(
        normalize_thousands("12,34,567.89", &ThousandsSeparator::Add),
        "1,234,567.89"
    );
}

// normalize_decimals tests

#[test]
fn decimals_keep_unchanged() {
    assert_eq!(normalize_decimals("5.60", DecimalMode::Keep, 2), "5.60");
    assert_eq!(normalize_decimals("5", DecimalMode::Keep, 2), "5");
    assert_eq!(normalize_decimals("- 5.6", DecimalMode::Keep, 2), "- 5.6");
}

#[test]
fn decimals_minimal_strips_trailing_zeros() {
    let cases = [
        ("5.60", "5.6"),
        ("5.00", "5"),
        ("5", "5"),
        ("5.6", "5.6"),
        ("-0.50", "-0.5"),
        ("-100.000", "-100"),
        ("- 5.60", "-5.6"),
        ("1,234.50", "1,234.5"),
    ];
    for (input, expected) in cases {
        assert_eq!(
            normalize_decimals(input, DecimalMode::Minimal, 2),
            expected,
            "input: {input}"
        );
    }
}

#[test]
fn decimals_pad_shorter_fraction() {
    let cases = [
        ("5.6", "5.60"),
        ("5", "5.00"),
        ("-0.5", "-0.50"),
        ("- 5.6", "-5.60"),
        ("1,234.5", "1,234.50"),
    ];
    for (input, expected) in cases {
        assert_eq!(
            normalize_decimals(input, DecimalMode::Pad, 2),
            expected,
            "input: {input}"
        );
    }
}

#[test]
fn decimals_pad_leaves_longer_fraction() {
    assert_eq!(normalize_decimals("5.123", DecimalMode::Pad, 2), "5.123");
    assert_eq!(normalize_decimals("5.12", DecimalMode::Pad, 2), "5.12");
}

#[test]
fn decimals_pad_zero_places_leaves_integer() {
    assert_eq!(normalize_decimals("5", DecimalMode::Pad, 0), "5");
    assert_eq!(normalize_decimals("5.6", DecimalMode::Pad, 0), "5.6");
}

#[test]
fn decimals_pad_custom_places() {
    assert_eq!(normalize_decimals("5.6", DecimalMode::Pad, 4), "5.6000");
}

// normalize_amount tests

fn amount_opts(mode: DecimalMode, thousands: ThousandsSeparator) -> Options {
    Options {
        decimal_mode: mode,
        thousands_separator: thousands,
        ..Options::default()
    }
}

#[test]
fn amount_decimals_in_cost() {
    let opts = amount_opts(DecimalMode::Pad, ThousandsSeparator::Keep);
    assert_eq!(normalize_amount("{100.5 USD}", &opts), "{100.50 USD}");
    assert_eq!(normalize_amount("{{100.5 USD}}", &opts), "{{100.50 USD}}");
}

#[test]
fn amount_decimals_in_cost_with_date() {
    let opts = amount_opts(DecimalMode::Pad, ThousandsSeparator::Keep);
    assert_eq!(
        normalize_amount("{100.5 USD, 2024-01-01}", &opts),
        "{100.50 USD, 2024-01-01}"
    );
}

#[test]
fn amount_decimals_leaves_bare_date_cost() {
    let opts = amount_opts(DecimalMode::Pad, ThousandsSeparator::Keep);
    assert_eq!(normalize_amount("{2024-01-01}", &opts), "{2024-01-01}");
}

#[test]
fn amount_decimals_in_price() {
    let opts = amount_opts(DecimalMode::Pad, ThousandsSeparator::Keep);
    assert_eq!(normalize_amount("@ 7.1 CNY", &opts), "@ 7.10 CNY");

    let opts = amount_opts(DecimalMode::Minimal, ThousandsSeparator::Keep);
    assert_eq!(normalize_amount("@@ 7.100 CNY", &opts), "@@ 7.1 CNY");
}

#[test]
fn amount_thousands_add_in_cost() {
    let opts = amount_opts(DecimalMode::Keep, ThousandsSeparator::Add);
    assert_eq!(normalize_amount("{1500 CNY}", &opts), "{1,500 CNY}");
    assert_eq!(
        normalize_amount("{{1500000 CNY}}", &opts),
        "{{1,500,000 CNY}}"
    );
}

#[test]
fn amount_thousands_add_in_price() {
    let opts = amount_opts(DecimalMode::Keep, ThousandsSeparator::Add);
    assert_eq!(normalize_amount("@ 1000.5 CNY", &opts), "@ 1,000.5 CNY");
    assert_eq!(normalize_amount("@@ 12345 CNY", &opts), "@@ 12,345 CNY");
}

#[test]
fn amount_thousands_remove_in_cost() {
    let opts = amount_opts(DecimalMode::Keep, ThousandsSeparator::Remove);
    assert_eq!(normalize_amount("{1,500 CNY}", &opts), "{1500 CNY}");
}

#[test]
fn amount_thousands_and_decimals_compose() {
    let opts = amount_opts(DecimalMode::Pad, ThousandsSeparator::Add);
    assert_eq!(normalize_amount("{1500 CNY}", &opts), "{1,500.00 CNY}");
    assert_eq!(normalize_amount("@ 1000.5 CNY", &opts), "@ 1,000.50 CNY");
}

#[test]
fn amount_keep_defaults_are_identity() {
    let opts = Options::default();
    assert_eq!(normalize_amount("{100.5 USD}", &opts), "{100.5 USD}");
    assert!(matches!(
        normalize_amount("{100.5 USD}", &opts),
        std::borrow::Cow::Borrowed(_)
    ));
}
