use deepwash::tasks::clean::{filter_sort, total_size, Match};
use std::path::PathBuf;

fn m(size: u64, kind: &str) -> Match {
    Match { path: PathBuf::from(format!("/p/{}", kind)), size, kind: kind.to_string() }
}

#[test]
fn filter_sort_orders_desc_and_applies_min() {
    let input = vec![m(100, "a"), m(3000, "b"), m(500, "c")];
    let out = filter_sort(input, Some(400));
    let sizes: Vec<u64> = out.iter().map(|x| x.size).collect();
    assert_eq!(sizes, vec![3000, 500]); // 100 filtered out, sorted desc
}

#[test]
fn filter_sort_no_min_keeps_all() {
    let out = filter_sort(vec![m(1, "a"), m(2, "b")], None);
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].size, 2);
}

#[test]
fn total_size_sums() {
    assert_eq!(total_size(&[m(10, "a"), m(32, "b")]), 42);
}
