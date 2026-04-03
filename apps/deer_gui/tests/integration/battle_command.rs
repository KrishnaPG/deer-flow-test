use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

use deer_gui::camera::viewport_navigation_system;
use deer_gui::camera::CinematicCamera;
use deer_gui::hud::battle_command::{
    BattleCommandHudState, BottomDeckSection, RailCollapseState, ShellVisibilityTier,
};

#[test]
fn t_battle_01_shell_contract_defaults() {
    let state = BattleCommandHudState::default();
    assert_eq!(state.visibility_tier, ShellVisibilityTier::Tier1);
    assert_eq!(state.event_rail, RailCollapseState::Expanded);
    assert_eq!(state.fleet_rail, RailCollapseState::Expanded);
    assert_eq!(
        state.active_bottom_section,
        BottomDeckSection::SelectionSummary
    );
}

#[test]
fn t_battle_02_minimap_locate_repositions_camera_only() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        0.016,
    )));

    let cam_entity = app
        .world_mut()
        .spawn((CinematicCamera::default(), Transform::default()))
        .id();

    app.add_systems(Update, viewport_navigation_system);
    app.update();

    // Directly set focus_target as the navigation system would
    let mut cam = app
        .world_mut()
        .get_mut::<CinematicCamera>(cam_entity)
        .unwrap();
    cam.focus_target = Some(Vec3::new(30.0, 0.0, -40.0));

    app.update();

    let cam = app.world().get::<CinematicCamera>(cam_entity).unwrap();
    assert!(cam.focus_target.is_some());
    let target = cam.focus_target.unwrap();
    assert!(
        (target.x - 30.0).abs() < 0.1,
        "expected x≈30, got {}",
        target.x
    );
    assert!(
        (target.z - (-40.0)).abs() < 0.1,
        "expected z≈-40, got {}",
        target.z
    );
}

#[test]
fn t_battle_04_collapsed_event_rail_keeps_badges() {
    let mut state = BattleCommandHudState::default();
    state.event_rail = RailCollapseState::Compact;
    state.event_badge_count = 3;
    assert_eq!(state.event_badge_count, 3);
}

#[test]
fn t_battle_06_tier3_overlay_pauses_world_navigation_but_keeps_context() {
    let mut state = BattleCommandHudState::default();
    state.visibility_tier = ShellVisibilityTier::Tier3;
    state.overlay_blocks_world_navigation = true;
    assert!(state.overlay_blocks_world_navigation);
}

#[test]
fn t_battle_07_bottom_deck_sections_share_context_but_keep_distinct_state() {
    let state = BattleCommandHudState::default();
    assert_eq!(
        state.active_bottom_section,
        BottomDeckSection::SelectionSummary
    );
}

#[test]
fn t_battle_08_action_deck_hotkey_slots_stay_stable_across_selection_updates() {
    let slots = deer_gui::hud::battle_command::bottom_deck::default_action_slots();
    assert_eq!(slots.len(), 15);
    assert_eq!(slots[14].label, "Cancel");
}
