use std::fmt;
use std::str::FromStr;

use berg10_storage_vfs::ContentHash;
use chrono::{DateTime, Utc};
use deer_foundation_contracts::{
    CanonicalLevel, CanonicalPlane, StoragePayloadFormat, StoragePayloadKind, WriterId,
};
use serde::{Deserialize, Serialize};

/// Berg10-scoped semantic hierarchy describing what kind of meaning a piece of
/// immutable content carries.
///
/// This is intentionally distinct from a virtual folder hierarchy.
///
/// - A `Berg10DataHierarchy` answers: "what kind of thing is this content?"
/// - A `VirtualFolderHierarchy` answers: "how should matching content be
///   organized for browsing?"
///
/// The architecture docs describe three interleaved data hierarchies:
///
/// - `Orchestration`: what the agent/runtime did.
/// - `ArtifactContent`: what the agent/user/tool produced.
/// - `Presentation`: what a downstream consumer wants to see as a view model.
///
/// A single user-visible session can contain records from all three
/// hierarchies. Keeping this as a strong enum prevents accidentally storing
/// orchestration data as artifact content or vice versa.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Berg10DataHierarchy {
    /// Agent execution metadata: tool calls, lifecycle events, retries,
    /// approvals, failures, and other orchestration traces.
    Orchestration,
    /// Agent and user produced content: prompts, responses, raw tool outputs,
    /// gathered sources, summaries, and derived artifacts.
    ArtifactContent,
    /// Consumer-facing projections and view models derived from lower levels.
    ///
    /// This is intentionally separate from `ArtifactContent` because
    /// presentation records are optimized for a particular consumer or UX,
    /// not for canonical storage truth. For example, a thread view, a
    /// commander dashboard row, and a researcher-oriented summary can all be
    /// derived from the same underlying artifact/orchestration content.
    Presentation,
}

impl Berg10DataHierarchy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Orchestration => "orchestration",
            Self::ArtifactContent => "artifact_content",
            Self::Presentation => "presentation",
        }
    }
}

impl fmt::Display for Berg10DataHierarchy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Berg10DataHierarchy {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "orchestration" => Ok(Self::Orchestration),
            "artifact_content" => Ok(Self::ArtifactContent),
            "presentation" => Ok(Self::Presentation),
            other => Err(format!("unknown Berg10 data hierarchy: {other}")),
        }
    }
}

/// Berg10-scoped wrapper around the canonical storage levels used throughout the
/// state-server architecture.
///
/// A level answers: "how far has this content been transformed relative to its
/// current consumer?"
///
/// Levels are about semantic readiness and derivation stage, not about physical
/// storage representation. A record's plane tells you *how* it is stored, while
/// its level tells you *how processed* it is.
///
/// Practical interpretation of levels:
///
/// - `L0`: crude/raw data captured from the generator boundary.
/// - `L1`: sanitized/normalized forms of `L0`.
/// - `L2`: views or domain-shaped projections on top of lower levels.
/// - `L3`: aggregates, insights, or historical interpretations.
/// - `L4`: generated/predictive/final artifacts.
/// - `L5`: prescriptions/plans intended to influence `L4` predicted outcomes.
/// - `L6`: outcome evaluation/deviation versus what was expected/predicted.
///
/// The same underlying fact can appear at multiple levels in different derived
/// forms. For example, a final artifact may be `L4` in the artifact hierarchy,
/// while a UI-ready projection of that same artifact later appears as
/// `Presentation/L2`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Berg10DataLevel(pub CanonicalLevel);

impl Berg10DataLevel {
    /// Raw/crude source data with minimal interpretation.
    ///
    /// Use this for directly captured runtime events, raw prompts, raw tool
    /// outputs, logs, audio/video, and other source-of-truth payloads.
    pub const L0: Self = Self(CanonicalLevel::L0);
    /// Sanitized or normalized data derived from `L0`.
    ///
    /// Use this when the payload has been cleaned, deduplicated, structurally
    /// normalized, or lightly corrected without changing its core meaning.
    pub const L1: Self = Self(CanonicalLevel::L1);
    /// Consumer-relative views and domain-shaped projections.
    ///
    /// Use this for query-ready views, joined projections, presentation-ready
    /// slices, or domain models built from lower levels.
    pub const L2: Self = Self(CanonicalLevel::L2);
    /// Aggregates, insights, and historical interpretation.
    ///
    /// Use this when the record captures a higher-order conclusion such as
    /// summaries, metrics, rollups, extracted insights, or trend signals.
    pub const L3: Self = Self(CanonicalLevel::L3);
    /// Predictive, generated, or final output artifacts.
    ///
    /// Use this for content that did not exist verbatim in the source stream,
    /// such as reports, generated code, synthesized media, or forecasts.
    pub const L4: Self = Self(CanonicalLevel::L4);
    /// Prescriptions or plans intended to alter future outcomes.
    ///
    /// Use this for recommended actions, interventions, mitigations, or
    /// alternative plans derived from lower-level observations.
    pub const L5: Self = Self(CanonicalLevel::L5);
    /// Outcome evaluation and deviation versus projections.
    ///
    /// Use this to record whether `L4` predictions or generated outcomes matched
    /// observed reality, including the influence of any `L5` interventions.
    pub const L6: Self = Self(CanonicalLevel::L6);

    pub fn as_str(self) -> &'static str {
        match self.0 {
            CanonicalLevel::L0 => "L0",
            CanonicalLevel::L1 => "L1",
            CanonicalLevel::L2 => "L2",
            CanonicalLevel::L3 => "L3",
            CanonicalLevel::L4 => "L4",
            CanonicalLevel::L5 => "L5",
            CanonicalLevel::L6 => "L6",
        }
    }
}

impl fmt::Display for Berg10DataLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<CanonicalLevel> for Berg10DataLevel {
    fn from(value: CanonicalLevel) -> Self {
        Self(value)
    }
}

impl From<Berg10DataLevel> for CanonicalLevel {
    fn from(value: Berg10DataLevel) -> Self {
        value.0
    }
}

impl FromStr for Berg10DataLevel {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "L0" => Ok(Self::L0),
            "L1" => Ok(Self::L1),
            "L2" => Ok(Self::L2),
            "L3" => Ok(Self::L3),
            "L4" => Ok(Self::L4),
            "L5" => Ok(Self::L5),
            "L6" => Ok(Self::L6),
            other => Err(format!("unknown Berg10 data level: {other}")),
        }
    }
}

/// Berg10-scoped wrapper around canonical storage planes.
///
/// A plane answers: "what physical representation of this content is stored?"
///
/// Planes are intentionally *not* levels:
///
/// - A level describes semantic processing stage (`L0` raw, `L1` normalized,
///   `L2` view, etc.).
/// - A plane describes storage representation (`AsIs`, `Chunks`,
///   `Embeddings`).
///
/// This distinction matters because the same semantic level can exist on
/// multiple planes. For example, `L1` normalized text may exist both as
/// canonical `AsIs` content and as derived `Chunks` for retrieval. Likewise,
/// embeddings are a storage representation derived from chunk content; they are
/// not themselves a semantic level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Berg10StoragePlane(pub CanonicalPlane);

impl Berg10StoragePlane {
    /// Canonical payload bytes exactly as stored for that level.
    pub const AS_IS: Self = Self(CanonicalPlane::AsIs);
    /// Segmented or chunked derivatives of canonical payloads.
    pub const CHUNKS: Self = Self(CanonicalPlane::Chunks);
    /// Embedding/vector representations that point back to chunk payloads.
    pub const EMBEDDINGS: Self = Self(CanonicalPlane::Embeddings);

    pub fn as_str(self) -> &'static str {
        match self.0 {
            CanonicalPlane::AsIs => "as-is",
            CanonicalPlane::Chunks => "chunks",
            CanonicalPlane::Embeddings => "embeddings",
        }
    }
}

impl fmt::Display for Berg10StoragePlane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<CanonicalPlane> for Berg10StoragePlane {
    fn from(value: CanonicalPlane) -> Self {
        Self(value)
    }
}

impl From<Berg10StoragePlane> for CanonicalPlane {
    fn from(value: Berg10StoragePlane) -> Self {
        value.0
    }
}

impl FromStr for Berg10StoragePlane {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "as-is" => Ok(Self::AS_IS),
            "chunks" => Ok(Self::CHUNKS),
            "embeddings" => Ok(Self::EMBEDDINGS),
            other => Err(format!("unknown Berg10 storage plane: {other}")),
        }
    }
}

/// Strongly typed payload kind carried by Berg10 content.
///
/// `payload_kind` answers: "what is this payload within its hierarchy/level?"
/// Examples: `message.delta`, `tool.started`, `session.segment`,
/// `thread.view`, `report.markdown`.
///
/// This remains string-backed for extensibility, but wrapping it prevents it
/// from being confused with arbitrary strings elsewhere in the codebase.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Berg10PayloadKind(pub StoragePayloadKind);

impl Berg10PayloadKind {
    pub fn new(value: impl Into<String>) -> Self {
        Self(StoragePayloadKind::new(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for Berg10PayloadKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<StoragePayloadKind> for Berg10PayloadKind {
    fn from(value: StoragePayloadKind) -> Self {
        Self(value)
    }
}

impl From<&str> for Berg10PayloadKind {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for Berg10PayloadKind {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<Berg10PayloadKind> for StoragePayloadKind {
    fn from(value: Berg10PayloadKind) -> Self {
        value.0
    }
}

/// Strongly typed payload format for Berg10 content.
///
/// `payload_format` answers: "what serialization or media format is this
/// payload encoded in?" Examples: `json`, `jsonl`, `md`, `mp3`, `parquet`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Berg10PayloadFormat(pub StoragePayloadFormat);

impl Berg10PayloadFormat {
    pub fn new(value: impl Into<String>) -> Self {
        Self(StoragePayloadFormat::new(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn extension(&self) -> &str {
        self.0.extension()
    }
}

impl fmt::Display for Berg10PayloadFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<StoragePayloadFormat> for Berg10PayloadFormat {
    fn from(value: StoragePayloadFormat) -> Self {
        Self(value)
    }
}

impl From<&str> for Berg10PayloadFormat {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for Berg10PayloadFormat {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<Berg10PayloadFormat> for StoragePayloadFormat {
    fn from(value: Berg10PayloadFormat) -> Self {
        value.0
    }
}

/// Application-defined key for metadata tags attached to immutable content.
///
/// Tags are intentionally open-ended and user/system defined. They are not a
/// replacement for strongly typed architectural fields such as hierarchy, level,
/// and plane.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TagKey(String);

impl TagKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TagKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for TagKey {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for TagKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// Application-defined value for metadata tags attached to immutable content.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TagValue(String);

impl TagValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TagValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for TagValue {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for TagValue {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// A strongly typed key/value attribute used for routing, filtering, and
/// virtual hierarchy construction.
///
/// These are auxiliary attributes, not identity fields. The same content hash
/// may be referenced through many tags and many virtual views later.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentTag {
    pub key: TagKey,
    pub value: TagValue,
}

impl ContentTag {
    pub fn new(key: impl Into<TagKey>, value: impl Into<TagValue>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Named lineage reference to another immutable content item or external source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LineageRef(String);

impl LineageRef {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LineageRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for LineageRef {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for LineageRef {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// Optional user-facing filename hint for a content record when a view chooses
/// to present it as a file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LogicalFilename(String);

impl LogicalFilename {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LogicalFilename {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for LogicalFilename {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for LogicalFilename {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// A single immutable content item registered in Berg10.
///
/// This is intentionally not called a file record. Berg10 stores immutable
/// content plus attributes; files are only one possible virtual presentation of
/// that content later.
///
/// The core identity is `content_hash`. Everything else describes the content's
/// architectural role (`data_hierarchy`, `data_level`, `storage_plane`) and its
/// query/browse metadata (`payload_kind`, tags, lineage, logical filename).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentRecord {
    pub content_hash: ContentHash,
    pub data_hierarchy: Berg10DataHierarchy,
    pub data_level: Berg10DataLevel,
    pub storage_plane: Berg10StoragePlane,
    pub payload_kind: Berg10PayloadKind,
    pub payload_format: Berg10PayloadFormat,
    pub payload_size_bytes: u64,
    pub correlation_ids: Vec<ContentTag>,
    pub lineage_refs: Vec<LineageRef>,
    pub routing_tags: Vec<ContentTag>,
    pub written_at: DateTime<Utc>,
    pub writer_identity: Option<WriterId>,
    pub logical_filename: Option<LogicalFilename>,
}

/// Activation state of a virtual folder hierarchy definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HierarchyStatus {
    Active,
    Inactive,
}

impl HierarchyStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

impl fmt::Display for HierarchyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for HierarchyStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            other => Err(format!("unknown hierarchy status: {other}")),
        }
    }
}

/// Stable identifier for a virtual folder hierarchy definition.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HierarchyName(String);

impl HierarchyName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HierarchyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for HierarchyName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for HierarchyName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// A single segment in a virtual folder path builder.
///
/// Built-in segments target strongly typed architectural fields. `Tag(...)`
/// segments target arbitrary metadata tags. This keeps view-construction logic
/// separate from the stored semantics of the content itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum HierarchyPathSegment {
    DataHierarchy,
    DataLevel,
    StoragePlane,
    PayloadKind,
    PayloadFormat,
    WriterIdentity,
    Tag(TagKey),
}

impl HierarchyPathSegment {
    pub fn legacy_name(&self) -> String {
        match self {
            Self::DataHierarchy => "data_hierarchy".to_string(),
            Self::DataLevel => "data_level".to_string(),
            Self::StoragePlane => "storage_plane".to_string(),
            Self::PayloadKind => "payload_kind".to_string(),
            Self::PayloadFormat => "payload_format".to_string(),
            Self::WriterIdentity => "writer_identity".to_string(),
            Self::Tag(tag) => tag.as_str().to_string(),
        }
    }
}

impl FromStr for HierarchyPathSegment {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "data_hierarchy" | "hierarchy" => Ok(Self::DataHierarchy),
            "data_level" | "level" => Ok(Self::DataLevel),
            "storage_plane" | "plane" => Ok(Self::StoragePlane),
            "payload_kind" => Ok(Self::PayloadKind),
            "payload_format" => Ok(Self::PayloadFormat),
            "writer_identity" => Ok(Self::WriterIdentity),
            other => Ok(Self::Tag(TagKey::new(other))),
        }
    }
}

/// User-defined virtual folder hierarchy over content attributes. This defines a
/// browsing order only; it does not change the stored content or its identity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtualFolderHierarchy {
    pub hierarchy_name: HierarchyName,
    pub hierarchy_order: Vec<HierarchyPathSegment>,
    pub filter_expr: Option<String>,
    pub status: HierarchyStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct HierarchyCheckoutReceipt {
    pub hierarchy_name: HierarchyName,
    pub checkout_path: String,
    pub file_count: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct HierarchyCheckoutInfo {
    pub hierarchy_name: HierarchyName,
    pub checkout_path: String,
    pub file_count: usize,
    pub status: HierarchyStatus,
}
