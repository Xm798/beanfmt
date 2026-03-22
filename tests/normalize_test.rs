use husk::normalize::*;
use husk::options::ThousandsSeparator;

// normalize_indent tests

#[test]
fn indent_posting_line() {
    let result = normalize_indent("  Expenses:Food  100 USD", "    ");
    assert_eq!(result, "    Expenses:Food  100 USD");
}

#[test]
fn indent_tab_indent() {
    let result = normalize_indent("\tExpenses:Food  100 USD", "    ");
    assert_eq!(result, "    Expenses:Food  100 USD");
}

#[test]
fn indent_non_indented_passthrough() {
    let result = normalize_indent("2024-01-01 open Assets:Bank", "    ");
    assert_eq!(result, "2024-01-01 open Assets:Bank");
}

#[test]
fn indent_blank_passthrough() {
    assert_eq!(normalize_indent("", "    "), "");
    assert_eq!(normalize_indent("   ", "    "), "   ");
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
    assert_eq!(
        normalize_thousands("999", &ThousandsSeparator::Add),
        "999"
    );
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
