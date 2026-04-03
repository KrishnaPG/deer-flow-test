use deer_design::run_spatial_projection_proof;
use insta::assert_yaml_snapshot;

#[test]
fn proves_spatial_projection_selection_and_artifact_drill_down() {
    let proof = run_spatial_projection_proof();

    assert_yaml_snapshot!(proof, @r#"
mode: spatial_analysis
world_objects:
  - WorldTaskBeacon
  - WorldArtifactUnlock
selection:
  source_record_id: artifact_1
camera_sync: bidirectional
drill_down_target: artifact_detail
scene_anchor_count: 2
telemetry_selected_marker: artifact_1
minimap_viewport_id: minimap_panel
"#);
}
