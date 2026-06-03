//! Minimally-ambiguous name abbreviation.
//!
//! Given a list of segmented names (e.g. path components), produce the shortest
//! unambiguous label for each. Shared segments are collapsed to ellipsis.
//!
//! Side preference:
//! - `Right`: anchor from the right (files: show filename, disambiguate leftward)
//! - `Left`: anchor from the left (domains: show TLD, disambiguate rightward)

#[cfg(test)]
mod tests;

/// Which end of the segmented name is the "identity" anchor.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// Rightmost segment is primary (e.g. filenames: `mod.rs`).
    Right,
    /// Leftmost segment is primary (e.g. domain names).
    Left,
}

/// Produce minimally-ambiguous labels for a list of names.
///
/// Each name is split by `delimiter`. The algorithm finds the shortest suffix
/// (or prefix, depending on `side`) that uniquely identifies each name among
/// the set, collapsing shared intermediate segments with the unicode ellipsis `…`.
///
/// Returns labels 1:1 with input. Names that are already unique get just their
/// anchor segment (filename or first component).
pub fn disambiguate(names: &[&str], delimiter: char, side: Side) -> Vec<String> {
    disambiguate_with(names, delimiter, side, "…")
}

/// Like [`disambiguate`], but lets the caller choose the collapse marker
/// (e.g. `"…"`, `"**"`, `"..."`).
pub fn disambiguate_with(names: &[&str], delimiter: char, side: Side, ellipsis: &str) -> Vec<String> {
    if names.is_empty() {
        return Vec::new();
    }
    let split: Vec<Vec<&str>> = names.iter().map(|n| n.split(delimiter).collect::<Vec<_>>()).collect();

    let anchors: Vec<&str> = split
        .iter()
        .map(|parts| match side {
            Side::Right => parts.last().copied().unwrap_or(""),
            Side::Left => parts.first().copied().unwrap_or(""),
        })
        .collect();

    // If all anchors are unique, just return them
    if all_unique(&anchors) {
        return anchors.iter().map(|s| (*s).to_string()).collect();
    }

    // For each name, find minimal distinguishing segments
    let mut results: Vec<String> = Vec::with_capacity(names.len());
    for (i, parts) in split.iter().enumerate() {
        let label = disambiguate_one(i, parts, &split, side, delimiter, ellipsis);
        results.push(label);
    }
    results
}

fn disambiguate_one(idx: usize, parts: &[&str], all: &[Vec<&str>], side: Side, delim: char, ellipsis: &str) -> String {
    let anchor = match side {
        Side::Right => parts.last().copied().unwrap_or(""),
        Side::Left => parts.first().copied().unwrap_or(""),
    };

    // Find indices that share the same anchor
    let conflicts: Vec<usize> = all
        .iter()
        .enumerate()
        .filter(|(j, other)| {
            *j != idx && {
                let other_anchor = match side {
                    Side::Right => other.last().copied().unwrap_or(""),
                    Side::Left => other.first().copied().unwrap_or(""),
                };
                other_anchor == anchor
            }
        })
        .map(|(j, _)| j)
        .collect();

    if conflicts.is_empty() {
        return anchor.to_string();
    }

    // Walk from anchor outward, find first segment that distinguishes
    let depth = find_distinguishing_depth(idx, parts, &conflicts, all, side);
    format_abbreviated(parts, depth, side, delim, ellipsis)
}

/// Find how many segments from the anchor are needed to be unique.
fn find_distinguishing_depth(_idx: usize, parts: &[&str], conflicts: &[usize], all: &[Vec<&str>], side: Side) -> usize {
    let len = parts.len();
    for depth in 1..len {
        let my_seg = segment_at_depth(parts, depth, side);
        let still_ambiguous = conflicts.iter().any(|&j| {
            let other = &all[j];
            if depth >= other.len() {
                return false;
            }
            segment_at_depth(other, depth, side) == my_seg
        });
        if !still_ambiguous {
            return depth;
        }
    }
    // Full path needed
    len - 1
}

/// Get the segment at a given depth from the anchor side.
fn segment_at_depth<'a>(parts: &[&'a str], depth: usize, side: Side) -> &'a str {
    match side {
        Side::Right => {
            let idx = parts.len().saturating_sub(1 + depth);
            parts.get(idx).copied().unwrap_or("")
        }
        Side::Left => parts.get(depth).copied().unwrap_or(""),
    }
}

/// Format: show anchor + distinguishing segment, collapse middle with ellipsis.
fn format_abbreviated(parts: &[&str], depth: usize, side: Side, delim: char, ellipsis: &str) -> String {
    let len = parts.len();
    match side {
        Side::Right => {
            // parts = [a, b, c, d, filename]
            // depth=1 means we need parts[len-2] (d) + filename
            let anchor_idx = len - 1;
            let dist_idx = len.saturating_sub(1 + depth);
            if depth + 1 >= len || dist_idx + 1 == anchor_idx {
                // Adjacent or full path — no ellipsis needed
                let start = dist_idx;
                parts[start..].join(&delim.to_string())
            } else {
                // dist_idx ... anchor_idx with gap
                let d = delim.to_string();
                format!("{}{}{}{}{}", parts[dist_idx], d, ellipsis, d, parts[anchor_idx])
            }
        }
        Side::Left => {
            let dist_idx = depth;
            if dist_idx <= 1 || dist_idx + 1 >= len {
                parts[..=dist_idx].join(&delim.to_string())
            } else {
                let d = delim.to_string();
                format!("{}{}{}{}{}", parts[0], d, ellipsis, d, parts[dist_idx])
            }
        }
    }
}

fn all_unique(items: &[&str]) -> bool {
    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            if items[i] == items[j] {
                return false;
            }
        }
    }
    true
}
