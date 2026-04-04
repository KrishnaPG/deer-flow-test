pub fn mission_partition_tag(mission_id: &str) -> (String, String) {
    ("mission".into(), mission_id.into())
}
