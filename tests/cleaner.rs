use deepwash::utils::{format_size, parse_ids, parse_size, resolve_scope};

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

#[test]
fn parse_size_bare_number_is_bytes() {
    assert_eq!(parse_size("1024"), Ok(1024));
}

#[test]
fn parse_size_suffixes_are_powers_of_1000() {
    assert_eq!(parse_size("1K"), Ok(1_000));
    assert_eq!(parse_size("500M"), Ok(500_000_000));
    assert_eq!(parse_size("1G"), Ok(1_000_000_000));
    assert_eq!(parse_size("2T"), Ok(2_000_000_000_000));
}

#[test]
fn parse_size_is_case_insensitive_and_decimal() {
    assert_eq!(parse_size("2.5g"), Ok(2_500_000_000));
    assert_eq!(parse_size("1g"), Ok(1_000_000_000));
}

#[test]
fn parse_size_rejects_garbage() {
    assert!(parse_size("").is_err());
    assert!(parse_size("abc").is_err());
    assert!(parse_size("1X").is_err());
    assert!(parse_size("G").is_err());
}

#[test]
fn format_size_picks_unit() {
    assert_eq!(format_size(7), "7 B");
    assert_eq!(format_size(512_000), "512 KB");
    assert_eq!(format_size(840_000_000), "840 MB");
    assert_eq!(format_size(1_500_000_000), "1.5 GB");
}
