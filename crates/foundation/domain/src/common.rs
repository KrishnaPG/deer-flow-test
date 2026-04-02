use deer_foundation_contracts::{
    CanonicalLevel, CanonicalPlane, CorrelationMeta, IdentityMeta, LineageMeta, RecordFamily,
    RecordHeader, RecordId, StorageDisposition,
};

pub fn build_header(
    record_id: RecordId,
    family: RecordFamily,
    level: CanonicalLevel,
    plane: CanonicalPlane,
    storage: StorageDisposition,
    identity: IdentityMeta,
    lineage: LineageMeta,
) -> RecordHeader {
    RecordHeader::new(
        record_id,
        family,
        level,
        plane,
        storage,
        identity,
        CorrelationMeta::default(),
        lineage,
    )
}

#[macro_export]
macro_rules! define_record {
    ($name:ident, $body:ty, $family:expr, $level:expr, $plane:expr, $storage:expr) => {
        #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            pub header: deer_foundation_contracts::RecordHeader,
            pub body: $body,
        }

        impl $name {
            pub fn new(
                record_id: deer_foundation_contracts::RecordId,
                identity: deer_foundation_contracts::IdentityMeta,
                lineage: deer_foundation_contracts::LineageMeta,
                body: $body,
            ) -> Self {
                Self {
                    header: crate::common::build_header(
                        record_id, $family, $level, $plane, $storage, identity, lineage,
                    ),
                    body,
                }
            }

            pub fn record_id(&self) -> &deer_foundation_contracts::RecordId {
                &self.header.record_id
            }
        }

        impl deer_foundation_contracts::CanonicalRecord for $name {
            fn header(&self) -> &deer_foundation_contracts::RecordHeader {
                &self.header
            }
        }
    };
}
