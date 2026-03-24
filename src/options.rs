use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThousandsSeparator {
    Add,
    Remove,
    Keep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum SortOrder {
    Off,
    Asc,
    Desc,
}

impl FromStr for SortOrder {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "asc" | "true" => Ok(SortOrder::Asc),
            "desc" => Ok(SortOrder::Desc),
            "off" | "false" => Ok(SortOrder::Off),
            other => Err(format!(
                "invalid sort: {other:?}, expected \"asc\", \"desc\", or \"off\""
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub indent: usize,
    pub currency_column: usize,
    pub cost_column: usize,
    pub thousands_separator: ThousandsSeparator,
    pub spaces_in_braces: bool,
    pub fixed_cjk_width: bool,
    pub sort: SortOrder,
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
            sort: SortOrder::Off,
        }
    }
}
