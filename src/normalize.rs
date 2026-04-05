use crate::options::ThousandsSeparator;
use regex::Regex;
use std::sync::LazyLock;

// \s* consumes all whitespace so normalize_comment can re-emit exactly one space
static COMMENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*)(;;?)\s*(.*?)\s*$").unwrap());

/// Replace leading whitespace with the configured indent string.
/// Indent depth is determined by dividing leading space count by base width (default 4),
/// rounding up, with a minimum of 1.
pub fn normalize_indent(line: &str, indent: usize) -> String {
    if line.is_empty() || !line.starts_with(|c: char| c.is_whitespace()) {
        return line.to_string();
    }
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return line.to_string();
    }

    let leading_len = line.len() - trimmed.len();
    let leading = &line[..leading_len];

    let indent_str = " ".repeat(indent);
    let base_width = if indent > 0 { indent } else { 4 };
    let effective: usize = leading
        .chars()
        .map(|c| if c == '\t' { base_width } else { 1 })
        .sum();

    let depth = effective.div_ceil(base_width).max(1);
    format!("{}{}", indent_str.repeat(depth), trimmed)
}

/// Normalize comment spacing: exactly one space after semicolons, no trailing space if empty.
pub fn normalize_comment(line: &str) -> String {
    if let Some(caps) = COMMENT_RE.captures(line) {
        let ws = &caps[1];
        let semis = &caps[2];
        let content = &caps[3];
        if content.is_empty() {
            format!("{}{}", ws, semis)
        } else {
            format!("{}{} {}", ws, semis, content)
        }
    } else {
        line.to_string()
    }
}

/// Normalize thousands separators on an extracted number string.
pub fn normalize_thousands(num_str: &str, separator: &ThousandsSeparator) -> String {
    // Strip any space between sign and digits (e.g., "- 619.47" → "-619.47")
    let num_str = &num_str.replace(' ', "");
    match separator {
        ThousandsSeparator::Keep => num_str.to_string(),
        ThousandsSeparator::Remove => num_str.replace(',', ""),
        ThousandsSeparator::Add => {
            let stripped = num_str.replace(',', "");

            // Handle sign prefix
            let (sign, rest) = if let Some(r) = stripped.strip_prefix('-') {
                ("-", r)
            } else if let Some(r) = stripped.strip_prefix('+') {
                ("+", r)
            } else {
                ("", stripped.as_str())
            };

            // Split integer and decimal parts
            let (int_part, dec_part) = match rest.find('.') {
                Some(pos) => (&rest[..pos], Some(&rest[pos..])),
                None => (rest, None),
            };

            // Insert commas every 3 digits from the right
            let int_with_commas = add_commas(int_part);

            match dec_part {
                Some(dec) => format!("{}{}{}", sign, int_with_commas, dec),
                None => format!("{}{}", sign, int_with_commas),
            }
        }
    }
}

fn add_commas(int_part: &str) -> String {
    let bytes = int_part.as_bytes();
    let len = bytes.len();
    if len <= 3 {
        return int_part.to_string();
    }

    let mut result = String::with_capacity(len + len / 3);
    let first_group = len % 3;
    if first_group > 0 {
        result.push_str(&int_part[..first_group]);
    }
    for i in (first_group..len).step_by(3) {
        if !result.is_empty() {
            result.push(',');
        }
        result.push_str(&int_part[i..i + 3]);
    }
    result
}

/// Normalize brace spacing: add spaces inside `{...}` and `{{...}}` if enabled.
pub fn normalize_braces(s: &str, spaces_in_braces: bool) -> String {
    if !spaces_in_braces {
        return s.to_string();
    }

    static DOUBLE_BRACE_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\{\{\s*(.*?)\s*\}\}").unwrap());
    static SINGLE_BRACE_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\{\s*(.*?)\s*\}").unwrap());

    // Replace double braces with placeholders to avoid single-brace regex matching them
    const OPEN_PH: &str = "\x00DOPEN\x00";
    const CLOSE_PH: &str = "\x00DCLOSE\x00";

    let result = DOUBLE_BRACE_RE.replace_all(s, |caps: &regex::Captures| {
        let content = &caps[1];
        if content.is_empty() {
            format!("{}{}", OPEN_PH, CLOSE_PH)
        } else {
            format!("{} {} {}", OPEN_PH, content, CLOSE_PH)
        }
    });

    // Process single braces
    let result = SINGLE_BRACE_RE.replace_all(&result, |caps: &regex::Captures| {
        let content = &caps[1];
        if content.is_empty() {
            "{}".to_string()
        } else {
            format!("{{ {} }}", content)
        }
    });

    // Restore double braces
    result.replace(OPEN_PH, "{{").replace(CLOSE_PH, "}}")
}
