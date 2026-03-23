#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum ThousandsSeparator {
    Add,
    Remove,
    Keep,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub indent: usize,
    pub currency_column: usize,
    pub cost_column: usize,
    pub thousands_separator: ThousandsSeparator,
    pub spaces_in_braces: bool,
    pub fixed_cjk_width: bool,
    pub sort: bool,
}

impl Options {
    pub fn indent_str(&self) -> String {
        " ".repeat(self.indent)
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            indent: 4,
            currency_column: 70,
            cost_column: 75,
            thousands_separator: ThousandsSeparator::Keep,
            spaces_in_braces: false,
            fixed_cjk_width: true,
            sort: false,
        }
    }
}
