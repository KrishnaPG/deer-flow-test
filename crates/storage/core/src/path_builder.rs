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
