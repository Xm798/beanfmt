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
    Keep,
}

impl FromStr for TimelessPosition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "begin" => Ok(TimelessPosition::Begin),
            "end" => Ok(TimelessPosition::End),
            "keep" => Ok(TimelessPosition::Keep),
            other => Err(format!(
                "invalid sort_timeless: {other:?}, expected \"begin\", \"end\", or \"keep\""
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum DecimalMode {
    Keep,
    Minimal,
    Pad,
}

impl FromStr for DecimalMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "keep" => Ok(DecimalMode::Keep),
            "minimal" => Ok(DecimalMode::Minimal),
            "pad" => Ok(DecimalMode::Pad),
            other => Err(format!(
                "invalid decimal_mode: {other:?}, expected \"keep\", \"minimal\", or \"pad\""
            )),
        }
    }
}

impl std::fmt::Display for DecimalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecimalMode::Keep => write!(f, "keep"),
            DecimalMode::Minimal => write!(f, "minimal"),
            DecimalMode::Pad => write!(f, "pad"),
        }
    }
}

/// Which numbers thousands-separator and decimal normalization reach: posting and
/// `balance` amounts plus costs, price annotations and `price` directives
/// ([`AmountScope::All`]), or posting and `balance` amounts only ([`AmountScope::Amounts`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum AmountScope {
    All,
    Amounts,
}

impl FromStr for AmountScope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "all" => Ok(AmountScope::All),
            "amounts" => Ok(AmountScope::Amounts),
            other => Err(format!(
                "invalid amount_scope: {other:?}, expected \"all\" or \"amounts\""
            )),
        }
    }
}

impl std::fmt::Display for AmountScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AmountScope::All => write!(f, "all"),
            AmountScope::Amounts => write!(f, "amounts"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub indent: usize,
    pub currency_column: usize,
    pub cost_column: usize,
    /// Column to align inline comments (`;`) to. 0 disables alignment.
    /// Applies to postings, `balance`, `open`, `close`, and `price`. Transaction
    /// headers and date directives keep their comment inline (it lives in their
    /// free-form payload, which may contain a quoted `;`).
    pub inline_comment_column: usize,
    pub thousands_separator: ThousandsSeparator,
    pub decimal_mode: DecimalMode,
    /// Fraction width used by [`DecimalMode::Pad`].
    pub decimal_places: usize,
    /// Which numbers thousands-separator and decimal normalization reach.
    pub amount_scope: AmountScope,
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
            indent: 2,
            currency_column: 70,
            cost_column: 75,
            inline_comment_column: 0,
            thousands_separator: ThousandsSeparator::Keep,
            decimal_mode: DecimalMode::Keep,
            decimal_places: 2,
            amount_scope: AmountScope::All,
            spaces_in_braces: false,
            fixed_cjk_width: true,
            sort: SortOrder::Off,
            sort_timeless: TimelessPosition::Keep,
            sort_exclude: Vec::new(),
        }
    }
}
