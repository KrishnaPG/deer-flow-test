use super::layout_presets::FIRST_PLAYABLE_PRESET;
use super::panel_catalog::FIRST_PLAYABLE_PANELS;
use super::shell_mode::{ShellBrokers, FIRST_PLAYABLE_BROKERS};
use super::view_hosts::{ViewHostBinding, FIRST_PLAYABLE_VIEW_HOSTS};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstPlayableShell {
    pub mode: &'static str,
    pub panels: &'static [&'static str],
    pub brokers: ShellBrokers,
    pub view_hosts: &'static [ViewHostBinding],
}

pub fn build_first_playable_shell() -> FirstPlayableShell {
    FirstPlayableShell {
        mode: FIRST_PLAYABLE_PRESET,
        panels: &FIRST_PLAYABLE_PANELS,
        brokers: FIRST_PLAYABLE_BROKERS,
        view_hosts: &FIRST_PLAYABLE_VIEW_HOSTS,
    }
}
