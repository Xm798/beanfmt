use beanfmt::format;
use beanfmt::options::{Options, SortOrder, ThousandsSeparator, TimelessPosition};

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

    // Each posting should use the configured indent (default 2 spaces)
    for line in result.lines().skip(1) {
        assert!(
            line.starts_with("  "),
            "posting should start with 2-space indent: {:?}",
            line
        );
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
fn posting_no_space_between_number_and_currency() {
    let opts = default_opts();
    let input = "2025-12-21 * \"Test\" \"Interest\"\n    Income:Investment:Interest\n    Assets:Bank:Test:9999                                        0.04CNY\n";
    let result = format(input, &opts);
    assert!(
        result.contains("Assets:Bank:Test:9999"),
        "account should not be mangled: {result}"
    );
    assert!(
        result.contains("0.04 CNY"),
        "number and currency should be separated by space: {result}"
    );
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
fn posting_no_amount_with_comment() {
    let input = "2024-01-01 * \"Test\"\n  Assets:Bank ; reconciled\n";
    let result = format(input, &default_opts());
    assert!(
        result.contains("; reconciled"),
        "comment on no-amount posting should be preserved: {}",
        result
    );
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
fn block_comment_semicolon_alignment() {
    // A block of consecutive comments with differing semicolon counts aligns
    // content to the widest semicolon prefix (mirrors autobean-format).
    // https://github.com/SEIAROTg/autobean-format/issues/15
    let input = ";; A\n;;; B\n; C\n;;hello\n;four\n";
    let result = format(input, &default_opts());
    assert_eq!(result, ";;  A\n;;; B\n;   C\n;;  hello\n;   four\n");
}

#[test]
fn block_comment_triple_semicolon_preserved() {
    let input = ";;; section\n";
    let result = format(input, &default_opts());
    assert_eq!(result, ";;; section\n");
}

#[test]
fn block_comment_alignment_resets_across_blank() {
    // A blank line ends a block; each block aligns independently.
    let input = "; a\n;; b\n\n;; c\n; d\n";
    let result = format(input, &default_opts());
    assert_eq!(result, ";  a\n;; b\n\n;; c\n;  d\n");
}

#[test]
fn block_comment_uniform_semicolons_unchanged() {
    let input = "; one\n; two\n; three\n";
    let result = format(input, &default_opts());
    assert_eq!(result, "; one\n; two\n; three\n");
}

#[test]
fn block_comment_indented_block_aligns_independently() {
    // A change of indentation splits the run: the top-level block aligns to its
    // own widest prefix (2), the indented block to its own (3) — no bleed across.
    let input = ";; top1\n; top2\n    ; in1\n    ;;; in3\n";
    let result = format(input, &default_opts());
    assert_eq!(result, ";; top1\n;  top2\n    ;   in1\n    ;;; in3\n");
}

#[test]
fn block_comment_empty_line_bridges_block() {
    // An empty ";" line continues the run (so the first line still aligns to the
    // block max of 3) yet is itself emitted unpadded with no trailing space.
    let input = "; A\n;\n;;; B\n";
    let result = format(input, &default_opts());
    assert_eq!(result, ";   A\n;\n;;; B\n");
}

#[test]
fn block_comment_alignment_idempotent() {
    let input = ";; A\n;;; B\n; C\n;;hello\n;four\n";
    let once = format(input, &default_opts());
    let twice = format(&once, &default_opts());
    assert_eq!(once, twice, "block alignment must be idempotent");
    assert_eq!(once, ";;  A\n;;; B\n;   C\n;;  hello\n;   four\n");
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
        indent: 2,
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
    assert!(
        result.contains("1,000,000.00"),
        "should add thousands separators: {}",
        result
    );
    // Date must NOT be corrupted
    assert!(
        result.starts_with("2024-01-31"),
        "date must not be corrupted"
    );
}

#[test]
fn thousands_separator_remove() {
    let opts = Options {
        thousands_separator: ThousandsSeparator::Remove,
        ..default_opts()
    };
    let input = "2024-01-31 balance Assets:Bank 1,000.00 USD\n";
    let result = format(input, &opts);
    assert!(
        result.contains("1000.00"),
        "should remove commas: {}",
        result
    );
}

#[test]
fn posting_with_cost_and_price() {
    let opts = default_opts();
    let input =
        "2024-01-01 * \"Buy stock\"\n  Assets:Brokerage  10 AAPL {185.50 USD} @ 185.50 USD\n";
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
    assert_eq!(
        pos1, pos2,
        "CJK postings should align currencies at same column"
    );
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
    assert!(
        result.contains("{ 185.50 USD }"),
        "should add spaces in braces: {}",
        result
    );
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
        sort: SortOrder::Asc,
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
fn format_with_sort_desc() {
    let input = "\
2024-01-01 * \"A\" \"A\"
    Expenses:A  10 USD
    Assets:Bank

2024-01-03 * \"C\" \"C\"
    Expenses:C  30 USD
    Assets:Bank
";
    let opts = Options {
        sort: SortOrder::Desc,
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

#[test]
fn format_with_sort_timeless_end() {
    let input = "\
2024-01-01 * \"No time\"
    Assets:Bank  100 USD

2024-01-01 * \"Has time\"
    time: \"09:00\"
    Expenses:Food  100 USD
    Assets:Bank
";
    let opts = Options {
        sort: SortOrder::Asc,
        sort_timeless: TimelessPosition::End,
        currency_column: 50,
        ..Options::default()
    };
    let result = format(input, &opts);
    assert!(
        result.find("Has time").unwrap() < result.find("No time").unwrap(),
        "timed entry should come before timeless with End position"
    );
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
            let width = beanfmt::align::display_width(before, true);
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
    assert!(
        // Padded to width 2: this line forms a block with the following ";;" comment.
        result.contains(";  comment without space"),
        "Comment should be normalized"
    );
    assert!(
        result.contains(";; narration with extra spaces"),
        "Narration should be normalized"
    );
    assert!(result.contains("1,234,567.89"), "Thousands should be added");
    assert!(result.contains("{ 150 USD }"), "Braces should have spaces");
}

#[test]
fn format_idempotent() {
    let input = "\
2024-01-15 * \"Grocery Store\" \"Weekly shopping\"
  Expenses:Food     50.00 USD
  Assets:Bank      -50.00 USD

2024-01-16 balance Assets:Bank  1000.00 USD

2024-01-01 open Assets:Bank  USD

; A section comment
2024-02-01 * \"Restaurant\"
  Expenses:Food:DiningOut    35.50 USD
  Assets:CreditCard
";
    let opts = Options::default();
    let once = format(input, &opts);
    let twice = format(&once, &opts);
    assert_eq!(once, twice, "formatting should be idempotent");
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
        sort: SortOrder::Off,
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

#[test]
fn posting_with_spaced_negative_amount() {
    let input = "\
2024-01-01 * \"Shop\" \"Item\"
    time: \"20:16:47\"
    Liabilities:Credit:Bank                                     - 619.47 CNY
        source: \"import\"
    Expenses:Travel:Shopping                                  619.47 CNY
";
    let opts = Options {
        currency_column: 60,
        ..Options::default()
    };
    let result = format(input, &opts);

    // Bug 1: account name must not be broken (no space after colon)
    assert!(
        !result.contains("Liabilities: Credit"),
        "Account name should not have space after colon: {}",
        result
    );
    assert!(
        result.contains("Liabilities:Credit:Bank"),
        "Account name should be preserved: {}",
        result
    );

    // Bug 2: spaced negative should be normalized to no space
    assert!(
        result.contains("-619.47"),
        "Spaced negative '- 619.47' should be normalized to '-619.47': {}",
        result
    );

    // Bug 3: posting sub-metadata should have deeper indent than transaction metadata
    let lines: Vec<&str> = result.lines().collect();
    let card_line = lines.iter().find(|l| l.contains("source")).unwrap();
    assert!(
        card_line.starts_with("    "),
        "Posting sub-metadata should have double indent: '{}'",
        card_line
    );
}

#[test]
fn inline_comment_column_aligns_across_directives() {
    let input = concat!(
        "2024-01-01 open Assets:Bank USD ;opened\n",
        "2024-01-01 * \"Payee\"\n",
        "  Assets:Bank  10.00 USD ;deposit\n",
        "2024-01-02 balance Assets:Bank 10.00 USD ;checked\n",
        "2024-01-02 price USD 0.92 EUR ;fx\n",
        "2024-12-31 close Assets:Bank ;done\n",
    );
    let opts = Options {
        currency_column: 30,
        cost_column: 35,
        inline_comment_column: 50,
        ..Options::default()
    };
    let result = format(input, &opts);
    for line in result.lines() {
        if let Some(idx) = line.find(';') {
            assert_eq!(idx + 1, 50, "comment should start at column 50: {:?}", line);
        }
    }
}

#[test]
fn inline_comment_column_disabled_by_default() {
    let input = "2024-01-02 balance Assets:Bank 10.00 USD ;checked\n";
    let result = format(input, &default_opts());
    // No alignment: comment kept one space after the amount, not pushed to a column.
    assert!(result.contains("USD ;checked"), "got: {:?}", result);
}
