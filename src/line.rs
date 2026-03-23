use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    PushTag,
    PopTag,
    PushMeta,
    PopMeta,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Line<'a> {
    TransactionHeader {
        date: &'a str,
        flag: &'a str,
        rest: &'a str,
    },
    Posting {
        indent: &'a str,
        account: &'a str,
        number: Option<&'a str>,
        currency: Option<&'a str>,
        cost: Option<&'a str>,
        price: Option<&'a str>,
        comment: Option<&'a str>,
    },
    Balance {
        date: &'a str,
        account: &'a str,
        number: &'a str,
        currency: &'a str,
    },
    Open {
        date: &'a str,
        account: &'a str,
        currencies: &'a str,
    },
    Close {
        date: &'a str,
        account: &'a str,
    },
    Price {
        date: &'a str,
        commodity: &'a str,
        number: &'a str,
        currency: &'a str,
    },
    MetaItem {
        indent: &'a str,
        key: &'a str,
        value: &'a str,
    },
    Comment {
        indent: &'a str,
        semicolons: &'a str,
        content: &'a str,
    },
    BlockDirective {
        kind: BlockKind,
        rest: &'a str,
    },
    DateDirective {
        date: &'a str,
        keyword: &'a str,
        rest: &'a str,
    },
    Include {
        path: &'a str,
    },
    BlankLine,
    Other(&'a str),
}

const DATE: &str = r"\d{4}-\d{2}-\d{2}";
const ACCOUNT: &str = r"[A-Z\p{Lu}][\w-]*(?::[\w\p{L}-]+)+";
const NUMBER: &str = r"-?\d[\d,]*(?:\.\d+)?";
const CURRENCY: &str = r"[A-Z][A-Z0-9'._-]{0,22}[A-Z0-9]";

static TRANSACTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"^({DATE})\s+(txn|tx|\*|!|T|X)\s*(.*?)\s*$")).unwrap());

static POSTING_RE: LazyLock<Regex> = LazyLock::new(|| {
    let cost = r"(\{\{.*?\}\}|\{.*?\})";
    let pattern = format!(
        r"^(\s+)({ACCOUNT})(?:\s+({NUMBER})\s+({CURRENCY}))?(?:\s+{cost})?(?:\s+(@@?\s+{NUMBER}\s+{CURRENCY}))?(?:\s+(;.*))?$"
    );
    Regex::new(&pattern).unwrap()
});

static BALANCE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^({DATE})\s+balance\s+({ACCOUNT})\s+({NUMBER})\s+({CURRENCY})\s*$"
    ))
    .unwrap()
});

static OPEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^({DATE})\s+open\s+({ACCOUNT})(?:\s+(.*\S))?\s*$"
    ))
    .unwrap()
});

static CLOSE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"^({DATE})\s+close\s+({ACCOUNT})\s*$")).unwrap());

static PRICE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^({DATE})\s+price\s+({CURRENCY})\s+({NUMBER})\s+({CURRENCY})\s*$"
    ))
    .unwrap()
});

static META_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s+)([\w-]+)\s*:\s*(.*?)\s*$").unwrap());

// \s? consumes at most one space — preserves original spacing in content field for parsing
static COMMENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*)(;;?)\s?(.*?)\s*$").unwrap());

static BLOCK_DIRECTIVE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(pushtag|poptag|pushmeta|popmeta)\s+(.*?)\s*$").unwrap());

static DATE_DIRECTIVE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^({DATE})\s+(note|document|pad|event|custom|query|commodity)\s+(.*?)\s*$"
    ))
    .unwrap()
});

static INCLUDE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^include\s+"(.*?)"\s*$"#).unwrap());

pub fn parse_line(line: &str) -> Line<'_> {
    if line.trim().is_empty() {
        return Line::BlankLine;
    }

    if let Some(caps) = TRANSACTION_RE.captures(line) {
        return Line::TransactionHeader {
            date: caps.get(1).unwrap().as_str(),
            flag: caps.get(2).unwrap().as_str(),
            rest: caps.get(3).unwrap().as_str(),
        };
    }

    if let Some(caps) = BALANCE_RE.captures(line) {
        return Line::Balance {
            date: caps.get(1).unwrap().as_str(),
            account: caps.get(2).unwrap().as_str(),
            number: caps.get(3).unwrap().as_str(),
            currency: caps.get(4).unwrap().as_str(),
        };
    }

    if let Some(caps) = OPEN_RE.captures(line) {
        return Line::Open {
            date: caps.get(1).unwrap().as_str(),
            account: caps.get(2).unwrap().as_str(),
            currencies: caps.get(3).map(|m| m.as_str()).unwrap_or(""),
        };
    }

    if let Some(caps) = CLOSE_RE.captures(line) {
        return Line::Close {
            date: caps.get(1).unwrap().as_str(),
            account: caps.get(2).unwrap().as_str(),
        };
    }

    if let Some(caps) = PRICE_RE.captures(line) {
        return Line::Price {
            date: caps.get(1).unwrap().as_str(),
            commodity: caps.get(2).unwrap().as_str(),
            number: caps.get(3).unwrap().as_str(),
            currency: caps.get(4).unwrap().as_str(),
        };
    }

    if let Some(caps) = POSTING_RE.captures(line) {
        return Line::Posting {
            indent: caps.get(1).unwrap().as_str(),
            account: caps.get(2).unwrap().as_str(),
            number: caps.get(3).map(|m| m.as_str()),
            currency: caps.get(4).map(|m| m.as_str()),
            cost: caps.get(5).map(|m| m.as_str()),
            price: caps.get(6).map(|m| m.as_str()),
            comment: caps.get(7).map(|m| m.as_str()),
        };
    }

    if let Some(caps) = META_RE.captures(line) {
        return Line::MetaItem {
            indent: caps.get(1).unwrap().as_str(),
            key: caps.get(2).unwrap().as_str(),
            value: caps.get(3).unwrap().as_str(),
        };
    }

    if let Some(caps) = COMMENT_RE.captures(line) {
        return Line::Comment {
            indent: caps.get(1).unwrap().as_str(),
            semicolons: caps.get(2).unwrap().as_str(),
            content: caps.get(3).unwrap().as_str(),
        };
    }

    if let Some(caps) = BLOCK_DIRECTIVE_RE.captures(line) {
        let kind = match caps.get(1).unwrap().as_str() {
            "pushtag" => BlockKind::PushTag,
            "poptag" => BlockKind::PopTag,
            "pushmeta" => BlockKind::PushMeta,
            "popmeta" => BlockKind::PopMeta,
            _ => unreachable!(),
        };
        return Line::BlockDirective {
            kind,
            rest: caps.get(2).unwrap().as_str(),
        };
    }

    if let Some(caps) = DATE_DIRECTIVE_RE.captures(line) {
        return Line::DateDirective {
            date: caps.get(1).unwrap().as_str(),
            keyword: caps.get(2).unwrap().as_str(),
            rest: caps.get(3).unwrap().as_str(),
        };
    }

    if let Some(caps) = INCLUDE_RE.captures(line) {
        return Line::Include {
            path: caps.get(1).unwrap().as_str(),
        };
    }

    Line::Other(line)
}
