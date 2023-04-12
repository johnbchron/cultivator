use bevy::{
  diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
    LogDiagnosticsPlugin,
  },
  prelude::*,
  render::{
    camera::ScalingMode, mesh::Indices, render_resource::PrimitiveTopology,
  },
};
use bevy_diagnostic_vertex_count::{
  VertexCountDiagnosticsPlugin, VertexCountDiagnosticsSettings,
};
use hexx::*;
use rand::Rng;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const HEX_SIZE: Vec2 = Vec2::splat(1.0);
const HEX_HEIGHT: f32 = 1.0;
const PAN_SPEED: f32 = 50.0;
const ZOOM_SPEED: f32 = 1.0;
const UNIT_TO_PIXEL: f32 = 50.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
#[allow(dead_code)]
#[derive(EnumIter, Eq, Hash, PartialEq, Clone, Copy)]
enum GridItem {
  Dirt,
  Grass,
  Stone,
  Cobblestone,
}

impl GridItem {
  fn random() -> Self {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..4) {
      0 => Self::Dirt,
      1 => Self::Grass,
      2 => Self::Stone,
      3 => Self::Cobblestone,
      _ => unreachable!(),
    }
  }
  fn hex_height(&self) -> f32 {
    match self {
      Self::Dirt => HEX_HEIGHT,
      Self::Grass => HEX_HEIGHT + 0.2,
      Self::Stone => HEX_HEIGHT + 0.3,
      Self::Cobblestone => HEX_HEIGHT + 0.4,
    }
  }
  fn base_color(&self) -> Color {
    match self {
      Self::Dirt => Color::rgb(0.5, 0.3, 0.1),
      Self::Grass => Color::rgb(0.0, 0.5, 0.0),
      // Self::Grass => Color::rgb(1.0, 1.0, 1.0),
      Self::Stone => Color::rgb(0.5, 0.5, 0.5),
      Self::Cobblestone => Color::rgb(0.3, 0.3, 0.3),
    }
  }
  fn load_texture(&self, asset_server: &AssetServer) -> Option<Handle<Image>> {
    match self {
      Self::Dirt => None,
      // Self::Grass => Some(asset_server.load("textures/grass.png")),
      Self::Grass => None,
      Self::Stone => None,
      Self::Cobblestone => None,
    }
  }

  fn build_material(&self, asset_server: &AssetServer) -> StandardMaterial {
    StandardMaterial {
      base_color: self.base_color(),
      base_color_texture: match self.load_texture(asset_server) {
        Some(texture) => Some(texture),
        None => StandardMaterial::default().base_color_texture,
      },
      metallic: match self {
        Self::Dirt => 0.0,
        Self::Grass => 0.0,
        Self::Stone => 0.5,
        Self::Cobblestone => 0.5,
      },
      perceptual_roughness: match self {
        Self::Dirt => 0.8,
        Self::Grass => 0.8,
        Self::Stone => 0.4,
        Self::Cobblestone => 0.5,
      },
      // reflectance: ??,
      ..Default::default()
    }
  }
}

#[derive(Component)]
struct GridPosition(Hex);

#[derive(Resource)]
struct HexMaterials(HashMap<GridItem, Handle<StandardMaterial>>);

#[derive(Resource)]
struct HexMeshes(HashMap<GridItem, Handle<Mesh>>);

#[derive(Resource)]
struct RenderSettings {
  frac_resolution: f32,
}

impl FromWorld for HexMaterials {
  fn from_world(world: &mut World) -> Self {
    let mut map = HashMap::new();
    for item in GridItem::iter() {
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
    for item in GridItem::iter() {
      map.insert(item, meshes.add(build_hex_mesh(item.hex_height())));
    }
    Self(map)
  }
}

impl Default for RenderSettings {
  fn default() -> Self {
    Self {
      frac_resolution: 0.125,
    }
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
  grid_item: &GridItem,
  hex: &Hex,
  hex_materials: &HexMaterials,
  hex_meshes: &HexMeshes,
) -> (PbrBundle, GridPosition, GridItem) {
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
    GridPosition(*hex),
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
      let item = GridItem::random();
      grid.insert(hex, item);
    }
  }

  for (hex, item) in grid.iter() {
    commands.spawn(build_hex_bundle(item, hex, &hex_materials, &hex_meshes));
  }
}

fn setup_graphics(mut commands: Commands) {
  commands.spawn(Camera3dBundle {
    transform: Transform::from_translation(Vec3::new(0.0, 20.0, 20.0))
      .looking_at(Vec3::ZERO, Vec3::Y)
      .with_translation(Vec3::new(
        0.0,
        500.0 + 20.0 + HEX_HEIGHT,
        500.0 + 20.0,
      )),
    projection: Projection::Orthographic(OrthographicProjection {
      scale: 1.0,
      scaling_mode: ScalingMode::WindowSize(UNIT_TO_PIXEL),
      ..Default::default()
    }),
    ..Default::default()
  });
  // spawn light
  commands.spawn(DirectionalLightBundle {
    transform: Transform::from_translation(Vec3::new(0.0, 100.0, 0.0))
      .looking_to(Vec3::NEG_Y, Vec3::Z),
    ..default()
  });
}

// // edit the camera viewport to maintain fractional pixel resolution
// fn maintain_fractional_camera_resolution(
// 	mut query: Query<&mut Camera>,
// 	render_settings: Res<RenderSettings>,
// ) {
// 	for mut camera in query.iter_mut() {
// 		let mut viewport = camera.get_viewport();
// 		viewport.scale = render_settings.frac_resolution;
// 		camera.set_viewport(viewport);
// 	}
// }

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
            // *projection = Projection::Orthographic(ortho);
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
    .add_plugins(DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {
      	present_mode: bevy::window::PresentMode::AutoNoVsync,
				..Default::default()
			}),
			..Default::default()
    }))
    // graphics config
    .insert_resource(Msaa::Sample2)
    .insert_resource(RenderSettings::default())
    // diagnostic config
    .add_plugin(LogDiagnosticsPlugin::default())
    .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .add_plugin(EntityCountDiagnosticsPlugin::default())
    .insert_resource(VertexCountDiagnosticsSettings { only_visible: true })
    .add_plugin(VertexCountDiagnosticsPlugin::default())
    // prebuild meshes and materials
    .init_resource::<HexMaterials>()
    .init_resource::<HexMeshes>()
    // prebuild game logic config
    .add_startup_system(setup_graphics)
    // spawn game objects
    .add_startup_system(build_test_grid)
    // handle input
    .add_system(handle_camera_movement)
    .run();
}
