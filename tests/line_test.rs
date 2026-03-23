use beanfmt::line::{BlockKind, Line, parse_line};

#[test]
fn blank_line_empty() {
    assert_eq!(parse_line(""), Line::BlankLine);
}

#[test]
fn blank_line_whitespace() {
    assert_eq!(parse_line("   "), Line::BlankLine);
}

#[test]
fn transaction_header_star_flag() {
    let line = r#"2024-01-15 * "Grocery Store" "Weekly shopping""#;
    match parse_line(line) {
        Line::TransactionHeader { date, flag, rest } => {
            assert_eq!(date, "2024-01-15");
            assert_eq!(flag, "*");
            assert_eq!(rest, r#""Grocery Store" "Weekly shopping""#);
        }
        other => panic!("Expected TransactionHeader, got {:?}", other),
    }
}

#[test]
fn transaction_header_txn_flag() {
    let line = r#"2024-03-01 txn "Payee" "Narration""#;
    match parse_line(line) {
        Line::TransactionHeader { date, flag, rest } => {
            assert_eq!(date, "2024-03-01");
            assert_eq!(flag, "txn");
            assert_eq!(rest, r#""Payee" "Narration""#);
        }
        other => panic!("Expected TransactionHeader, got {:?}", other),
    }
}

#[test]
fn posting_with_amount() {
    let line = "  Expenses:Food  50.00 USD";
    match parse_line(line) {
        Line::Posting {
            indent,
            account,
            number,
            currency,
            cost,
            price,
            comment,
        } => {
            assert_eq!(indent, "  ");
            assert_eq!(account, "Expenses:Food");
            assert_eq!(number, Some("50.00"));
            assert_eq!(currency, Some("USD"));
            assert_eq!(cost, None);
            assert_eq!(price, None);
            assert_eq!(comment, None);
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_account_only() {
    let line = "  Assets:Bank:Checking";
    match parse_line(line) {
        Line::Posting {
            indent,
            account,
            number,
            currency,
            ..
        } => {
            assert_eq!(indent, "  ");
            assert_eq!(account, "Assets:Bank:Checking");
            assert_eq!(number, None);
            assert_eq!(currency, None);
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_with_cost_and_price() {
    let line = "  Assets:Stock  10 AAPL {150.00 USD} @ 155.00 USD";
    match parse_line(line) {
        Line::Posting {
            account,
            number,
            currency,
            cost,
            price,
            ..
        } => {
            assert_eq!(account, "Assets:Stock");
            assert_eq!(number, Some("10"));
            assert_eq!(currency, Some("AAPL"));
            assert_eq!(cost, Some("{150.00 USD}"));
            assert_eq!(price, Some("@ 155.00 USD"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_with_inline_comment() {
    let line = "  Expenses:Food  50.00 USD ; lunch";
    match parse_line(line) {
        Line::Posting {
            account,
            number,
            comment,
            ..
        } => {
            assert_eq!(account, "Expenses:Food");
            assert_eq!(number, Some("50.00"));
            assert_eq!(comment, Some("; lunch"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn balance_directive() {
    let line = "2024-01-01 balance Assets:Bank:Checking 1000.00 USD";
    match parse_line(line) {
        Line::Balance {
            date,
            account,
            number,
            currency,
        } => {
            assert_eq!(date, "2024-01-01");
            assert_eq!(account, "Assets:Bank:Checking");
            assert_eq!(number, "1000.00");
            assert_eq!(currency, "USD");
        }
        other => panic!("Expected Balance, got {:?}", other),
    }
}

#[test]
fn open_directive_with_currencies() {
    let line = "2024-01-01 open Assets:Bank:Checking USD,EUR";
    match parse_line(line) {
        Line::Open {
            date,
            account,
            currencies,
        } => {
            assert_eq!(date, "2024-01-01");
            assert_eq!(account, "Assets:Bank:Checking");
            assert_eq!(currencies, "USD,EUR");
        }
        other => panic!("Expected Open, got {:?}", other),
    }
}

#[test]
fn close_directive() {
    let line = "2024-12-31 close Expenses:Old";
    match parse_line(line) {
        Line::Close { date, account } => {
            assert_eq!(date, "2024-12-31");
            assert_eq!(account, "Expenses:Old");
        }
        other => panic!("Expected Close, got {:?}", other),
    }
}

#[test]
fn price_directive() {
    let line = "2024-01-15 price USD 0.92 EUR";
    match parse_line(line) {
        Line::Price {
            date,
            commodity,
            number,
            currency,
        } => {
            assert_eq!(date, "2024-01-15");
            assert_eq!(commodity, "USD");
            assert_eq!(number, "0.92");
            assert_eq!(currency, "EUR");
        }
        other => panic!("Expected Price, got {:?}", other),
    }
}

#[test]
fn meta_item() {
    let line = "    time: 10:30";
    match parse_line(line) {
        Line::MetaItem { indent, key, value } => {
            assert_eq!(indent, "    ");
            assert_eq!(key, "time");
            assert_eq!(value, "10:30");
        }
        other => panic!("Expected MetaItem, got {:?}", other),
    }
}

#[test]
fn comment_single_semicolon() {
    let line = "; This is a comment";
    match parse_line(line) {
        Line::Comment {
            indent,
            semicolons,
            content,
        } => {
            assert_eq!(indent, "");
            assert_eq!(semicolons, ";");
            assert_eq!(content, "This is a comment");
        }
        other => panic!("Expected Comment, got {:?}", other),
    }
}

#[test]
fn comment_double_semicolon() {
    let line = "  ;; Section header";
    match parse_line(line) {
        Line::Comment {
            indent,
            semicolons,
            content,
        } => {
            assert_eq!(indent, "  ");
            assert_eq!(semicolons, ";;");
            assert_eq!(content, "Section header");
        }
        other => panic!("Expected Comment, got {:?}", other),
    }
}

#[test]
fn block_directive_pushtag() {
    let line = "pushtag #trip";
    match parse_line(line) {
        Line::BlockDirective { kind, rest } => {
            assert_eq!(kind, BlockKind::PushTag);
            assert_eq!(rest, "#trip");
        }
        other => panic!("Expected BlockDirective, got {:?}", other),
    }
}

#[test]
fn block_directive_popmeta() {
    let line = "popmeta location:";
    match parse_line(line) {
        Line::BlockDirective { kind, rest } => {
            assert_eq!(kind, BlockKind::PopMeta);
            assert_eq!(rest, "location:");
        }
        other => panic!("Expected BlockDirective, got {:?}", other),
    }
}

#[test]
fn include_directive() {
    let line = r#"include "accounts.beancount""#;
    match parse_line(line) {
        Line::Include { path } => {
            assert_eq!(path, "accounts.beancount");
        }
        other => panic!("Expected Include, got {:?}", other),
    }
}

#[test]
fn option_falls_to_other() {
    let line = r#"option "title" "My Ledger""#;
    assert_eq!(parse_line(line), Line::Other(line));
}

#[test]
fn cjk_account_name() {
    let line = "  Expenses:餐饮:中餐  50.00 CNY";
    match parse_line(line) {
        Line::Posting {
            account,
            number,
            currency,
            ..
        } => {
            assert_eq!(account, "Expenses:餐饮:中餐");
            assert_eq!(number, Some("50.00"));
            assert_eq!(currency, Some("CNY"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

// === Adversarial tests: edge cases and potential bugs ===

#[test]
fn posting_with_cost_only_no_price() {
    let line = "    Assets:Stock  10 AAPL {150 USD}";
    match parse_line(line) {
        Line::Posting {
            account,
            number,
            currency,
            cost,
            price,
            comment,
            ..
        } => {
            assert_eq!(account, "Assets:Stock");
            assert_eq!(number, Some("10"));
            assert_eq!(currency, Some("AAPL"));
            assert_eq!(cost, Some("{150 USD}"));
            assert_eq!(price, None);
            assert_eq!(comment, None);
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_with_price_only_no_cost() {
    let line = "    Assets:Stock  10 AAPL @ 155 USD";
    match parse_line(line) {
        Line::Posting {
            account,
            number,
            currency,
            cost,
            price,
            comment,
            ..
        } => {
            assert_eq!(account, "Assets:Stock");
            assert_eq!(number, Some("10"));
            assert_eq!(currency, Some("AAPL"));
            assert_eq!(cost, None);
            assert_eq!(price, Some("@ 155 USD"));
            assert_eq!(comment, None);
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_with_total_cost() {
    let line = "    Assets:Stock  10 AAPL {{1500 USD}}";
    match parse_line(line) {
        Line::Posting { cost, price, .. } => {
            assert_eq!(cost, Some("{{1500 USD}}"));
            assert_eq!(price, None);
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_with_total_price() {
    let line = "    Assets:Stock  10 AAPL @@ 1550 USD";
    match parse_line(line) {
        Line::Posting { cost, price, .. } => {
            assert_eq!(cost, None);
            assert_eq!(price, Some("@@ 1550 USD"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_with_everything() {
    let line = "    Assets:Stock  10 AAPL {150 USD} @ 155 USD ; comment";
    match parse_line(line) {
        Line::Posting {
            account,
            number,
            currency,
            cost,
            price,
            comment,
            ..
        } => {
            assert_eq!(account, "Assets:Stock");
            assert_eq!(number, Some("10"));
            assert_eq!(currency, Some("AAPL"));
            assert_eq!(cost, Some("{150 USD}"));
            assert_eq!(price, Some("@ 155 USD"));
            assert_eq!(comment, Some("; comment"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_account_with_comment_only() {
    let line = "    Assets:Bank ; note";
    match parse_line(line) {
        Line::Posting {
            account,
            number,
            currency,
            comment,
            ..
        } => {
            assert_eq!(account, "Assets:Bank");
            assert_eq!(number, None);
            assert_eq!(currency, None);
            assert_eq!(comment, Some("; note"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_negative_number() {
    let line = "    Expenses:Food  -50.00 USD";
    match parse_line(line) {
        Line::Posting {
            number, currency, ..
        } => {
            assert_eq!(number, Some("-50.00"));
            assert_eq!(currency, Some("USD"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_number_with_commas() {
    let line = "    Expenses:Food  1,234.56 USD";
    match parse_line(line) {
        Line::Posting {
            number, currency, ..
        } => {
            assert_eq!(number, Some("1,234.56"));
            assert_eq!(currency, Some("USD"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn transaction_txn_flag() {
    let line = r#"2024-01-15 txn "payee" "narration""#;
    match parse_line(line) {
        Line::TransactionHeader { date, flag, rest } => {
            assert_eq!(date, "2024-01-15");
            assert_eq!(flag, "txn");
            assert_eq!(rest, r#""payee" "narration""#);
        }
        other => panic!("Expected TransactionHeader, got {:?}", other),
    }
}

#[test]
fn transaction_bang_flag() {
    let line = r#"2024-01-15 ! "payee" "narration""#;
    match parse_line(line) {
        Line::TransactionHeader { date, flag, rest } => {
            assert_eq!(date, "2024-01-15");
            assert_eq!(flag, "!");
            assert_eq!(rest, r#""payee" "narration""#);
        }
        other => panic!("Expected TransactionHeader, got {:?}", other),
    }
}

#[test]
fn open_without_currencies() {
    let line = "2024-01-15 open Assets:Bank";
    match parse_line(line) {
        Line::Open {
            date,
            account,
            currencies,
        } => {
            assert_eq!(date, "2024-01-15");
            assert_eq!(account, "Assets:Bank");
            assert_eq!(currencies, "");
        }
        other => panic!("Expected Open, got {:?}", other),
    }
}

#[test]
fn single_char_currency_rejected_by_currency_pattern() {
    // Single-char currency like 'X' should NOT match CURRENCY pattern (min 2 chars).
    // This means "10 X" won't parse as amount+currency in a posting.
    let line = "    Assets:Bank  10 X";
    match parse_line(line) {
        Line::Posting {
            number, currency, ..
        } => {
            // If it matches as a posting, currency should be None since X is too short
            assert_eq!(
                number, None,
                "Single char 'X' should not match CURRENCY; expected no amount capture"
            );
            assert_eq!(currency, None);
        }
        other => {
            // Also acceptable: falls through to MetaItem or Other
            match other {
                Line::Other(_) | Line::MetaItem { .. } => {}
                _ => panic!("Unexpected parse: {:?}", other),
            }
        }
    }
}

#[test]
fn meta_vs_posting_ambiguity() {
    // A line like "  filename: value" could match MetaItem.
    // But if it looks like an account (uppercase start + colon), it should match Posting.
    // Since Posting is checked before MetaItem, "  Assets:Bank" should be Posting, not MetaItem.
    let line = "  Assets:Bank";
    assert!(matches!(parse_line(line), Line::Posting { .. }));
}

#[test]
fn meta_with_account_like_key() {
    // "  Assets: something" — starts with uppercase and has colon, but...
    // ACCOUNT requires at least two colon-separated segments with the pattern [\w\p{L}-]+
    // "Assets" alone (no second segment after colon with valid chars) could be tricky.
    // "  category: food" should be MetaItem, not Posting
    let line = "  category: food";
    match parse_line(line) {
        Line::MetaItem { key, value, .. } => {
            assert_eq!(key, "category");
            assert_eq!(value, "food");
        }
        other => panic!("Expected MetaItem, got {:?}", other),
    }
}

#[test]
fn cjk_account_posting() {
    let line = "  Expenses:餐饮:中餐  100.00 CNY";
    match parse_line(line) {
        Line::Posting { account, .. } => {
            assert_eq!(account, "Expenses:餐饮:中餐");
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_tab_indent() {
    let line = "\tExpenses:Food  50.00 USD";
    match parse_line(line) {
        Line::Posting {
            indent, account, ..
        } => {
            assert_eq!(indent, "\t");
            assert_eq!(account, "Expenses:Food");
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn comment_at_start_not_posting() {
    // A line starting with ";" should be Comment, not anything else
    let line = "; Assets:Bank 100 USD";
    assert!(matches!(parse_line(line), Line::Comment { .. }));
}

#[test]
fn transaction_x_flag_uppercase() {
    // X is a valid transaction flag in the regex
    let line = r#"2024-01-15 X "voided""#;
    match parse_line(line) {
        Line::TransactionHeader { flag, .. } => {
            assert_eq!(flag, "X");
        }
        other => panic!("Expected TransactionHeader, got {:?}", other),
    }
}

#[test]
fn transaction_t_flag_uppercase() {
    let line = r#"2024-01-15 T "transfer""#;
    match parse_line(line) {
        Line::TransactionHeader { flag, .. } => {
            assert_eq!(flag, "T");
        }
        other => panic!("Expected TransactionHeader, got {:?}", other),
    }
}

#[test]
fn transaction_tx_flag() {
    let line = r#"2024-01-15 tx "short txn""#;
    match parse_line(line) {
        Line::TransactionHeader { flag, .. } => {
            assert_eq!(flag, "tx");
        }
        other => panic!("Expected TransactionHeader, got {:?}", other),
    }
}

// === More adversarial tests ===

#[test]
fn posting_price_without_cost_has_correct_groups() {
    // Critical: when cost is absent but price is present, the capture group
    // indices must still be correct. Cost is group 5, price is group 6.
    let line = "    Assets:Stock  10 AAPL @ 155 USD";
    match parse_line(line) {
        Line::Posting { cost, price, .. } => {
            assert_eq!(cost, None, "cost should be None when only price present");
            assert_eq!(
                price,
                Some("@ 155 USD"),
                "price capture group must be correct"
            );
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_total_price_has_correct_groups() {
    // @@ is total price - verify the @@? alternation captures correctly
    let line = "    Assets:Stock  10 AAPL @@ 1550 USD";
    match parse_line(line) {
        Line::Posting { cost, price, .. } => {
            assert_eq!(cost, None);
            assert_eq!(price, Some("@@ 1550 USD"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_cost_with_date() {
    // Beancount allows cost with date: {150 USD, 2024-01-01}
    let line = "    Assets:Stock  10 AAPL {150 USD, 2024-01-01}";
    match parse_line(line) {
        Line::Posting { cost, .. } => {
            assert_eq!(cost, Some("{150 USD, 2024-01-01}"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn indented_comment_parsed_before_meta() {
    // "  ; something" should be Comment, not MetaItem
    // Comment check happens AFTER Posting and MetaItem in parse order...
    // Wait - Comment uses `;;?` which matches `;` or `;;`.
    // But MetaItem regex is `(\s+)([\w-]+)\s*:\s*(.*?)\s*$`
    // A line like "  ; comment" has key=";" which contains non-word char, so MetaItem won't match.
    let line = "  ; this is a comment";
    assert!(matches!(parse_line(line), Line::Comment { .. }));
}

#[test]
fn posting_comment_only_no_amount() {
    // Account + comment, no amount
    let line = "    Assets:Bank:Checking ; reconciled";
    match parse_line(line) {
        Line::Posting {
            account,
            number,
            currency,
            cost,
            price,
            comment,
            ..
        } => {
            assert_eq!(account, "Assets:Bank:Checking");
            assert_eq!(number, None);
            assert_eq!(currency, None);
            assert_eq!(cost, None);
            assert_eq!(price, None);
            assert_eq!(comment, Some("; reconciled"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn two_char_currency_is_valid() {
    // CURRENCY pattern: [A-Z][A-Z0-9'._-]{0,22}[A-Z0-9]
    // Minimum 2 chars. "US" should match.
    let line = "    Expenses:Food  50.00 US";
    match parse_line(line) {
        Line::Posting { currency, .. } => {
            assert_eq!(currency, Some("US"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn currency_with_dots_and_dashes() {
    // Beancount allows currencies like "BTC.X" or "USD-2"
    let line = "    Expenses:Crypto  1.5 BTC.X";
    match parse_line(line) {
        Line::Posting { currency, .. } => {
            assert_eq!(currency, Some("BTC.X"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn balance_with_negative_number() {
    let line = "2024-01-01 balance Assets:Bank -500.00 USD";
    match parse_line(line) {
        Line::Balance { number, .. } => {
            assert_eq!(number, "-500.00");
        }
        other => panic!("Expected Balance, got {:?}", other),
    }
}

#[test]
fn open_close_account_with_dashes() {
    let line = "2024-01-01 open Assets:My-Bank:Checking-Account";
    match parse_line(line) {
        Line::Open { account, .. } => {
            assert_eq!(account, "Assets:My-Bank:Checking-Account");
        }
        other => panic!("Expected Open, got {:?}", other),
    }
}

#[test]
fn posting_with_cost_and_comment_no_price() {
    let line = "    Assets:Stock  10 AAPL {150 USD} ; bought";
    match parse_line(line) {
        Line::Posting {
            cost,
            price,
            comment,
            ..
        } => {
            assert_eq!(cost, Some("{150 USD}"));
            assert_eq!(price, None);
            assert_eq!(comment, Some("; bought"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn meta_key_with_dashes_and_underscores() {
    let line = "    my-custom_key: some value";
    match parse_line(line) {
        Line::MetaItem { key, value, .. } => {
            assert_eq!(key, "my-custom_key");
            assert_eq!(value, "some value");
        }
        other => panic!("Expected MetaItem, got {:?}", other),
    }
}

#[test]
fn posting_whole_number_no_decimal() {
    let line = "    Expenses:Food  50 USD";
    match parse_line(line) {
        Line::Posting { number, .. } => {
            assert_eq!(number, Some("50"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

// === Parse order and ambiguity edge cases ===

#[test]
fn indented_line_starting_with_lowercase_is_not_posting() {
    // "  expenses:food  50 USD" — lowercase start should NOT match ACCOUNT
    // because ACCOUNT requires [A-Z\p{Lu}] as first char
    let line = "  expenses:food  50 USD";
    assert!(
        !matches!(parse_line(line), Line::Posting { .. }),
        "Lowercase account should not match Posting"
    );
}

#[test]
fn meta_item_with_uppercase_key_like_account() {
    // "  Filename: test.txt" — starts with uppercase, has colon.
    // ACCOUNT requires `:[\w\p{L}-]+` (second segment after colon).
    // "Filename" has no second colon segment, so it's MetaItem not Posting.
    let line = "  Filename: test.txt";
    match parse_line(line) {
        Line::MetaItem { key, value, .. } => {
            assert_eq!(key, "Filename");
            assert_eq!(value, "test.txt");
        }
        other => panic!("Expected MetaItem, got {:?}", other),
    }
}

#[test]
fn posting_with_three_level_account() {
    let line = "  Income:Salary:Bonus  -1000.00 USD";
    match parse_line(line) {
        Line::Posting {
            account, number, ..
        } => {
            assert_eq!(account, "Income:Salary:Bonus");
            assert_eq!(number, Some("-1000.00"));
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn posting_number_starting_with_dot_does_not_match() {
    // ".50" should not match NUMBER pattern since it requires digits before optional dot
    let line = "  Expenses:Food  .50 USD";
    match parse_line(line) {
        Line::Posting {
            number, currency, ..
        } => {
            // The number regex is -?[\d,]+(?:\.\d+)? which requires at least one digit before dot
            // So ".50 USD" won't match the number+currency group
            assert_eq!(number, None);
            assert_eq!(currency, None);
        }
        other => {
            // Could also fail to match Posting entirely
            eprintln!("Got: {:?}", other);
        }
    }
}

#[test]
fn comment_semicolon_no_space() {
    // ";comment" with no space after semicolon
    let line = ";comment without space";
    match parse_line(line) {
        Line::Comment {
            semicolons,
            content,
            ..
        } => {
            assert_eq!(semicolons, ";");
            // The regex is `(\s*)(;;?)\s?(.*?)\s*$` — the \s? makes space optional
            assert_eq!(content, "comment without space");
        }
        other => panic!("Expected Comment, got {:?}", other),
    }
}

#[test]
fn empty_comment() {
    let line = ";";
    match parse_line(line) {
        Line::Comment {
            semicolons,
            content,
            ..
        } => {
            assert_eq!(semicolons, ";");
            assert_eq!(content, "");
        }
        other => panic!("Expected Comment, got {:?}", other),
    }
}

#[test]
fn posting_account_numbers_in_name() {
    // Account names can contain digits: Assets:Bank123:Acct456
    let line = "  Assets:Bank123:Acct456  100 USD";
    match parse_line(line) {
        Line::Posting { account, .. } => {
            assert_eq!(account, "Assets:Bank123:Acct456");
        }
        other => panic!("Expected Posting, got {:?}", other),
    }
}

#[test]
fn balance_commas_in_number() {
    let line = "2024-01-01 balance Assets:Bank 1,000,000.00 USD";
    match parse_line(line) {
        Line::Balance { number, .. } => {
            assert_eq!(number, "1,000,000.00");
        }
        other => panic!("Expected Balance, got {:?}", other),
    }
}

#[test]
fn transaction_no_payee_narration() {
    // Transaction with flag but no payee/narration
    let line = "2024-01-15 *";
    match parse_line(line) {
        Line::TransactionHeader { date, flag, rest } => {
            assert_eq!(date, "2024-01-15");
            assert_eq!(flag, "*");
            assert_eq!(rest, "");
        }
        other => panic!("Expected TransactionHeader, got {:?}", other),
    }
}
