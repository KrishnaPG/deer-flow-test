//! Per-frame systems for procedural generator entities.

use bevy::log::trace;
use bevy::prelude::{Query, Res, Time, Transform};

use super::cloud_layer::{self, CloudParticle};
use super::drop_pods::{self, DropPod};
use super::path_travellers::{self, Traveller};
use super::river_barges::{self, Barge};
use crate::constants::descent::CLOUD_RADIUS;

pub fn barge_system(time: Res<Time>, mut query: Query<(&mut Barge, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut barge, mut tf) in query.iter_mut() {
        barge.t = (barge.t + barge.speed * dt * 0.01) % 1.0;
        tf.translation = river_barges::barge_position(barge.t, barge.index, barge.river_radius);
        trace!("barge_system: index={} t={:.4}", barge.index, barge.t);
    }
}

pub fn traveller_system(time: Res<Time>, mut query: Query<(&mut Traveller, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut traveller, mut tf) in query.iter_mut() {
        traveller.t = (traveller.t + traveller.speed * dt * 0.01) % 1.0;
        tf.translation = path_travellers::traveller_position(
            traveller.t,
            traveller.index,
            traveller.path_radius,
        );
        trace!(
            "traveller_system: index={} t={:.4}",
            traveller.index,
            traveller.t
        );
    }
}

pub fn cloud_system(time: Res<Time>, mut query: Query<(&mut CloudParticle, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut cloud, mut transform) in query.iter_mut() {
        cloud.t = (cloud.t + cloud.speed * dt * 0.01) % 1.0;
        transform.translation = cloud_layer::cloud_position(cloud.t, cloud.index, CLOUD_RADIUS);
        trace!("cloud_system: index={} t={:.4}", cloud.index, cloud.t);
    }
}

pub fn drop_pod_system(time: Res<Time>, mut query: Query<(&mut DropPod, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut pod, mut transform) in query.iter_mut() {
        pod.t = (pod.t + pod.speed * dt * 0.01) % 1.0;
        transform.translation = drop_pods::pod_position(pod.t, pod.index);
        trace!("drop_pod_system: index={} t={:.4}", pod.index, pod.t);
    }
}
