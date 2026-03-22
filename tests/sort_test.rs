use husk::sort::sort_input;

#[test]
fn already_sorted_unchanged() {
    let input = "\
2024-01-01 * \"First\"
  Assets:Bank  100 USD

2024-01-02 * \"Second\"
  Assets:Bank  200 USD

2024-01-03 * \"Third\"
  Assets:Bank  300 USD
";
    assert_eq!(sort_input(input), input);
}

#[test]
fn out_of_order_sorted_by_date() {
    let input = "\
2024-01-03 * \"Third\"
  Assets:Bank  300 USD

2024-01-01 * \"First\"
  Assets:Bank  100 USD

2024-01-02 * \"Second\"
  Assets:Bank  200 USD
";
    let expected = "\
2024-01-01 * \"First\"
  Assets:Bank  100 USD

2024-01-02 * \"Second\"
  Assets:Bank  200 USD

2024-01-03 * \"Third\"
  Assets:Bank  300 USD
";
    assert_eq!(sort_input(input), expected);
}

#[test]
fn barriers_respected() {
    let input = "\
2024-01-03 * \"Late\"
  Assets:Bank  300 USD

2024-01-01 * \"Early\"
  Assets:Bank  100 USD

pushtag #trip

2024-01-05 * \"After barrier late\"
  Assets:Bank  500 USD

2024-01-04 * \"After barrier early\"
  Assets:Bank  400 USD

poptag #trip
";
    let expected = "\
2024-01-01 * \"Early\"
  Assets:Bank  100 USD

2024-01-03 * \"Late\"
  Assets:Bank  300 USD

pushtag #trip

2024-01-04 * \"After barrier early\"
  Assets:Bank  400 USD

2024-01-05 * \"After barrier late\"
  Assets:Bank  500 USD

poptag #trip
";
    assert_eq!(sort_input(input), expected);
}

#[test]
fn time_metadata_sorting() {
    let input = "\
2024-01-01 * \"Later\"
  time: \"15:30\"
  Assets:Bank  200 USD

2024-01-01 * \"Earlier\"
  time: \"09:00\"
  Assets:Bank  100 USD
";
    let expected = "\
2024-01-01 * \"Earlier\"
  time: \"09:00\"
  Assets:Bank  100 USD

2024-01-01 * \"Later\"
  time: \"15:30\"
  Assets:Bank  200 USD
";
    assert_eq!(sort_input(input), expected);
}

#[test]
fn prudent_sort_minimal_moves() {
    // 1,2,5,3,4 — the LIS is 1,2,3,4 (or 1,2,5) — with prudent sort,
    // 5 is the only out-of-place entry that moves, result: 1,2,3,4,5
    let input = "\
2024-01-01 * \"One\"

2024-01-02 * \"Two\"

2024-01-05 * \"Five\"

2024-01-03 * \"Three\"

2024-01-04 * \"Four\"
";
    let expected = "\
2024-01-01 * \"One\"

2024-01-02 * \"Two\"

2024-01-03 * \"Three\"

2024-01-04 * \"Four\"

2024-01-05 * \"Five\"
";
    assert_eq!(sort_input(input), expected);
}

#[test]
fn entries_without_date_stay_in_place() {
    let input = "\
2024-01-02 * \"Second\"
  Assets:Bank  200 USD

; A comment with no date

2024-01-01 * \"First\"
  Assets:Bank  100 USD
";
    // The comment has no date so it forms its own block (separated by blank lines
    // from the transactions). Only within each block are entries sorted.
    // Here block1=[Second], blank, block2=[comment], blank, block3=[First]
    // Each block has only 1 entry, so no reordering happens.
    // But actually the blank lines separate them into single-entry blocks.
    // Let me verify the actual behavior: the comment is in its own block.
    let result = sort_input(input);
    // The undated comment should remain between the two transactions
    assert!(result.contains("; A comment with no date"));
    // Each block has only one entry, so relative order is preserved
    assert_eq!(result, input);
}

#[test]
fn same_date_preserves_relative_order() {
    let input = "\
2024-01-01 * \"Alpha\"
  Assets:Bank  100 USD

2024-01-01 * \"Beta\"
  Assets:Bank  200 USD

2024-01-01 * \"Gamma\"
  Assets:Bank  300 USD
";
    // All same date, no time — should preserve original order
    assert_eq!(sort_input(input), input);
}
