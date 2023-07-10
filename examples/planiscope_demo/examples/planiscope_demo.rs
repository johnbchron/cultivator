// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::f32::consts::*;

use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_pixel_cam::PixelCamBundle;
use planiscope::{
  builder::*,
  comp::{CompilationSettings, Composition},
  mesh::FullMesh,
};
use timing::start;

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
    PixelCamBundle {
      settings: bevy_pixel_cam::PixelCamSettings::new(12.0, 15.0, 0.05),
      ..default()
    },
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

  let mut composition = Composition::new();
  // add a bunch of spheres in random positions with the rand crate
  for _ in 0..200 {
    let x = rand::random::<f32>() * 10.0 - 5.0;
    let y = rand::random::<f32>() * 10.0 - 5.0;
    let z = rand::random::<f32>() * 10.0 - 5.0;
    let radius = rand::random::<f32>();
    let r: u8 = rand::random::<u8>();
    let g: u8 = rand::random::<u8>();
    let b: u8 = rand::random::<u8>();
    composition.add_shape(recolor(sphere(radius), r, g, b), [x, y, z]);
  }
  let mut ctx = fidget::Context::new();
  let compilation_settings = CompilationSettings {
    min_voxel_size: 0.01,
  };
  let solid_root_node =
    composition.compile_solid(&mut ctx, &compilation_settings);
  let color_root_node =
    composition.compile_color(&mut ctx, &compilation_settings);

  let solid_root_node = planiscope::csg::csg_normalize_region(
    solid_root_node,
    [0.0, 0.0, 0.0],
    [5.0, 5.0, 5.0],
    &mut ctx,
  );
  let color_root_node = planiscope::csg::csg_normalize_region(
    color_root_node,
    [0.0, 0.0, 0.0],
    [5.0, 5.0, 5.0],
    &mut ctx,
  );

  let solid_tape: fidget::eval::Tape<fidget::vm::Eval> =
    ctx.get_tape(solid_root_node).unwrap();
  let color_tape: fidget::eval::Tape<fidget::vm::Eval> =
    ctx.get_tape(color_root_node).unwrap();

  println!("building mesh...");
  let start = start();
  let mut full_mesh: FullMesh = FullMesh::mesh_new(&solid_tape, &color_tape, 7);
  println!("mesh has {} vertices", full_mesh.vertices.len());
  full_mesh.prune();
  full_mesh.denormalize([0.0, 0.0, 0.0].into(), [5.0, 5.0, 5.0].into());
  let mesh = Mesh::from(full_mesh);
  println!("built mesh in {} ms", start.elapsed().as_millis());

  let mesh_handle = meshes.add(mesh);
  commands.spawn(PbrBundle {
    mesh: mesh_handle,
    material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
    ..Default::default()
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

fn main() {
  App::new()
    .insert_resource(AmbientLight {
      color:      Color::WHITE,
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
