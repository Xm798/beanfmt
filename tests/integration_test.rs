use husk::format;
use husk::options::{Options, ThousandsSeparator};

fn default_opts() -> Options {
    Options::default()
}

#[test]
fn empty_input() {
    assert_eq!(format("", &default_opts()), "");
}

#[test]
fn trailing_newline_preserved() {
    let input = "option \"title\" \"Test\"\n";
    let result = format(input, &default_opts());
    assert!(result.ends_with('\n'));
}

#[test]
fn trailing_newline_absent() {
    let input = "option \"title\" \"Test\"";
    let result = format(input, &default_opts());
    assert!(!result.ends_with('\n'));
}

#[test]
fn passthrough_lines_unchanged() {
    let input = "option \"title\" \"Test\"\nplugin \"auto_accounts\"\n";
    let result = format(input, &default_opts());
    assert_eq!(result, input);
}

#[test]
fn basic_transaction_with_postings() {
    let input = "2024-01-20 * \"Shop\" \"Groceries\"\n  Expenses:Food  50.00 USD\n  Assets:Bank  -50.00 USD\n";
    let result = format(input, &default_opts());

    // Each posting should use the configured indent (4 spaces)
    for line in result.lines().skip(1) {
        assert!(line.starts_with("    "), "posting should start with 4-space indent: {:?}", line);
    }
}

#[test]
fn posting_currency_alignment() {
    let opts = Options {
        currency_column: 50,
        ..default_opts()
    };
    let input = "2024-01-01 * \"Test\"\n  Expenses:Food  10.00 USD\n  Expenses:Transportation  200.00 USD\n";
    let result = format(input, &opts);
    let lines: Vec<&str> = result.lines().collect();

    // Both postings should have USD at the same column
    let pos1 = lines[1].find("USD").unwrap();
    let pos2 = lines[2].find("USD").unwrap();
    assert_eq!(pos1, pos2, "currencies should align at same column");
}

#[test]
fn balance_alignment() {
    let opts = default_opts();
    let input = "2024-01-31 balance Assets:Bank:Checking 1000.00 USD\n";
    let result = format(input, &opts);
    assert!(result.contains("balance"));
    assert!(result.contains("1000.00 USD"));
}

#[test]
fn open_alignment() {
    let opts = default_opts();
    let input = "2024-01-15 open Assets:Bank:Checking USD\n";
    let result = format(input, &opts);
    assert!(result.starts_with("2024-01-15 open Assets:Bank:Checking"));
    assert!(result.contains("USD"));
}

#[test]
fn price_alignment() {
    let opts = default_opts();
    let input = "2024-02-01 price AAPL 185.50 USD\n";
    let result = format(input, &opts);
    assert!(result.contains("price AAPL"));
    assert!(result.contains("185.50 USD"));
}

#[test]
fn close_passthrough() {
    let input = "2024-01-15 close Expenses:Food\n";
    let result = format(input, &default_opts());
    assert_eq!(result, "2024-01-15 close Expenses:Food\n");
}

#[test]
fn comment_normalization() {
    let input = ";   hello world\n";
    let result = format(input, &default_opts());
    assert_eq!(result, "; hello world\n");
}

#[test]
fn comment_empty_content() {
    let input = ";  \n";
    let result = format(input, &default_opts());
    assert_eq!(result, ";\n");
}

#[test]
fn blank_lines_become_empty() {
    let input = "option \"title\" \"X\"\n\n2024-01-01 open Assets:Bank USD\n";
    let result = format(input, &default_opts());
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines[1], "");
}

#[test]
fn meta_item_uses_configured_indent() {
    let opts = Options {
        indent: "  ".to_string(),
        ..default_opts()
    };
    let input = "2024-01-01 * \"Test\"\n    filename: \"test.txt\"\n";
    let result = format(input, &opts);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines[1], "  filename: \"test.txt\"");
}

#[test]
fn thousands_separator_add() {
    let opts = Options {
        thousands_separator: ThousandsSeparator::Add,
        ..default_opts()
    };
    let input = "2024-01-31 balance Assets:Bank 1000000.00 USD\n";
    let result = format(input, &opts);
    assert!(result.contains("1,000,000.00"), "should add thousands separators: {}", result);
    // Date must NOT be corrupted
    assert!(result.starts_with("2024-01-31"), "date must not be corrupted");
}

#[test]
fn thousands_separator_remove() {
    let opts = Options {
        thousands_separator: ThousandsSeparator::Remove,
        ..default_opts()
    };
    let input = "2024-01-31 balance Assets:Bank 1,000.00 USD\n";
    let result = format(input, &opts);
    assert!(result.contains("1000.00"), "should remove commas: {}", result);
}

#[test]
fn posting_with_cost_and_price() {
    let opts = default_opts();
    let input = "2024-01-01 * \"Buy stock\"\n  Assets:Brokerage  10 AAPL {185.50 USD} @ 185.50 USD\n";
    let result = format(input, &opts);
    assert!(result.contains("AAPL"));
    assert!(result.contains("{185.50 USD}"));
    assert!(result.contains("@ 185.50 USD"));
}

#[test]
fn cjk_account_alignment() {
    let opts = Options {
        currency_column: 50,
        fixed_cjk_width: true,
        ..default_opts()
    };
    // CJK characters are double-width, so alignment should account for that
    let input = "2024-01-01 * \"Test\"\n  Expenses:\u{98df}\u{54c1}  100.00 JPY\n  Assets:\u{9280}\u{884c}  -100.00 JPY\n";
    let result = format(input, &opts);
    let lines: Vec<&str> = result.lines().collect();

    let pos1 = lines[1].find("JPY").unwrap();
    let pos2 = lines[2].find("JPY").unwrap();
    assert_eq!(pos1, pos2, "CJK postings should align currencies at same column");
}

#[test]
fn include_passthrough() {
    let input = "include \"other.beancount\"\n";
    let result = format(input, &default_opts());
    assert_eq!(result, input);
}

#[test]
fn block_directive_passthrough() {
    let input = "pushtag #trip\n";
    let result = format(input, &default_opts());
    assert_eq!(result, input);
}

#[test]
fn multiple_postings_consistent_alignment() {
    let opts = Options {
        currency_column: 60,
        ..default_opts()
    };
    let input = concat!(
        "2024-01-01 * \"Multi\"\n",
        "  Expenses:Food  10.00 USD\n",
        "  Expenses:Transport  25.00 USD\n",
        "  Expenses:Entertainment  150.00 USD\n",
        "  Assets:Bank  -185.00 USD\n",
    );
    let result = format(input, &opts);
    let posting_lines: Vec<&str> = result.lines().skip(1).collect();

    let positions: Vec<usize> = posting_lines
        .iter()
        .map(|l| l.find("USD").expect("should contain USD"))
        .collect();

    // All USD should be at the same column
    assert!(
        positions.windows(2).all(|w| w[0] == w[1]),
        "all currencies should be at same column: {:?}",
        positions
    );
}

#[test]
fn spaces_in_braces() {
    let opts = Options {
        spaces_in_braces: true,
        ..default_opts()
    };
    let input = "2024-01-01 * \"Buy\"\n  Assets:Brokerage  10 AAPL {185.50 USD}\n";
    let result = format(input, &opts);
    assert!(result.contains("{ 185.50 USD }"), "should add spaces in braces: {}", result);
}

#[test]
fn format_with_sort() {
    let input = "\
2024-01-03 * \"C\" \"C\"
    Expenses:C  30 USD
    Assets:Bank

2024-01-01 * \"A\" \"A\"
    Expenses:A  10 USD
    Assets:Bank
";
    let opts = Options {
        sort: true,
        currency_column: 50,
        ..Options::default()
    };
    let result = format(input, &opts);
    let dates: Vec<&str> = result
        .lines()
        .filter(|l| l.starts_with("2024"))
        .map(|l| &l[..10])
        .collect();
    assert_eq!(dates, vec!["2024-01-01", "2024-01-03"]);
}

#[test]
fn format_cjk_fixture() {
    let input = include_str!("fixtures/cjk.beancount");
    let opts = Options {
        currency_column: 50,
        ..Options::default()
    };
    let result = format(input, &opts);
    for line in result.lines() {
        if line.contains("CNY") && line.starts_with("    ") {
            let cny_start = line.find("CNY").unwrap();
            let before = &line[..cny_start];
            let width = husk::align::display_width(before, true);
            // currency_column is 1-indexed, so 0-indexed position is column - 1
            assert_eq!(width, 49, "CJK line not aligned: {}", line);
        }
    }
}

#[test]
fn format_normalize_fixture() {
    let input = include_str!("fixtures/normalize.beancount");
    let opts = Options {
        thousands_separator: ThousandsSeparator::Add,
        spaces_in_braces: true,
        ..Options::default()
    };
    let result = format(input, &opts);
    assert!(result.contains("; comment without space"), "Comment should be normalized");
    assert!(result.contains(";; narration with extra spaces"), "Narration should be normalized");
    assert!(result.contains("1,234,567.89"), "Thousands should be added");
    assert!(result.contains("{ 150 USD }"), "Braces should have spaces");
}

#[test]
fn format_without_sort_preserves_order() {
    let input = "\
2024-01-03 * \"C\" \"C\"
    Expenses:C  30 USD
    Assets:Bank

2024-01-01 * \"A\" \"A\"
    Expenses:A  10 USD
    Assets:Bank
";
    let opts = Options {
        sort: false,
        currency_column: 50,
        ..Options::default()
    };
    let result = format(input, &opts);
    let dates: Vec<&str> = result
        .lines()
        .filter(|l| l.starts_with("2024"))
        .map(|l| &l[..10])
        .collect();
    assert_eq!(dates, vec!["2024-01-03", "2024-01-01"]);
}
