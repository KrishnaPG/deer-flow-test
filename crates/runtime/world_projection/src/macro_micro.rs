pub fn macro_micro_label(source_kind: &str) -> &'static str {
    match source_kind {
        "task" => "macro_micro_consistent",
        _ => "macro_micro_generic",
    }
}
