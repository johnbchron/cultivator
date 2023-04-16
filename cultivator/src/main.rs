mod constants;
mod hex;

use constants::*;
use hex::HexItem;

use bevy::{
  app::{App, PluginGroup},
  asset::{AssetServer, Assets, Handle},
  core_pipeline::core_3d::Camera3dBundle,
  diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
    LogDiagnosticsPlugin,
  },
  ecs::{
    component::Component,
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
    world::{FromWorld, World},
  },
  input::{keyboard::KeyCode, Input},
  math::{Quat, Vec3},
  pbr::{DirectionalLightBundle, PbrBundle, StandardMaterial},
  render::{
    camera::{Camera, OrthographicProjection, Projection, ScalingMode},
    mesh::{Indices, Mesh},
    render_resource::PrimitiveTopology,
    texture::{Image, ImagePlugin},
    view::Msaa,
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
use bevy_pixel_cam::{PixelCamPlugin, PixelCamSettings};

use hexx::*;

use std::collections::HashMap;
// for enum to vec
use strum::IntoEnumIterator;

#[derive(Component)]
struct HexPosition(Hex);

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
      let mut materials = world
        .get_resource_mut::<Assets<StandardMaterial>>()
        .unwrap();
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
      map.insert(item, meshes.add(build_hex_mesh(item.hex_height())));
    }
    Self(map)
  }
}

fn build_hex_mesh(height: f32) -> Mesh {
  let hex_layout = HexLayout {
    hex_size: HEX_SIZE,
    ..Default::default()
  };
  let mesh_info =
    MeshInfo::partial_hexagonal_column(&hex_layout, Hex::ZERO, height);
  let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
  mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices.to_vec());
  mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals.to_vec());
  mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs.to_vec());
  mesh.set_indices(Some(Indices::U16(mesh_info.indices)));
  mesh
}

fn build_hex_bundle(
  grid_item: &HexItem,
  hex: &Hex,
  hex_materials: &HexMaterials,
  hex_meshes: &HexMeshes,
) -> (PbrBundle, HexPosition, HexItem) {
  let hex_layout = HexLayout {
    hex_size: HEX_SIZE,
    ..Default::default()
  };
  let position = hex_layout.hex_to_world_pos(*hex);
  let material = hex_materials.0.get(grid_item).unwrap();
  let mesh = hex_meshes.0.get(grid_item).unwrap();
  (
    PbrBundle {
      mesh: mesh.clone(),
      material: material.clone(),
      transform: Transform::from_translation(Vec3::new(
        position.x, 0.0, position.y,
      ))
      .looking_to(Vec3::Z, Vec3::Y),
      ..Default::default()
    },
    HexPosition(*hex),
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

fn setup_graphics(
  mut commands: Commands,
  _windows: Query<&Window>,
  _images: ResMut<Assets<Image>>,
) {
  // spawn lighting
  commands.spawn(DirectionalLightBundle {
    transform: Transform::from_translation(Vec3::new(0.0, 100.0, 0.0))
      .looking_to(Vec3::NEG_Y, Vec3::Z),
    ..default()
  });

  let isometric_rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4);

  commands.spawn((
    Camera3dBundle {
      transform: Transform::from_translation(Vec3::new(0.0, 20.0, 20.0))
        .with_rotation(isometric_rotation)
        .with_translation(Vec3::new(
          0.0,
          500.0 + 20.0 + HEX_HEIGHT,
          500.0 + 20.0,
        )),
      projection: Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::WindowSize(UNIT_TO_PIXEL),
        ..Default::default()
      }),
      ..Default::default()
    },
    PixelCamSettings { intensity: 0.01 },
  ));
}

fn handle_camera_movement(
  mut query: Query<(&mut Transform, &mut Projection), With<Camera>>,
  time: Res<Time>,
  keyboard_input: Res<Input<KeyCode>>,
) {
  for (mut transform, projection) in query.iter_mut() {
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
    // diagnostic config
    .add_plugin(LogDiagnosticsPlugin::default())
    .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .add_plugin(EntityCountDiagnosticsPlugin::default())
    .insert_resource(VertexCountDiagnosticsSettings { only_visible: true })
    .add_plugin(VertexCountDiagnosticsPlugin::default())
    // prebuild meshes and materials
    .init_resource::<HexMaterials>()
    .init_resource::<HexMeshes>()
    // setup graphics
    .add_startup_system(setup_graphics)
    // spawn game objects
    .add_startup_system(build_test_grid)
    // handle input
    .add_system(handle_camera_movement)
    .run();
}
