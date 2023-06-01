use bevy_render::mesh::Mesh as BevyMesh;
use fidget::mesh::Mesh;
use rayon::prelude::*;

/// A Fidget mesh plus vertex normals.
pub struct FullMesh {
  pub mesh: Mesh,
  pub normals: Vec<[f32; 3]>,
}

impl Clone for FullMesh {
  fn clone(&self) -> Self {
    FullMesh {
      mesh: Mesh {
        vertices: self.mesh.vertices.clone(),
        triangles: self.mesh.triangles.clone(),
      },
      normals: self.normals.clone(),
    }
  }
}

impl From<FullMesh> for BevyMesh {
  fn from(fullmesh: FullMesh) -> Self {
    let mut mesh =
      BevyMesh::new(bevy_render::mesh::PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
      BevyMesh::ATTRIBUTE_POSITION,
      fullmesh
        .mesh
        .vertices
        .iter()
        .map(|v| [v.x, v.y, v.z])
        .collect::<Vec<[f32; 3]>>(),
    );
    mesh.insert_attribute(BevyMesh::ATTRIBUTE_NORMAL, fullmesh.normals);

    mesh.set_indices(Some(bevy_render::mesh::Indices::U32(
      fullmesh
        .mesh
        .triangles
        .iter()
        .flat_map(|t| vec![t.x, t.y, t.z])
        .map(|x| x as u32)
        .collect::<Vec<u32>>(),
    )));

    mesh
  }
}

pub fn prune_mesh(input: Mesh) -> Mesh {
  // prune triangles outside of the -1 to 1 range on any axis
  // to do this, we'll make a list of any violating vertices' indexes
  // and then remove any triangles that contain them
  let mut mesh = input;
  let violating_verts = mesh
    .vertices
    .par_iter()
    .enumerate()
    .filter(|(_, v)| v[0].abs() > 1.0 || v[1].abs() > 1.0 || v[2].abs() > 1.0)
    .map(|(i, _)| i)
    .collect::<Vec<usize>>();

  let new_triangles = mesh
    .triangles
    .par_iter()
    .filter(|tri| {
      !violating_verts
        .iter()
        .any(|&v| tri.data.0.iter().any(|&x| x.contains(&v)))
    })
    .map(|tri| *tri)
    .collect::<Vec<_>>();

  // if new_triangles.len() != mesh.triangles.len() {
  //   println!(
  //     "pruned {} triangles",
  //     mesh.triangles.len() - new_triangles.len()
  //   );
  // }
  mesh.triangles = new_triangles;
  mesh
}
