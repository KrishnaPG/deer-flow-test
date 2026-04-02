use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, RecordFamily, StorageDisposition};
use serde::{Deserialize, Serialize};

use crate::define_record;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L0SourceBody {
    pub summary: String,
}
define_record!(
    L0SourceRecord,
    L0SourceBody,
    RecordFamily::L0Source,
    CanonicalLevel::L0,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L1SanitizedBody {
    pub summary: String,
}
define_record!(
    L1SanitizedRecord,
    L1SanitizedBody,
    RecordFamily::L1Sanitized,
    CanonicalLevel::L1,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L2ViewBody {
    pub summary: String,
}
define_record!(
    L2ViewRecord,
    L2ViewBody,
    RecordFamily::L2View,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L3InsightBody {
    pub summary: String,
}
define_record!(
    L3InsightRecord,
    L3InsightBody,
    RecordFamily::L3Insight,
    CanonicalLevel::L3,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L4PredictionBody {
    pub summary: String,
}
define_record!(
    L4PredictionRecord,
    L4PredictionBody,
    RecordFamily::L4Prediction,
    CanonicalLevel::L4,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L5PrescriptionBody {
    pub summary: String,
}
define_record!(
    L5PrescriptionRecord,
    L5PrescriptionBody,
    RecordFamily::L5Prescription,
    CanonicalLevel::L5,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L6OutcomeBody {
    pub summary: String,
}
define_record!(
    L6OutcomeRecord,
    L6OutcomeBody,
    RecordFamily::L6Outcome,
    CanonicalLevel::L6,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);
