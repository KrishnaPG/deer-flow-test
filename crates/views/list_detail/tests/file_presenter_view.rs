use deer_view_list_detail::{render_file_presenter_view, FilePresenterInput};
use insta::assert_yaml_snapshot;

#[test]
fn file_presenter_view_distinguishes_preview_from_pointer_access() {
    let preview = render_file_presenter_view(&FilePresenterInput::Preview {
        mime: "image/png".into(),
    });
    let pointer = render_file_presenter_view(
        &FilePresenterInput::mediated_pointer(
            "application/pdf".into(),
            "mediated://artifact/2".into(),
        )
        .unwrap(),
    );

    assert_yaml_snapshot!(preview, @r#"
mode: preview
mime: image/png
access_action: inline_preview
href: ~
"#);

    assert_yaml_snapshot!(pointer, @r#"
mode: pointer
mime: application/pdf
access_action: open_pointer
href: "mediated://artifact/2"
"#);
}

#[test]
fn mediated_pointer_input_rejects_unmediated_locations() {
    assert!(FilePresenterInput::mediated_pointer(
        "application/pdf".into(),
        "/tmp/direct.pdf".into(),
    )
    .is_err());
    assert!(FilePresenterInput::mediated_pointer(
        "application/pdf".into(),
        "https://example.com/direct.pdf".into(),
    )
    .is_err());
}
