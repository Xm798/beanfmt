use husk::options::Options;
use husk::recursive::format_recursive;
use std::fs;
use tempfile::TempDir;

fn default_options() -> Options {
    Options::default()
}

#[test]
fn follows_includes() {
    let dir = TempDir::new().unwrap();
    let main_path = dir.path().join("main.beancount");
    let accounts_path = dir.path().join("accounts.beancount");

    fs::write(
        &main_path,
        "include \"accounts.beancount\"\n\n2024-01-01 open Assets:Bank\n",
    )
    .unwrap();
    fs::write(
        &accounts_path,
        "2024-01-01 open Expenses:Food\n",
    )
    .unwrap();

    let results = format_recursive(&main_path, &default_options());

    assert_eq!(results.len(), 2, "should format both files");

    let paths: Vec<_> = results.iter().map(|f| f.path.clone()).collect();
    assert_eq!(paths[0], fs::canonicalize(&main_path).unwrap());
    assert_eq!(paths[1], fs::canonicalize(&accounts_path).unwrap());
}

#[test]
fn no_cycles() {
    let dir = TempDir::new().unwrap();
    let a_path = dir.path().join("a.beancount");
    let b_path = dir.path().join("b.beancount");

    fs::write(&a_path, "include \"b.beancount\"\n").unwrap();
    fs::write(&b_path, "include \"a.beancount\"\n").unwrap();

    let results = format_recursive(&a_path, &default_options());

    assert_eq!(results.len(), 2, "each file visited exactly once");

    let paths: Vec<_> = results.iter().map(|f| f.path.clone()).collect();
    let canonical_a = fs::canonicalize(&a_path).unwrap();
    let canonical_b = fs::canonicalize(&b_path).unwrap();
    assert!(paths.contains(&canonical_a));
    assert!(paths.contains(&canonical_b));
}

#[test]
fn missing_include_skipped() {
    let dir = TempDir::new().unwrap();
    let main_path = dir.path().join("main.beancount");

    fs::write(
        &main_path,
        "include \"nonexistent.beancount\"\n\n2024-01-01 open Assets:Bank\n",
    )
    .unwrap();

    let results = format_recursive(&main_path, &default_options());

    assert_eq!(results.len(), 1, "only the existing file is formatted");
    assert_eq!(results[0].path, fs::canonicalize(&main_path).unwrap());
}

#[test]
fn follows_glob_includes() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("accounts");
    fs::create_dir(&sub).unwrap();
    let main_path = dir.path().join("main.beancount");

    fs::write(
        &main_path,
        "include \"accounts/*.beancount\"\n\n2024-01-01 open Assets:Bank\n",
    )
    .unwrap();
    fs::write(
        sub.join("a.beancount"),
        "2024-01-01 open Assets:A\n",
    )
    .unwrap();
    fs::write(
        sub.join("b.beancount"),
        "2024-01-01 open Assets:B\n",
    )
    .unwrap();

    let results = format_recursive(&main_path, &default_options());

    assert_eq!(results.len(), 3, "should format main + 2 glob matches");
}
