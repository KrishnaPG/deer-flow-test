use deer_storage_core::manifest::CommitManifest;

#[test]
fn file_saved_for_batch_requires_final_marker() {
    let manifest = CommitManifest::new(
        "manifest_1",
        vec!["one.parquet".into(), "two.parquet".into()],
    );

    assert!(!manifest.is_visible());

    let visible = manifest.mark_finalized();
    assert!(visible.is_visible());
    assert_eq!(visible.member_count(), 2);
}
