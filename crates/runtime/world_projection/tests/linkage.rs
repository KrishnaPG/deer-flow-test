use deer_runtime_world_projection::WorldObject;

#[test]
fn world_objects_preserve_level_plane_and_reopen_metadata() {
    let object = WorldObject::task_beacon("task_1", "task_detail");

    assert_eq!(object.source_record_id, "task_1");
    assert_eq!(object.drill_down_target, "task_detail");
    assert_eq!(object.level, "L2");
    assert_eq!(object.plane, "AsIs");
}
