use deer_pipeline_derivations::artifacts::{ArtifactEntryVm, ArtifactShelfVm};
use deer_view_list_detail::render_artifact_shelf_view;
use insta::assert_yaml_snapshot;

#[test]
fn artifact_shelf_view_only_exposes_presented_mediated_entries() {
    let vm = ArtifactShelfVm {
        entries: vec![
            ArtifactEntryVm {
                artifact_id: "artifact_1".into(),
                title: "scan.png".into(),
                status: "presented".into(),
                preview_supported: true,
                retrieval_mode: "mediated_pointer",
                provenance: Some("sha256:scan".into()),
                presentation_state: "presented",
            },
            ArtifactEntryVm {
                artifact_id: "artifact_2".into(),
                title: "hidden.log".into(),
                status: "withheld".into(),
                preview_supported: true,
                retrieval_mode: "mediated_pointer",
                provenance: None,
                presentation_state: "withheld",
            },
            ArtifactEntryVm {
                artifact_id: "artifact_3".into(),
                title: "report.pdf".into(),
                status: "presented".into(),
                preview_supported: false,
                retrieval_mode: "mediated_pointer",
                provenance: None,
                presentation_state: "presented",
            },
        ],
    };

    let rendered = render_artifact_shelf_view(&vm);

    assert_yaml_snapshot!(rendered, @r#"
item_count: 2
items:
  - artifact_id: artifact_1
    title: scan.png
    preview_action: preview_artifact
    preview_available: true
    open_action: open_artifact
    download_action: download_artifact
    install_action: install_artifact
  - artifact_id: artifact_3
    title: report.pdf
    preview_action: ~
    preview_available: false
    open_action: open_artifact
    download_action: download_artifact
    install_action: install_artifact
"#);
}
