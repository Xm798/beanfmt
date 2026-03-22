use husk::line::{parse_line, BlockKind, Line};

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
