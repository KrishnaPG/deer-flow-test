use serde::{Deserialize, Serialize};

use crate::{
    AgentId, ArtifactId, CanonicalLevel, CanonicalPlane, IdempotencyKey, LogicalWriteId, MissionId,
    RunId, TraceId, WriterId,
};

pub type StorageTag = (String, String);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageHierarchyTag(String);

impl StorageHierarchyTag {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoragePayloadKind(String);

impl StoragePayloadKind {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoragePayloadFormat(String);

impl StoragePayloadFormat {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn extension(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageLayout {
    pub hierarchy: StorageHierarchyTag,
    pub level: CanonicalLevel,
    pub plane: CanonicalPlane,
    pub payload_kind: StoragePayloadKind,
    pub format: StoragePayloadFormat,
    pub partition_tags: Vec<StorageTag>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageCorrelationIds {
    pub mission_id: Option<MissionId>,
    pub run_id: Option<RunId>,
    pub agent_id: Option<AgentId>,
    pub artifact_id: Option<ArtifactId>,
    pub trace_id: Option<TraceId>,
    pub extra: Vec<StorageTag>,
}

impl StorageCorrelationIds {
    pub fn is_complete(&self) -> bool {
        self.mission_id.is_some()
            && self.run_id.is_some()
            && self.agent_id.is_some()
            && self.artifact_id.is_some()
            && self.trace_id.is_some()
    }

    pub fn from_tuples(pairs: Vec<(String, String)>) -> Self {
        let mut result = Self::default();
        for (k, v) in pairs {
            match k.as_str() {
                "mission_id" => result.mission_id = Some(MissionId::new(v)),
                "run_id" => result.run_id = Some(RunId::new(v)),
                "agent_id" => result.agent_id = Some(AgentId::new(v)),
                "artifact_id" => result.artifact_id = Some(ArtifactId::new(v)),
                "trace_id" => result.trace_id = Some(TraceId::new(v)),
                _ => result.extra.push((k, v)),
            }
        }
        result
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageLineageRefs {
    pub parent_refs: Vec<String>,
    pub derived_from_refs: Vec<String>,
}

impl StorageLineageRefs {
    pub fn from_parent_refs(parents: Vec<String>) -> Self {
        Self {
            parent_refs: parents,
            derived_from_refs: vec![],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageRequestMetadata {
    pub logical_write_id: Option<LogicalWriteId>,
    pub idempotency_key: Option<IdempotencyKey>,
    pub writer_identity: Option<WriterId>,
    pub known_content_hash: Option<String>,
    pub logical_filename: Option<String>,
    pub correlation: StorageCorrelationIds,
    pub lineage: StorageLineageRefs,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoragePayloadDescriptor {
    InlineBytes { bytes: Vec<u8> },
    ExternalRef { uri: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageQosTemplate {
    FireAndForget,
    DurableArtifact,
    RealtimeStream,
    ConflictIntent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageQosPolicy {
    pub template: StorageQosTemplate,
    pub durability_class: String,
    pub progress_emission: String,
    pub worker_priority: String,
}

impl StorageQosPolicy {
    pub fn from_template(template: StorageQosTemplate) -> Self {
        match template {
            StorageQosTemplate::FireAndForget => Self {
                template,
                durability_class: "balanced".into(),
                progress_emission: "internal-only".into(),
                worker_priority: "normal".into(),
            },
            StorageQosTemplate::DurableArtifact => Self {
                template,
                durability_class: "strong".into(),
                progress_emission: "internal-only".into(),
                worker_priority: "high".into(),
            },
            StorageQosTemplate::RealtimeStream => Self {
                template,
                durability_class: "latency-biased".into(),
                progress_emission: "internal-only".into(),
                worker_priority: "realtime".into(),
            },
            StorageQosTemplate::ConflictIntent => Self {
                template,
                durability_class: "strong".into(),
                progress_emission: "internal-only".into(),
                worker_priority: "high".into(),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageRejectionReason {
    MissingIdempotencyKey,
    MissingCorrelationIds,
    MissingControlTargets,
    MissingWriterIdentity,
    MissingLayoutField,
}

impl StorageRejectionReason {
    pub fn as_str(self) -> &'static str {
        match self {
            StorageRejectionReason::MissingIdempotencyKey => "missing idempotency key",
            StorageRejectionReason::MissingCorrelationIds => "missing correlation ids",
            StorageRejectionReason::MissingControlTargets => "missing control targets",
            StorageRejectionReason::MissingWriterIdentity => "missing writer identity",
            StorageRejectionReason::MissingLayoutField => "missing layout field",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppendDataRequest {
    pub layout: StorageLayout,
    pub metadata: StorageRequestMetadata,
    pub qos: StorageQosPolicy,
    pub payload: StoragePayloadDescriptor,
}

impl AppendDataRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        validate_layout(&self.layout)?;
        validate_metadata(&self.metadata)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlIntentKind {
    Exclusion,
    Supersession,
    Reprocess,
    Invalidation,
    Backfill,
    OverrideRequest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppendControlRequest {
    pub control_kind: ControlIntentKind,
    pub target_refs: Vec<String>,
    pub metadata: StorageRequestMetadata,
    pub qos: StorageQosPolicy,
    pub rationale: Option<String>,
}

impl AppendControlRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        validate_metadata(&self.metadata)?;
        if self.target_refs.is_empty() {
            return Err(StorageRejectionReason::MissingControlTargets.as_str());
        }
        Ok(())
    }
}

fn validate_layout(layout: &StorageLayout) -> Result<(), &'static str> {
    if layout.hierarchy.as_str().is_empty()
        || layout.payload_kind.as_str().is_empty()
        || layout.format.as_str().is_empty()
    {
        return Err(StorageRejectionReason::MissingLayoutField.as_str());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileSavedTarget {
    /// View-relative path for warm cache materialization (not physical storage path).
    SingleFile { relative_path: String },
    /// Multi-file commit manifest. The manifest_path is view-relative.
    CommitManifest {
        manifest_path: String,
        member_count: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileAccepted {
    pub logical_write_id: LogicalWriteId,
    pub idempotency_key: IdempotencyKey,
    pub topic_class: String,
    pub routing_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSaved {
    pub logical_write_id: LogicalWriteId,
    pub target: FileSavedTarget,
    pub hierarchy: StorageHierarchyTag,
    pub level: CanonicalLevel,
    pub plane: CanonicalPlane,
    pub payload_kind: StoragePayloadKind,
    pub format: StoragePayloadFormat,
    pub correlation_ids: StorageCorrelationIds,
    pub lineage_refs: StorageLineageRefs,
    pub routing_tags: Vec<StorageTag>,
    /// Base58-encoded blake3 content hash. This is the file's unique identity.
    pub content_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivationTrigger {
    pub logical_write_id: LogicalWriteId,
    pub relative_target: String,
    pub hierarchy: StorageHierarchyTag,
    pub level: CanonicalLevel,
    pub plane: CanonicalPlane,
    pub payload_kind: StoragePayloadKind,
    pub format: StoragePayloadFormat,
    pub correlation_ids: StorageCorrelationIds,
    pub lineage_refs: StorageLineageRefs,
    pub routing_tags: Vec<StorageTag>,
    pub content_hash: String,
}

fn validate_metadata(metadata: &StorageRequestMetadata) -> Result<(), &'static str> {
    if metadata.idempotency_key.is_none() {
        return Err(StorageRejectionReason::MissingIdempotencyKey.as_str());
    }
    if metadata.writer_identity.is_none() {
        return Err(StorageRejectionReason::MissingWriterIdentity.as_str());
    }
    if !metadata.correlation.is_complete() {
        return Err(StorageRejectionReason::MissingCorrelationIds.as_str());
    }
    Ok(())
}
