use deer_foundation_contracts::{
    CanonicalLevel, CanonicalPlane, StorageHierarchyTag, StoragePayloadFormat, StoragePayloadKind,
};
use deer_storage_core::view_path_builder::build_relative_path;

#[test]
fn canonical_paths_are_relative_and_storage_owned() {
    let path = build_relative_path(
        &StorageHierarchyTag::new("B"),
        CanonicalLevel::L4,
        CanonicalPlane::AsIs,
        &StoragePayloadKind::new("thumbnail"),
        &StoragePayloadFormat::new("png"),
        &[
            ("size".into(), "64 x 64".into()),
            ("capture date".into(), "2026/04/04".into()),
            ("kind".into(), "Meeting Audio".into()),
        ],
        "abc123",
    );

    assert_eq!(
        path,
        "b/L4/as-is/thumbnail/capture-date=2026-04-04/kind=meeting-audio/size=64-x-64/abc123.png"
    );
}

#[test]
fn path_segments_are_sanitized_against_path_shaping() {
    let path = build_relative_path(
        &StorageHierarchyTag::new("../../etc"),
        CanonicalLevel::L0,
        CanonicalPlane::AsIs,
        &StoragePayloadKind::new("chat-note"),
        &StoragePayloadFormat::new("jsonl"),
        &[],
        "hash/../../../passwd",
    );

    assert_eq!(path, "etc/L0/as-is/chat-note/hash-passwd.jsonl");
}

#[test]
fn view_path_with_custom_hierarchy() {
    use deer_storage_core::view_path_builder::build_view_path;

    let path = build_view_path(
        &["year".to_string(), "singer".to_string()],
        "A",
        "L0",
        "as-is",
        "mp3",
        "mp3",
        &[
            ("year".to_string(), "2024".to_string()),
            ("singer".to_string(), "Adele".to_string()),
        ],
        "abc123",
    );

    assert_eq!(path, "2024.adele.abc123.mp3");
}

#[test]
fn view_path_with_reversed_hierarchy() {
    use deer_storage_core::view_path_builder::build_view_path;

    let path = build_view_path(
        &["singer".to_string(), "year".to_string()],
        "A",
        "L0",
        "as-is",
        "mp3",
        "mp3",
        &[
            ("year".to_string(), "2024".to_string()),
            ("singer".to_string(), "Adele".to_string()),
        ],
        "abc123",
    );

    assert_eq!(path, "adele.2024.abc123.mp3");
}
