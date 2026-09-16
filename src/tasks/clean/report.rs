use crate::tasks::clean::Match;
use crate::utils::format_size;

/// Keep matches at least `min_size` (all if None), sorted by size descending.
pub fn filter_sort(mut matches: Vec<Match>, min_size: Option<u64>) -> Vec<Match> {
    if let Some(min) = min_size {
        matches.retain(|m| m.size >= min);
    }
    matches.sort_by(|a, b| b.size.cmp(&a.size));
    matches
}

/// Sum of match sizes.
pub fn total_size(matches: &[Match]) -> u64 {
    matches.iter().map(|m| m.size).sum()
}

/// Print the ranked table of matches.
pub fn print_report(matches: &[Match]) {
    if matches.is_empty() {
        println!("⏭️ No matching folders found");
        return;
    }
    println!("   {:>9}   {:<14} {}", "SIZE", "KIND", "PATH");
    for m in matches {
        println!(
            "  {:>9}   {:<14} {}",
            format_size(m.size),
            m.kind,
            m.path.display()
        );
    }
}
