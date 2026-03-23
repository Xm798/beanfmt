use crate::line::{parse_line, Line};
use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
struct Entry {
    lines: Vec<String>,
    date: Option<String>,
    time: Option<String>,
}

static TIME_HM_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{2}):(\d{2})$").unwrap());

static TIME_HMS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{2}):(\d{2}):(\d{2})$").unwrap());

static UNIX_TS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{10,16})$").unwrap());

pub fn parse_time(value: &str) -> Option<String> {
    let value = value.trim().trim_matches('"');
    if let Some(caps) = TIME_HMS_RE.captures(value) {
        return Some(format!("{}:{}:{}", &caps[1], &caps[2], &caps[3]));
    }
    if let Some(caps) = TIME_HM_RE.captures(value) {
        return Some(format!("{}:{}:00", &caps[1], &caps[2]));
    }
    if let Some(caps) = UNIX_TS_RE.captures(value) {
        let digits: u64 = caps[1].parse().ok()?;
        // Check micros before millis
        let secs = if digits >= 1_000_000_000_000_000 {
            digits / 1_000_000
        } else if digits >= 1_000_000_000_000 {
            digits / 1_000
        } else {
            digits
        };
        let time_of_day = secs % 86400;
        let h = time_of_day / 3600;
        let m = (time_of_day % 3600) / 60;
        let s = time_of_day % 60;
        return Some(format!("{h:02}:{m:02}:{s:02}"));
    }
    None
}

fn is_continuation(line: &Line) -> bool {
    match line {
        Line::Posting { .. } | Line::MetaItem { .. } => true,
        Line::Comment { indent, .. } => !indent.is_empty(),
        _ => false,
    }
}

fn extract_date(line: &Line) -> Option<String> {
    match line {
        Line::TransactionHeader { date, .. }
        | Line::Balance { date, .. }
        | Line::Open { date, .. }
        | Line::Close { date, .. }
        | Line::Price { date, .. }
        | Line::DateDirective { date, .. } => Some(date.to_string()),
        _ => None,
    }
}

/// A segment is either a barrier, a blank line, or a sortable group of entries.
#[derive(Debug)]
enum Segment {
    Barrier(String),
    Blank,
    Entries(Vec<Entry>),
}

/// Parse input into entries, grouping them into segments.
fn parse_segments(input: &str) -> Vec<Segment> {
    let raw_lines: Vec<&str> = input.lines().collect();
    let mut segments: Vec<Segment> = Vec::new();
    let mut current_entries: Vec<Entry> = Vec::new();
    let mut current_entry: Option<Entry> = None;

    let flush_entry = |current_entry: &mut Option<Entry>, current_entries: &mut Vec<Entry>| {
        if let Some(entry) = current_entry.take() {
            current_entries.push(entry);
        }
    };

    let flush_entries = |current_entries: &mut Vec<Entry>, segments: &mut Vec<Segment>| {
        if !current_entries.is_empty() {
            segments.push(Segment::Entries(std::mem::take(current_entries)));
        }
    };

    for raw in &raw_lines {
        let parsed = parse_line(raw);

        if matches!(parsed, Line::BlankLine) {
            flush_entry(&mut current_entry, &mut current_entries);
            flush_entries(&mut current_entries, &mut segments);
            segments.push(Segment::Blank);
            continue;
        }

        if matches!(parsed, Line::BlockDirective { .. }) {
            flush_entry(&mut current_entry, &mut current_entries);
            flush_entries(&mut current_entries, &mut segments);
            segments.push(Segment::Barrier(raw.to_string()));
            continue;
        }

        if is_continuation(&parsed) {
            if let Some(ref mut entry) = current_entry {
                if let Line::MetaItem { key, value, .. } = &parsed && *key == "time" {
                    entry.time = parse_time(value);
                }
                entry.lines.push(raw.to_string());
            } else {
                current_entry = Some(Entry {
                    lines: vec![raw.to_string()],
                    date: None,
                    time: None,
                });
            }
        } else {
            // New directive or top-level line — starts a new entry
            flush_entry(&mut current_entry, &mut current_entries);
            let date = extract_date(&parsed);
            current_entry = Some(Entry {
                lines: vec![raw.to_string()],
                date,
                time: None,
            });
        }
    }

    flush_entry(&mut current_entry, &mut current_entries);
    flush_entries(&mut current_entries, &mut segments);

    segments
}

fn sort_key(entry: &Entry) -> (&str, &str) {
    (
        entry.date.as_deref().unwrap_or(""),
        entry.time.as_deref().unwrap_or(""),
    )
}

fn can_go_before(a: &Entry, b: &Entry) -> bool {
    match (&a.date, &b.date) {
        (Some(da), Some(db)) => {
            if da < db {
                return true;
            }
            if da > db {
                return false;
            }
            // Same date
            match (&a.time, &b.time) {
                (Some(ta), Some(tb)) => ta <= tb,
                _ => true,
            }
        }
        _ => true,
    }
}

fn split_sorted_unsorted(entries: &[Entry]) -> (Vec<usize>, Vec<usize>) {
    if entries.is_empty() {
        return (vec![], vec![]);
    }

    // Patience sort for longest non-decreasing subsequence
    let mut tails: Vec<usize> = Vec::new();
    let mut predecessors: Vec<Option<usize>> = vec![None; entries.len()];

    for i in 0..entries.len() {
        let pos = tails.partition_point(|&t| can_go_before(&entries[t], &entries[i]));

        if pos == tails.len() {
            tails.push(i);
        } else {
            tails[pos] = i;
        }

        if pos > 0 {
            predecessors[i] = Some(tails[pos - 1]);
        }
    }

    // Backtrack to recover LIS
    let mut lis_indices: Vec<usize> = Vec::new();
    let mut idx = *tails.last().unwrap();
    lis_indices.push(idx);
    while let Some(prev) = predecessors[idx] {
        lis_indices.push(prev);
        idx = prev;
    }
    lis_indices.reverse();

    let lis_set: std::collections::HashSet<usize> = lis_indices.iter().copied().collect();
    let unsorted: Vec<usize> = (0..entries.len())
        .filter(|i| !lis_set.contains(i))
        .collect();

    (lis_indices, unsorted)
}

fn merge_entries(entries: &[Entry], sorted_idx: &[usize], unsorted_idx: &[usize]) -> Vec<usize> {
    if unsorted_idx.is_empty() {
        return sorted_idx.to_vec();
    }

    let mut to_insert: Vec<usize> = unsorted_idx.to_vec();
    to_insert.sort_by(|&a, &b| sort_key(&entries[a]).cmp(&sort_key(&entries[b])));

    let mut result = Vec::with_capacity(entries.len());
    let mut si = 0;
    let mut ui = 0;

    while si < sorted_idx.len() && ui < to_insert.len() {
        let s = sorted_idx[si];
        let u = to_insert[ui];
        if can_go_before(&entries[u], &entries[s]) {
            result.push(u);
            ui += 1;
        } else {
            result.push(s);
            si += 1;
        }
    }
    while si < sorted_idx.len() {
        result.push(sorted_idx[si]);
        si += 1;
    }
    while ui < to_insert.len() {
        result.push(to_insert[ui]);
        ui += 1;
    }

    result
}

fn sort_entries(mut entries: Vec<Entry>) -> Vec<Entry> {
    if entries.len() <= 1 {
        return entries;
    }

    let (sorted_idx, unsorted_idx) = split_sorted_unsorted(&entries);
    if unsorted_idx.is_empty() {
        return entries;
    }

    let order = merge_entries(&entries, &sorted_idx, &unsorted_idx);

    // Reorder in-place using the index mapping
    let mut result = Vec::with_capacity(entries.len());
    // We need to move out of entries by index; use Option wrapper
    let mut slots: Vec<Option<Entry>> = entries.drain(..).map(Some).collect();
    for i in order {
        result.push(slots[i].take().unwrap());
    }
    result
}

pub fn sort_input(input: &str) -> String {
    let segments = parse_segments(input);

    let mut output = String::new();

    #[derive(Debug)]
    enum CompartmentItem {
        Entries(Vec<Entry>),
        Blank,
    }

    let mut compartment_items: Vec<CompartmentItem> = Vec::new();

    let flush_compartment = |items: &mut Vec<CompartmentItem>, output: &mut String| {
        // Collect all entries from the compartment
        let mut all_entries: Vec<Entry> = Vec::new();
        let mut separator_positions: Vec<usize> = Vec::new(); // after which entry index to insert blank

        for item in items.drain(..) {
            match item {
                CompartmentItem::Entries(entries) => {
                    all_entries.extend(entries);
                }
                CompartmentItem::Blank => {
                    if !all_entries.is_empty() {
                        separator_positions.push(all_entries.len());
                    } else {
                        // Leading blank
                        output.push('\n');
                    }
                }
            }
        }

        if all_entries.is_empty() {
            return;
        }

        let sorted = sort_entries(all_entries);

        // Emit entries with blank line separators between them
        for (i, entry) in sorted.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            for line in &entry.lines {
                output.push_str(line);
                output.push('\n');
            }
        }

        // If there were trailing blanks beyond what we used as separators, they were already
        // consumed. The number of blanks between N entries = N-1 separators. Any extra blanks
        // from the original are trailing blanks for the compartment.
        let used_separators = if sorted.len() > 1 { sorted.len() - 1 } else { 0 };
        let total_blanks = separator_positions.len();
        for _ in used_separators..total_blanks {
            output.push('\n');
        }
    };

    for segment in segments {
        match segment {
            Segment::Barrier(line) => {
                flush_compartment(&mut compartment_items, &mut output);
                output.push_str(&line);
                output.push('\n');
            }
            Segment::Blank => {
                compartment_items.push(CompartmentItem::Blank);
            }
            Segment::Entries(entries) => {
                compartment_items.push(CompartmentItem::Entries(entries));
            }
        }
    }
    flush_compartment(&mut compartment_items, &mut output);

    // Preserve trailing newline behavior
    if !input.ends_with('\n') && output.ends_with('\n') {
        output.pop();
    }

    output
}
