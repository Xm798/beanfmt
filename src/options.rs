#[derive(Debug, Clone)]
pub enum ThousandsSeparator {
    Add,
    Remove,
    Keep,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub indent: String,
    pub currency_column: usize,
    pub cost_column: usize,
    pub thousands_separator: ThousandsSeparator,
    pub spaces_in_braces: bool,
    pub fixed_cjk_width: bool,
    pub sort: bool,
    pub recursive: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            indent: "    ".to_string(),
            currency_column: 70,
            cost_column: 75,
            thousands_separator: ThousandsSeparator::Keep,
            spaces_in_braces: false,
            fixed_cjk_width: true,
            sort: false,
            recursive: false,
        }
    }
}
