pub mod carrier;
pub mod envelopes;
pub mod error;
pub mod governance;
pub mod promotions;
pub mod representation;

pub use carrier::{normalize_batch, NormalizedBatch};
pub use envelopes::RawEnvelopeBatch;
pub use error::NormalizationError;
pub use governance::emit_intent_records;
pub use promotions::validate_promotion;
pub use representation::normalize_representation_chain;
