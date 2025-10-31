use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_4, PI};

/// Animate directional light rotation over time
pub fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut tf in &mut query {
        tf.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_secs() * PI / 100.0,
            -FRAC_PI_4,
        );
    }
}
