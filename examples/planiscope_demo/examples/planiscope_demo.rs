use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_pixel_cam::PixelCamBundle;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::{f32::consts::*, ops::Deref};
use timing::start;

use planiscope::{shape::{Shape, ShapeDef, ShapeOp, UnaryOp}, comp::Composition};

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

  let mut composition = Composition::new();
  // add a bunch of spheres in random positions with the rand crate
  for _ in 0..20 {
    let x = rand::random::<f32>() * 10.0 - 5.0;
    let y = rand::random::<f32>() * 10.0 - 5.0;
    let z = rand::random::<f32>() * 10.0 - 5.0;
    let radius = rand::random::<f32>() * 0.5 + 0.5;
    let r: u8 = rand::random::<u8>();
    let g: u8 = rand::random::<u8>();
    let b: u8 = rand::random::<u8>();
    composition.add_shape(
      Shape::ShapeOp(
        ShapeOp::UnaryOp(
          UnaryOp::Recolor { rgb: [r, g, b] },
          Box::new(Shape::ShapeDef(
            ShapeDef::SpherePrimitive { radius: radius }
          ))
        )
      ),
      [x, y, z],
    );
  }
  let mut context = fidget::Context::new();
  let timer = start();
  let node = composition.compile_solid(&mut context, &planiscope::comp::RenderSettings { min_voxel_size: 0.1 });
  println!("compile_solid took {}ms", timer.elapsed().as_millis());
  println!("context is {} bytes in memory", context.size_of());
  let timer = start();
  let tape: fidget::eval::Tape<fidget::vm::Eval> = context.get_tape(node).unwrap();
  println!("get_tape took {}ms", timer.elapsed().as_millis());
  println!("tape has {} nodes", tape.len());
  let timer = start();
  let interval_eval = tape.new_interval_evaluator();
  let (_, simplify) = interval_eval.eval(
      [-1.0, 1.0], // X
      [-1.0, 1.0], // Y
      [-1.0, 1.0], // Z
      &[]         // variables (unused)
  ).unwrap();
  let simplify = simplify.unwrap().simplify().unwrap();
  println!("simplification took {}ms", timer.elapsed().as_millis());
  println!("simplified tape has {} nodes", simplify.len());
  
  
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
