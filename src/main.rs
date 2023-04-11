
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use hexx::*;
// use hexx::shapes;

const HEX_SIZE: Vec2 = Vec2::splat(1.0);

#[derive(Component)]
struct Player;

#[derive(Component)]
#[allow(dead_code)]
enum GridItem {
	Dirt,
	Grass,
	Stone,
	Cobblestone,
}

#[derive(Component)]
struct GridPosition(Hex);

fn build_hex_mesh(hex_layout: &HexLayout, height: f32) -> Mesh {
	let mesh_info = MeshInfo::partial_hexagonal_column(hex_layout, Hex::ZERO, height);
  let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
  mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices.to_vec());
  mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals.to_vec());
  mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs.to_vec());
  mesh.set_indices(Some(Indices::U16(mesh_info.indices)));
  mesh
}

fn 


fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.run();
}
