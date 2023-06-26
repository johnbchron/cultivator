use std::f32::consts::*;

use bevy::{
  core_pipeline::tonemapping::Tonemapping,
  pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
  prelude::*,
};
use bevy_pixel_cam::{PixelCamBundle, PixelCamPlugin, PixelCamSettings};

fn main() {
  App::new()
    .insert_resource(AmbientLight {
      color:      Color::WHITE,
      brightness: 1.0 / 5.0f32,
    })
    .insert_resource(DirectionalLightShadowMap { size: 4096 })
    .add_plugins(DefaultPlugins)
    .add_plugin(PixelCamPlugin)
    .add_startup_system(setup)
    .add_system(animate_light_direction)
    .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn((
    Camera3dBundle {
      transform: Transform::from_xyz(0.7, 0.7, 1.0)
        .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
      tonemapping: Tonemapping::None,
      ..default()
    },
    EnvironmentMapLight {
      diffuse_map:  asset_server
        .load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
      specular_map: asset_server
        .load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
    },
    PixelCamBundle {
      settings: PixelCamSettings::new(4.0, 0.0, 0.0),
      ..default()
    },
  ));

  commands.spawn(DirectionalLightBundle {
    directional_light: DirectionalLight {
      shadows_enabled: true,
      ..default()
    },
    // This is a relatively small scene, so use tighter shadow
    // cascade bounds than the default for better quality.
    // We also adjusted the shadow map to be larger since we're
    // only using a single cascade.
    cascade_shadow_config: CascadeShadowConfigBuilder {
      num_cascades: 1,
      maximum_distance: 1.6,
      ..default()
    }
    .into(),
    ..default()
  });
  commands.spawn(bevy::prelude::SceneBundle {
    scene: asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"),
    ..default()
  });
}

fn animate_light_direction(
  time: Res<Time>,
  mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
  for mut transform in &mut query {
    transform.rotation = Quat::from_euler(
      EulerRot::ZYX,
      0.0,
      time.elapsed_seconds() * PI / 5.0,
      -FRAC_PI_4,
    );
  }
}
