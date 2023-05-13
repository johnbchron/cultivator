mod constants;
mod hex;

use constants::*;
use hex::item::HexItem;
use hex::position::HexPosition;

use bevy::{
  app::{App, PluginGroup},
  asset::{AddAsset, AssetServer, Assets, Handle},
  core_pipeline::{
    core_3d::Camera3dBundle,
    tonemapping::{DebandDither, Tonemapping},
  },
  diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
    LogDiagnosticsPlugin,
  },
  ecs::{
    query::With,
    system::{Commands, Query, Res, Resource},
    world::{FromWorld, World},
  },
  input::{keyboard::KeyCode, Input},
  math::{Quat, Vec3},
  pbr::{DirectionalLightBundle, PointLight, PointLightBundle},
  prelude::{MaterialMeshBundle, MaterialPlugin, StandardMaterial},
  render::{
    camera::{Camera, Projection},
    mesh::Mesh,
    texture::ImagePlugin,
    view::{ColorGrading, Msaa},
  },
  time::Time,
  transform::components::Transform,
  utils::default,
  window::{Window, WindowPlugin},
  DefaultPlugins,
};
use bevy_diagnostic_vertex_count::{
  VertexCountDiagnosticsPlugin, VertexCountDiagnosticsSettings,
};
use bevy_pixel_cam::{
  material::{PixelMaterial},
  post_process::{PixelCamPlugin, PixelCamSettings, PixelCamBundle},
};

use hexx::*;

use std::collections::HashMap;
// for enum to vec
use strum::IntoEnumIterator;

#[derive(Resource)]
struct HexMaterials(HashMap<HexItem, Handle<StandardMaterial>>);

#[derive(Resource)]
struct HexMeshes(HashMap<HexItem, Handle<Mesh>>);

impl FromWorld for HexMaterials {
  fn from_world(world: &mut World) -> Self {
    let mut map = HashMap::new();
    for item in HexItem::iter() {
      let asset_server = world.get_resource::<AssetServer>().unwrap();
      let material = item.build_material(asset_server);
      // get the material asset resource or add it if it doesn't exist
      let mut materials =
        world.get_resource_mut::<Assets<StandardMaterial>>().unwrap();

      map.insert(item, materials.add(material));
    }
    Self(map)
  }
}

impl FromWorld for HexMeshes {
  fn from_world(world: &mut World) -> Self {
    let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
    let mut map = HashMap::new();
    for item in HexItem::iter() {
      map.insert(item, meshes.add(item.build_mesh()));
    }
    Self(map)
  }
}

fn build_hex_bundle(
  grid_item: &HexItem,
  hex: &Hex,
  hex_materials: &HexMaterials,
  hex_meshes: &HexMeshes,
) -> (MaterialMeshBundle<StandardMaterial>, HexPosition, HexItem) {
  let hex_layout = HexLayout {
    hex_size: HEX_SIZE,
    ..Default::default()
  };
  let position = hex_layout.hex_to_world_pos(*hex);
  let material = hex_materials.0.get(grid_item).unwrap();
  let mesh = hex_meshes.0.get(grid_item).unwrap();
  (
    MaterialMeshBundle {
      mesh: mesh.clone(),
      material: material.clone(),
      transform: Transform::from_translation(Vec3::new(
        position.x, 0.0, position.y,
      ))
      .looking_to(Vec3::Z, Vec3::Y),
      ..Default::default()
    },
    HexPosition {
      pos: *hex,
      ..Default::default()
    },
    *grid_item,
  )
}

fn build_test_grid(
  mut commands: Commands,
  hex_materials: Res<HexMaterials>,
  hex_meshes: Res<HexMeshes>,
) {
  let n = 20;
  let mut grid = HashMap::new();
  for q in -n..(n + 1) {
    let r1 = (-n - q).max(-n);
    let r2 = (n - q).min(n);
    for r in r1..r2 {
      let hex = Hex::new(q, r);
      let item = HexItem::random();
      grid.insert(hex, item);
    }
  }

  for (hex, item) in grid.iter() {
    commands.spawn(build_hex_bundle(item, hex, &hex_materials, &hex_meshes));
  }
}

fn setup_graphics(mut commands: Commands) {
  // spawn lighting
  commands.spawn(DirectionalLightBundle {
    // rotate 45 degrees around the x axis and 45 degrees around the z axis
    transform: Transform::from_rotation(Quat::from_rotation_x(-0.5 * std::f32::consts::PI))
      * Transform::from_rotation(Quat::from_rotation_z(-0.25 * std::f32::consts::PI)),
    ..default()
  });
  commands.spawn(PointLightBundle {
    transform: Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),
    point_light: PointLight {
      intensity: 2400.0,
      range: 100.0,
      ..Default::default()
    },
    ..default()
  });

  let isometric_rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_8);

  commands.spawn((
    Camera3dBundle {
      transform: Transform::from_translation(Vec3::new(0.0, 20.0, 20.0))
        .with_rotation(isometric_rotation)
        .with_translation(Vec3::new(
          0.0,
          10.0 + HEX_HEIGHT,
          10.0,
        )),
      // projection: Projection::Orthographic(OrthographicProjection {
      //   scaling_mode: ScalingMode::WindowSize(UNIT_TO_PIXEL),
      //   ..Default::default()
      // }),
      tonemapping: Tonemapping::default(),
      dither: DebandDither::Disabled,
      color_grading: ColorGrading::default(),
      ..Default::default()
    },
    PixelCamBundle::default(),
  ));
}

fn handle_camera_movement(
  mut query: Query<(&mut Transform, &mut Projection, &mut PixelCamSettings), With<Camera>>,
  time: Res<Time>,
  keyboard_input: Res<Input<KeyCode>>,
) {
  for (mut transform, projection, mut pixel_cam_settings) in query.iter_mut() {
    let mut delta = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::A) {
      delta.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::D) {
      delta.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::W) {
      delta.z -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::S) {
      delta.z += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Up) {
      pixel_cam_settings.new_pixel_size -= 0.1;
    } else if keyboard_input.pressed(KeyCode::Down) {
      pixel_cam_settings.new_pixel_size += 0.1;
    }
    if keyboard_input.pressed(KeyCode::Q) || keyboard_input.pressed(KeyCode::E)
    {
      let projection = projection.into_inner();

      if keyboard_input.pressed(KeyCode::Q) {
        match projection {
          Projection::Orthographic(ref mut ortho) => {
            ortho.scale += ZOOM_SPEED * time.delta_seconds();
          }
          _ => {
            unimplemented!()
          }
        }
      }
      if keyboard_input.pressed(KeyCode::E) {
        match projection {
          Projection::Orthographic(ref mut ortho) => {
            ortho.scale -= ZOOM_SPEED * time.delta_seconds();
          }
          _ => {
            unimplemented!()
          }
        }
      }
    }
    if delta != Vec3::ZERO {
      delta = delta.normalize() * PAN_SPEED * time.delta_seconds();
      transform.translation += Vec3::new(delta.x, delta.y, delta.z);
    }
  }
}

fn main() {
  App::new()
    // default plugins with no vsync
    .add_plugins(
      DefaultPlugins
        .set(WindowPlugin {
          primary_window: Some(Window {
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..Default::default()
          }),
          ..Default::default()
        })
        .set(ImagePlugin::default_nearest()),
    )
    // graphics config
    .insert_resource(Msaa::Off)
    .add_plugin(PixelCamPlugin)
    // .add_plugin(PixelMaterialPlugin)
    .add_plugin(MaterialPlugin::<PixelMaterial>::default())
    // diagnostic config
    .add_plugin(LogDiagnosticsPlugin::default())
    .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .add_plugin(EntityCountDiagnosticsPlugin::default())
    .insert_resource(VertexCountDiagnosticsSettings { only_visible: true })
    .add_plugin(VertexCountDiagnosticsPlugin::default())
    // prebuild meshes and materials
    .add_asset::<PixelMaterial>()
    .init_resource::<HexMaterials>()
    .init_resource::<HexMeshes>()
    // setup graphics
    .add_startup_system(setup_graphics)
    // spawn game objects
    .add_startup_system(build_test_grid)
    // handle input
    .add_system(handle_camera_movement)
    // maintainers
    .run();
}
