use deer_pipeline_derivations::derive_thread_header_vm;
use insta::assert_yaml_snapshot;

#[test]
fn derives_thread_header_vm_with_stream_and_thread_state() {
    let vm = derive_thread_header_vm("thread_1", "Survey the ridge", "live");

    assert_yaml_snapshot!(vm, @r#"
thread_id: thread_1
title: Survey the ridge
stream_state: live
"#);
}
