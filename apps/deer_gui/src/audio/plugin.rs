//! [`DeerAudioPlugin`] — registers audio resources, messages, and systems.
//!
//! Processes [`AudioCommand`] messages each frame using Bevy's built-in
//! audio system (`bevy_audio`). Handles ambient loops, one-shot sounds,
//! and volume/mute control.

use bevy::audio::{AudioPlayer, PlaybackSettings};
use bevy::log::{debug, info, trace};
use bevy::prelude::*;

use super::events::{AudioCommand, UiSound};
use super::manager::AudioManager;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers the audio subsystem.
///
/// * Inserts an [`AudioManager`] resource.
/// * Registers the [`AudioCommand`] message type.
/// * Adds the command processing system to `Update`.
pub struct DeerAudioPlugin;

impl Plugin for DeerAudioPlugin {
    fn build(&self, app: &mut App) {
        info!("DeerAudioPlugin::build — registering audio resources and systems");

        app.init_resource::<AudioManager>()
            .add_message::<AudioCommand>()
            .add_systems(Update, audio_command_system);
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Processes [`AudioCommand`] messages using Bevy's built-in audio.
fn audio_command_system(
    mut commands: Commands,
    mut cmd_reader: MessageReader<AudioCommand>,
    mut manager: ResMut<AudioManager>,
    asset_server: Option<Res<AssetServer>>,
) {
    // Gracefully no-op when AssetPlugin is absent (e.g. headless / test).
    let Some(asset_server) = asset_server else {
        return;
    };

    for cmd in cmd_reader.read() {
        trace!("audio_command_system — processing {:?}", cmd);
        match cmd {
            AudioCommand::PlayAmbient {
                track,
                fade_in_secs,
            } => {
                handle_play_ambient(
                    &mut commands,
                    &mut manager,
                    &asset_server,
                    track,
                    *fade_in_secs,
                );
            }
            AudioCommand::StopAmbient { fade_out_secs: _ } => {
                handle_stop_ambient(&mut commands, &mut manager);
            }
            AudioCommand::PlayOneShot { sound } => {
                handle_play_one_shot(&mut commands, &manager, &asset_server, *sound);
            }
            AudioCommand::SetMasterVolume { volume } => {
                manager.set_volume(*volume);
            }
            AudioCommand::Mute => {
                manager.mute();
            }
            AudioCommand::Unmute => {
                manager.unmute();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

/// Starts an ambient loop track.
fn handle_play_ambient(
    commands: &mut Commands,
    manager: &mut AudioManager,
    asset_server: &AssetServer,
    track: &str,
    _fade_in_secs: f32,
) {
    // Stop existing ambient if any
    if let Some(entity) = manager.ambient_entity.take() {
        debug!("handle_play_ambient — despawning old ambient entity");
        commands.entity(entity).despawn();
    }

    let volume = manager.effective_volume();
    let handle: Handle<AudioSource> = asset_server.load(track.to_owned());

    let entity = commands
        .spawn((
            AudioPlayer::new(handle),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::Linear(volume),
                ..default()
            },
        ))
        .id();

    manager.current_ambient = Some(track.to_string());
    manager.ambient_entity = Some(entity);

    info!(
        "handle_play_ambient — playing '{}' at volume {}",
        track, volume
    );
}

/// Stops the current ambient loop.
fn handle_stop_ambient(commands: &mut Commands, manager: &mut AudioManager) {
    if let Some(entity) = manager.ambient_entity.take() {
        debug!("handle_stop_ambient — despawning ambient entity");
        commands.entity(entity).despawn();
        manager.current_ambient = None;
    } else {
        debug!("handle_stop_ambient — no ambient playing");
    }
}

/// Plays a one-shot UI sound effect.
fn handle_play_one_shot(
    commands: &mut Commands,
    manager: &AudioManager,
    asset_server: &AssetServer,
    sound: UiSound,
) {
    let path = sound.asset_path();
    let volume = manager.effective_volume();

    if volume <= 0.0 {
        trace!("handle_play_one_shot — muted, skipping {:?}", sound);
        return;
    }

    let handle: Handle<AudioSource> = asset_server.load(path);

    commands.spawn((
        AudioPlayer::new(handle),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::Linear(volume),
            ..default()
        },
    ));

    debug!("handle_play_one_shot — {:?} at volume {}", sound, volume);
}
