use deer_foundation_contracts::{
    CanonicalLevel, CanonicalPlane, StorageHierarchyTag, StoragePayloadFormat, StoragePayloadKind,
};

use crate::layout::{level_segment, plane_segment};

fn sanitize_segment(value: &str) -> String {
    let mut sanitized = String::new();
    let mut last_was_dash = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            sanitized.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            sanitized.push('-');
            last_was_dash = true;
        }
    }

    sanitized.trim_matches('-').to_owned()
}

fn canonical_partition_segments(partitions: &[(String, String)]) -> Vec<String> {
    let mut segments: Vec<_> = partitions
        .iter()
        .map(|(key, value)| (sanitize_segment(key), sanitize_segment(value)))
        .collect();

    segments.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));

    segments
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect()
}

/// Build a view-relative path from layout dimensions and content hash.
/// Used for warm cache materialization, not physical storage paths.
pub fn build_relative_path(
    hierarchy: &StorageHierarchyTag,
    level: CanonicalLevel,
    plane: CanonicalPlane,
    kind: &StoragePayloadKind,
    format: &StoragePayloadFormat,
    partitions: &[(String, String)],
    content_hash: &str,
) -> String {
    let mut path = String::new();
    path.push_str(&sanitize_segment(hierarchy.as_str()));
    path.push('/');
    path.push_str(level_segment(level));
    path.push('/');
    path.push_str(plane_segment(plane));
    path.push('/');
    path.push_str(&sanitize_segment(kind.as_str()));

    for segment in canonical_partition_segments(partitions) {
        path.push('/');
        path.push_str(&segment);
    }

    path.push('/');
    path.push_str(&sanitize_segment(content_hash));
    path.push('.');
    path.push_str(&sanitize_segment(format.extension()));
    path
}

/// Build a virtual-folder-hierarchy-relative path from a custom hierarchy ordering and file attributes.
/// This is used for warm cache materialization where the hierarchy defines its own ordering.
///
/// Supported attribute names: "hierarchy", "level", "plane", "payload_kind", "payload_format",
/// and any key found in routing_tags.
pub fn build_virtual_folder_hierarchy_path(
    hierarchy_order: &[String],
    hierarchy: &str,
    level: &str,
    plane: &str,
    payload_kind: &str,
    payload_format: &str,
    routing_tags: &[(String, String)],
    content_hash: &str,
) -> String {
    let mut parts = Vec::new();

    for attr in hierarchy_order {
        let value = match attr.as_str() {
            "hierarchy" => hierarchy.to_string(),
            "level" => level.to_string(),
            "plane" => plane.to_string(),
            "payload_kind" => payload_kind.to_string(),
            "payload_format" => payload_format.to_string(),
            _ => routing_tags
                .iter()
                .find(|(k, _)| k == attr)
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "unknown".to_string()),
        };

        if !value.is_empty() {
            parts.push(sanitize_segment(&value));
        }
    }

    parts.push(sanitize_segment(content_hash));
    parts.push(sanitize_segment(payload_format));

    parts.join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_paths_are_relative_and_storage_owned() {
        let path = build_relative_path(
            &StorageHierarchyTag::new("B"),
            CanonicalLevel::L4,
            CanonicalPlane::AsIs,
            &StoragePayloadKind::new("thumbnail"),
            &StoragePayloadFormat::new("png"),
            &[("size".into(), "64x64".into())],
            "abc123",
        );

        assert_eq!(path, "b/L4/as-is/thumbnail/size=64x64/abc123.png");
    }

    #[test]
    fn path_segments_are_sanitized_against_path_shaping() {
        let path = build_relative_path(
            &StorageHierarchyTag::new("A/B!C"),
            CanonicalLevel::L0,
            CanonicalPlane::AsIs,
            &StoragePayloadKind::new("chat note"),
            &StoragePayloadFormat::new("jsonl"),
            &[],
            "hash",
        );

        assert_eq!(path, "a-b-c/L0/as-is/chat-note/hash.jsonl");
    }

    #[test]
    fn virtual_folder_hierarchy_path_with_custom_ordering() {
        let path = build_virtual_folder_hierarchy_path(
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
    fn virtual_folder_hierarchy_path_with_reversed_ordering() {
        let path = build_virtual_folder_hierarchy_path(
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
}
