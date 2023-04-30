use crate::constants::*;

use hexx::*;
use bevy_pixel_cam::material::PixelMaterial;

use bevy::asset::AssetServer;
use bevy::asset::Handle;
use bevy::ecs::component::Component;
use bevy::render::color::Color;
use bevy::render::mesh::Indices;
use bevy::render::mesh::Mesh;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::texture::Image;

use rand::Rng;
// for enum to vec
use strum_macros::EnumIter;

#[derive(Component)]
#[allow(dead_code)]
#[derive(EnumIter, Eq, Hash, PartialEq, Clone, Copy)]
pub enum HexItem {
  Dirt,
  Grass,
  Stone,
  Cobblestone,
}

impl HexItem {
  pub fn random() -> Self {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..4) {
      0 => Self::Dirt,
      1 => Self::Grass,
      2 => Self::Stone,
      3 => Self::Cobblestone,
      _ => unreachable!(),
    }
  }
  pub fn hex_height(&self) -> f32 {
    match self {
      Self::Dirt => HEX_HEIGHT,
      Self::Grass => HEX_HEIGHT + 0.2,
      Self::Stone => HEX_HEIGHT + 0.3,
      Self::Cobblestone => HEX_HEIGHT + 0.4,
    }
  }
  pub fn base_color(&self) -> Color {
    match self {
      Self::Dirt => Color::rgb(0.5, 0.3, 0.1),
      Self::Grass => Color::rgb(0.0, 0.5, 0.0),
      // Self::Grass => Color::rgb(1.0, 1.0, 1.0),
      Self::Stone => Color::rgb(0.5, 0.5, 0.5),
      Self::Cobblestone => Color::rgb(0.3, 0.3, 0.3),
    }
  }
  pub fn load_texture(
    &self,
    _asset_server: &AssetServer,
  ) -> Option<Handle<Image>> {
    match self {
      Self::Dirt => None,
      // Self::Grass => Some(asset_server.load("textures/grass.png")),
      Self::Grass => None,
      Self::Stone => None,
      Self::Cobblestone => None,
    }
  }

  pub fn build_material(&self, asset_server: &AssetServer) -> PixelMaterial {
    PixelMaterial {
      color: self.base_color(),
      // base_color_texture: match self.load_texture(asset_server) {
      //   Some(texture) => Some(texture),
      //   None => StandardMaterial::default().base_color_texture,
      // },
      // metallic: match self {
      //   Self::Dirt => 0.0,
      //   Self::Grass => 0.0,
      //   Self::Stone => 0.5,
      //   Self::Cobblestone => 0.5,
      // },
      // perceptual_roughness: match self {
      //   Self::Dirt => 0.8,
      //   Self::Grass => 0.8,
      //   Self::Stone => 0.4,
      //   Self::Cobblestone => 0.5,
      // },
      // reflectance: ??,
      ..Default::default()
    }
  }

  pub fn build_mesh(&self) -> Mesh {
    match self {
      Self::Dirt => build_base_mesh(self.hex_height()),
      Self::Grass => build_base_mesh(self.hex_height()),
      Self::Stone => build_base_mesh(self.hex_height()),
      Self::Cobblestone => build_base_mesh(self.hex_height()),
    }
  }
}

fn build_base_mesh(height: f32) -> Mesh {
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
