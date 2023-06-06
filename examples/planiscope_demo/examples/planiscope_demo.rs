use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_pixel_cam::PixelCamBundle;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::f32::consts::*;
use timing::start;

use planiscope::{core::{build_tree, build_chunk, Template}, coords::LodCoords};

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  commands.spawn((
    Camera3dBundle {
      transform: Transform::from_xyz(0.0, 0.0, 25.0)
        .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
      ..default()
    },
    // PixelCamBundle {
    //   settings: bevy_pixel_cam::PixelCamSettings::new(
    //     24.0,
    //     1.0,
    //     0.05
    //   ), 
    //   ..default()
    // },
    FlyCamera {
      sensitivity: 5.0,
      ..default()
    },
  ));
  commands.spawn(DirectionalLightBundle {
    directional_light: DirectionalLight {
      shadows_enabled: true,
      ..default()
    },
    ..default()
  });

  // insert a meshed object from planiscope
  let expr = "sqrt(square(x)+square(y)+square(z)) - 20".to_string();
  let template = Template {
    source: expr,
    volume_size: 100.0,
    local_chunk_detail: 7,
    neighbor_count: 1,
    chunk_mesh_bleed: 1.1,
    targets: vec![LodCoords::new_from_world([0.0, 50.0, 0.0].into(), 5, 100.0)],
  };

  info!("meshing tree with expression \"{}\"", &template.source);
  let timer = start();
  let coords = build_tree(&template);
  let chunks = coords
    .into_iter()
    .map(|coords| build_chunk(&template, coords))
    .collect::<Vec<_>>();
  info!("meshed tree in {}ms", timer.stop().as_millis());
  info!("generated {} chunks", chunks.iter().count());
  info!(
    "tree has {} vertices",
    chunks
      .iter()
      .map(|v| v.1.vertices.len())
      .sum::<usize>()
  );

  for chunk in chunks.iter() {
    if chunk.1.vertices.is_empty() {
      continue;
    }

    let mesh_handle = meshes.add(chunk.1.clone().into());
    commands.spawn(PbrBundle {
      mesh: mesh_handle,
      material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
      // transform: bevy_transform,
      ..Default::default()
    });
  }
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

fn main() {
  App::new()
    .insert_resource(AmbientLight {
      color: Color::WHITE,
      brightness: 1.0 / 5.0f32,
    })
    .add_plugins(DefaultPlugins)
    // .add_plugin(WorldInspectorPlugin::new())
    .add_plugin(FlyCameraPlugin)
    .add_plugin(bevy_pixel_cam::PixelCamPlugin)
    .add_startup_system(setup)
    .add_system(animate_light_direction)
    .run();
}
