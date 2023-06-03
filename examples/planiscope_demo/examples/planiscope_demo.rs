use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use std::f32::consts::*;
use timing::start;

use planiscope::{build_from_template, LodCoord, Template};
// use bevy_pixel_cam::{PixelCamBundle, PixelCamPlugin};

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  commands.spawn((
    Camera3dBundle {
      transform: Transform::from_xyz(3.5, 3.5, 5.0)
        .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
      ..default()
    },
    // PixelCamBundle {
    //   settings: bevy_pixel_cam::PixelCamSettings::new(
    //     32.0,
    //     0.0,
    //     0.05,
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
  let expr = "square(x) + square(y) + square(z) - 0.5 + (sin(x*20.0) + sin(y*20.0) + sin(z*20.0)) / 50.0".to_string();
  let template = Template {
    volume_expr: expr,
    volume_size: 100.0,
    local_detail: 5,
    neighbors: 1,
    bleed: 1.1,
    targets: vec![LodCoord::from_float_coords([0.505, 0.6, 0.505], 5)],
  };

  info!("meshing tree with expression \"{}\"", &template.volume_expr);
  let timer = start();
  let mut tree = build_from_template(template);
  info!("meshed tree in {}ms", timer.stop().as_millis());
  info!("tree has {} chunks", tree.iter_chunks().count());
  info!(
    "tree has {} vertices",
    tree
      .iter_chunks()
      .map(|(_, chunk_c)| chunk_c.chunk.full_mesh.mesh.vertices.len())
      .sum::<usize>()
  );

  for (_, chunk_container) in tree.iter_chunks() {
    let chunk = &chunk_container.chunk;

    if chunk.full_mesh.mesh.vertices.is_empty() {
      continue;
    }

    let mesh_handle = meshes.add(chunk.full_mesh.clone().into());
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
    // .add_plugin(PixelCamPlugin)
    .add_plugin(FlyCameraPlugin)
    .add_startup_system(setup)
    .add_system(animate_light_direction)
    .run();
}
