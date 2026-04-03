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
