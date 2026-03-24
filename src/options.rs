use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum SortableDirective {
    Transaction,
    Balance,
    Open,
    Close,
    Price,
    Pad,
    Note,
    Document,
    Event,
    Custom,
    Query,
    Commodity,
}

impl FromStr for SortableDirective {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "transaction" | "txn" => Ok(SortableDirective::Transaction),
            "balance" => Ok(SortableDirective::Balance),
            "open" => Ok(SortableDirective::Open),
            "close" => Ok(SortableDirective::Close),
            "price" => Ok(SortableDirective::Price),
            "pad" => Ok(SortableDirective::Pad),
            "note" => Ok(SortableDirective::Note),
            "document" => Ok(SortableDirective::Document),
            "event" => Ok(SortableDirective::Event),
            "custom" => Ok(SortableDirective::Custom),
            "query" => Ok(SortableDirective::Query),
            "commodity" => Ok(SortableDirective::Commodity),
            other => Err(format!(
                "invalid directive: {other:?}, expected one of: transaction, balance, open, close, price, pad, note, document, event, custom, query, commodity"
            )),
        }
    }
}

impl std::fmt::Display for SortableDirective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortableDirective::Transaction => write!(f, "transaction"),
            SortableDirective::Balance => write!(f, "balance"),
            SortableDirective::Open => write!(f, "open"),
            SortableDirective::Close => write!(f, "close"),
            SortableDirective::Price => write!(f, "price"),
            SortableDirective::Pad => write!(f, "pad"),
            SortableDirective::Note => write!(f, "note"),
            SortableDirective::Document => write!(f, "document"),
            SortableDirective::Event => write!(f, "event"),
            SortableDirective::Custom => write!(f, "custom"),
            SortableDirective::Query => write!(f, "query"),
            SortableDirective::Commodity => write!(f, "commodity"),
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum TimelessPosition {
    Begin,
    End,
}

impl FromStr for TimelessPosition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "begin" => Ok(TimelessPosition::Begin),
            "end" => Ok(TimelessPosition::End),
            other => Err(format!(
                "invalid sort_timeless: {other:?}, expected \"begin\" or \"end\""
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
    pub sort_timeless: TimelessPosition,
    pub sort_exclude: Vec<SortableDirective>,
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
            sort_timeless: TimelessPosition::Begin,
            sort_exclude: Vec::new(),
        }
    }
}
