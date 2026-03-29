//! Scene-audio bridge — sends [`AudioCommand`] messages when the active scene changes.
//!
//! This module provides a **decoupled bridge** between scene activation and the
//! audio subsystem. The scene management system (PR-1) writes a desired ambient
//! track into [`SceneAudioState`]; the [`scene_audio_bridge_system`] detects the
//! change and emits the appropriate [`AudioCommand`] messages.
//!
//! No direct dependency on scene internals — only reads/writes a resource.

use bevy::log::{debug, info, trace};
use bevy::prelude::*;

use crate::audio::AudioCommand;
use crate::constants::timing::{SCENE_AUDIO_FADE_IN_SECS, SCENE_AUDIO_FADE_OUT_SECS};

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Tracks which ambient audio track the scene system wants playing.
///
/// Updated by the scene management system when scenes activate/deactivate.
/// The bridge system reads this and sends appropriate [`AudioCommand`] messages.
///
/// Track paths use `&'static str` because they originate from
/// [`SceneConfig::ambient_audio_track()`] which returns compile-time constants,
/// avoiding heap allocation on every scene switch.
#[derive(Resource, Debug, Default)]
pub struct SceneAudioState {
    /// The ambient track path that should currently be playing.
    /// `None` means no ambient should play.
    desired_track: Option<&'static str>,
    /// The track we last told the audio system to play.
    /// Used to detect changes without extra events.
    active_track: Option<&'static str>,
}

impl SceneAudioState {
    /// Request a specific ambient track to play.
    pub fn request_ambient(&mut self, track: &'static str) {
        info!("SceneAudioState::request_ambient — track='{track}'");
        self.desired_track = Some(track);
    }

    /// Request that ambient audio stop.
    pub fn request_stop(&mut self) {
        info!("SceneAudioState::request_stop");
        self.desired_track = None;
    }

    /// Returns the currently desired track (if any).
    pub fn desired_track(&self) -> Option<&str> {
        self.desired_track
    }

    /// Returns the currently active track (if any).
    pub fn active_track(&self) -> Option<&str> {
        self.active_track
    }

    /// Returns `true` if the desired track differs from the active track.
    pub fn has_pending_change(&self) -> bool {
        self.desired_track != self.active_track
    }

    /// Mark the current desired track as applied (now active).
    pub fn mark_applied(&mut self) {
        trace!(
            "SceneAudioState::mark_applied — active_track={:?}",
            self.desired_track
        );
        self.active_track = self.desired_track;
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Watches [`SceneAudioState`] for changes and sends [`AudioCommand`] messages.
///
/// When the desired track diverges from the active track the system will:
/// 1. Send [`AudioCommand::StopAmbient`] if an ambient track was playing.
/// 2. Send [`AudioCommand::PlayAmbient`] if a new track is requested.
///
/// Fade durations are sourced from [`crate::constants::timing`].
pub fn scene_audio_bridge_system(
    mut state: ResMut<SceneAudioState>,
    mut writer: MessageWriter<AudioCommand>,
) {
    if !state.has_pending_change() {
        return;
    }

    // Stop current ambient if one is playing.
    if state.active_track.is_some() {
        debug!("scene_audio_bridge: stopping current ambient");
        writer.write(AudioCommand::StopAmbient {
            fade_out_secs: SCENE_AUDIO_FADE_OUT_SECS,
        });
    }

    // Start new ambient if one is desired.
    if let Some(track) = state.desired_track {
        info!("scene_audio_bridge: starting ambient '{track}'");
        writer.write(AudioCommand::PlayAmbient {
            track: track.to_string(),
            fade_in_secs: SCENE_AUDIO_FADE_IN_SECS,
        });
    }

    state.mark_applied();
}
