use deepwash::utils::{parse_ids, resolve_scope};

#[test]
fn parse_ids_splits_nonempty_lines() {
    assert_eq!(parse_ids("a\nb\nc\n"), vec!["a", "b", "c"]);
}

#[test]
fn parse_ids_ignores_blank_and_whitespace_lines() {
    assert_eq!(parse_ids("a\n\n  \nb\n"), vec!["a", "b"]);
}

#[test]
fn parse_ids_empty_input_is_empty() {
    assert!(parse_ids("").is_empty());
    assert!(parse_ids("\n  \n").is_empty());
}

#[test]
fn resolve_scope_full_forces_images_and_volumes() {
    assert_eq!(resolve_scope(false, false, true), (true, true));
}

#[test]
fn resolve_scope_full_overrides_individual_flags() {
    // --full wins even when individual flags are already set or unset.
    assert_eq!(resolve_scope(true, false, true), (true, true));
    assert_eq!(resolve_scope(false, true, true), (true, true));
}

#[test]
fn resolve_scope_without_full_passes_flags_through() {
    assert_eq!(resolve_scope(false, false, false), (false, false));
    assert_eq!(resolve_scope(true, false, false), (true, false));
    assert_eq!(resolve_scope(false, true, false), (false, true));
    assert_eq!(resolve_scope(true, true, false), (true, true));
}
