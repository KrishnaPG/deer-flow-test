use deer_foundation_contracts::{
    AsIsHash, CanonicalLevel, CanonicalPlane, CorrelationMeta, IdentityMeta, LineageMeta,
    RecordFamily, RecordHeader, RecordId, StorageDisposition,
};

#[test]
fn record_header_carries_hash_anchored_identity() {
    let header = RecordHeader::new(
        RecordId::from_static("rec_1"),
        RecordFamily::AsIsRepresentation,
        CanonicalLevel::L2,
        CanonicalPlane::AsIs,
        StorageDisposition::StorageNative,
        IdentityMeta::hash_anchored(
            RecordId::from_static("rec_1"),
            Some(AsIsHash::from_static("sha256:artifact")),
            None,
            None,
        ),
        CorrelationMeta::default(),
        LineageMeta::root(),
    );

    assert_eq!(header.level, CanonicalLevel::L2);
    assert_eq!(header.plane, CanonicalPlane::AsIs);
    assert_eq!(header.family, RecordFamily::AsIsRepresentation);
    assert_eq!(
        header.identity.as_is_hash.as_ref().map(AsIsHash::as_str),
        Some("sha256:artifact")
    );
}
