use husk::format;
use husk::options::Options;

#[test]
fn test_format_with_sort() {
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
fn test_format_without_sort_preserves_order() {
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
