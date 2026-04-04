use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane};

pub fn level_segment(level: CanonicalLevel) -> &'static str {
    match level {
        CanonicalLevel::L0 => "L0",
        CanonicalLevel::L1 => "L1",
        CanonicalLevel::L2 => "L2",
        CanonicalLevel::L3 => "L3",
        CanonicalLevel::L4 => "L4",
        CanonicalLevel::L5 => "L5",
        CanonicalLevel::L6 => "L6",
    }
}

pub fn plane_segment(plane: CanonicalPlane) -> &'static str {
    match plane {
        CanonicalPlane::AsIs => "as-is",
        CanonicalPlane::Chunks => "chunks",
        CanonicalPlane::Embeddings => "embeddings",
    }
}
