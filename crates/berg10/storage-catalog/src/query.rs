use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::RecordBatch;
use arrow_schema::SchemaRef;
use datafusion::prelude::*;
use datafusion::common::Result as DfResult;

use crate::types::FileRecord;

const FILES_TABLE: &str = "files";

pub struct QueryEngine {
    ctx: SessionContext,
}

impl QueryEngine {
    pub fn new() -> Self {
        let config = SessionConfig::new()
            .with_default_catalog_and_schema("memory", "default");
        let ctx = SessionContext::new_with_config(config);
        Self { ctx }
    }

    pub fn register_files(&self, records: &[FileRecord]) -> DfResult<()> {
        let (schema, batch) = records_to_batch(records);
        let provider = Arc::new(datafusion::datasource::memory::MemTable::try_new(
            schema,
            vec![vec![batch]],
        )?);
        let _ = self.ctx.register_table(datafusion::common::TableReference::bare(FILES_TABLE), provider);
        Ok(())
    }

    pub async fn query(&self, sql: &str) -> DfResult<Vec<FileRecord>> {
        let df = self.ctx.sql(sql).await?;
        let results = datafusion::physical_plan::collect(
            df.create_physical_plan().await?,
            self.ctx.task_ctx(),
        ).await?;
        batches_to_records(&results)
    }

    pub async fn filter(&self, predicate: &str) -> DfResult<Vec<FileRecord>> {
        let sql = format!("SELECT * FROM {} WHERE {}", FILES_TABLE, predicate);
        self.query(&sql).await
    }
}

impl Default for QueryEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn records_to_batch(records: &[FileRecord]) -> (SchemaRef, RecordBatch) {
    use arrow_array::builder::StringBuilder;
    use arrow_array::builder::Int64Builder;
    use arrow_array::builder::TimestampMillisecondBuilder;

    let _num_records = records.len();

    let mut content_hash_builder = StringBuilder::new();
    let mut hierarchy_builder = StringBuilder::new();
    let mut level_builder = StringBuilder::new();
    let mut plane_builder = StringBuilder::new();
    let mut payload_kind_builder = StringBuilder::new();
    let mut payload_format_builder = StringBuilder::new();
    let mut payload_size_builder = Int64Builder::new();
    let mut physical_location_builder = StringBuilder::new();
    let mut written_at_builder = TimestampMillisecondBuilder::new();
    let mut writer_identity_builder = StringBuilder::new();

    for r in records {
        content_hash_builder.append_value(&r.content_hash);
        hierarchy_builder.append_value(&r.hierarchy);
        level_builder.append_value(&r.level);
        plane_builder.append_value(&r.plane);
        payload_kind_builder.append_value(&r.payload_kind);
        payload_format_builder.append_value(&r.payload_format);
        payload_size_builder.append_value(r.payload_size_bytes as i64);
        physical_location_builder.append_value(&r.physical_location);
        written_at_builder.append_value(r.written_at.timestamp_millis());
        writer_identity_builder.append_value(r.writer_identity.as_deref().unwrap_or(""));
    }

    let schema = arrow_schema::SchemaRef::new(arrow_schema::Schema::new(vec![
        arrow_schema::Field::new("content_hash", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("hierarchy", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("level", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("plane", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("payload_kind", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("payload_format", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("payload_size_bytes", arrow_schema::DataType::Int64, false),
        arrow_schema::Field::new("physical_location", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("written_at", arrow_schema::DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, None), false),
        arrow_schema::Field::new("writer_identity", arrow_schema::DataType::Utf8, false),
    ]));

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(content_hash_builder.finish()),
            Arc::new(hierarchy_builder.finish()),
            Arc::new(level_builder.finish()),
            Arc::new(plane_builder.finish()),
            Arc::new(payload_kind_builder.finish()),
            Arc::new(payload_format_builder.finish()),
            Arc::new(payload_size_builder.finish()),
            Arc::new(physical_location_builder.finish()),
            Arc::new(written_at_builder.finish()),
            Arc::new(writer_identity_builder.finish()),
        ],
    ).unwrap();

    (schema, batch)
}

fn batches_to_records(batches: &[RecordBatch]) -> DfResult<Vec<FileRecord>> {
    let mut records = Vec::new();

    for batch in batches {
        let num_rows = batch.num_rows();
        let content_hash = batch.column(0).as_string::<i32>();
        let hierarchy = batch.column(1).as_string::<i32>();
        let level = batch.column(2).as_string::<i32>();
        let plane = batch.column(3).as_string::<i32>();
        let payload_kind = batch.column(4).as_string::<i32>();
        let payload_format = batch.column(5).as_string::<i32>();
        let payload_size = batch.column(6).as_primitive::<arrow_array::types::Int64Type>();
        let physical_location = batch.column(7).as_string::<i32>();
        let written_at = batch.column(8).as_primitive::<arrow_array::types::TimestampMillisecondType>();
        let writer_identity = batch.column(9).as_string::<i32>();

        for i in 0..num_rows {
            let written_at_ts = chrono::DateTime::from_timestamp_millis(
                written_at.value(i)
            ).unwrap_or_else(chrono::Utc::now);

            records.push(FileRecord {
                content_hash: content_hash.value(i).to_string(),
                hierarchy: hierarchy.value(i).to_string(),
                level: level.value(i).to_string(),
                plane: plane.value(i).to_string(),
                payload_kind: payload_kind.value(i).to_string(),
                payload_format: payload_format.value(i).to_string(),
                payload_size_bytes: payload_size.value(i) as u64,
                physical_location: physical_location.value(i).to_string(),
                correlation_ids: Vec::new(),
                lineage_refs: Vec::new(),
                routing_tags: Vec::new(),
                written_at: written_at_ts,
                writer_identity: {
                    let wi = writer_identity.value(i);
                    if wi.is_empty() {
                        None
                    } else {
                        Some(wi.to_string())
                    }
                },
            });
        }
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_engine_register_and_query() {
        let engine = QueryEngine::new();
        let records = vec![
            FileRecord {
                content_hash: "hash1".to_string(),
                hierarchy: "A".to_string(),
                level: "L0".to_string(),
                plane: "as-is".to_string(),
                payload_kind: "mp3".to_string(),
                payload_format: "mp3".to_string(),
                payload_size_bytes: 100,
                physical_location: "s3://bucket/hash1".to_string(),
                correlation_ids: vec![("k1".to_string(), "v1".to_string())],
                lineage_refs: vec![],
                routing_tags: vec![],
                written_at: chrono::Utc::now(),
                writer_identity: None,
            },
            FileRecord {
                content_hash: "hash2".to_string(),
                hierarchy: "B".to_string(),
                level: "L1".to_string(),
                plane: "as-is".to_string(),
                payload_kind: "mp3".to_string(),
                payload_format: "mp3".to_string(),
                payload_size_bytes: 200,
                physical_location: "s3://bucket/hash2".to_string(),
                correlation_ids: vec![],
                lineage_refs: vec!["parent1".to_string()],
                routing_tags: vec![("singer".to_string(), "Adele".to_string())],
                written_at: chrono::Utc::now(),
                writer_identity: Some("writer1".to_string()),
            },
        ];

        engine.register_files(&records).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(engine.filter("payload_kind = 'mp3'")).unwrap();
        assert_eq!(result.len(), 2);

        let result = rt.block_on(engine.filter("hierarchy = 'A'")).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content_hash, "hash1");
    }
}
