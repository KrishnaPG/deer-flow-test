use deer_runtime_world_projection::macro_micro_label;

#[test]
fn task_world_objects_keep_macro_micro_continuity() {
    assert_eq!(macro_micro_label("task"), "macro_micro_consistent");
}

#[test]
fn non_task_world_objects_fall_back_to_generic_macro_micro_label() {
    assert_eq!(macro_micro_label("artifact"), "macro_micro_generic");
}
