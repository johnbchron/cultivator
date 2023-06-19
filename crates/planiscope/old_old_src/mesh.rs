use crate::coords::Transform;

use bevy_render::mesh::Mesh as BevyMesh;
use fidget::eval::{Family, Tape};
use fidget::mesh::{Mesh as FidgetMesh, Octree, Settings};

#[derive(Clone)]
pub struct FullMesh {
  pub vertices: Vec<glam::Vec3A>,
  pub triangles: Vec<glam::UVec3>,
  pub normals: Vec<glam::Vec3A>,
}

impl FullMesh {
  pub fn mesh_new<T: Family>(tape: &Tape<T>, depth: u8) -> Self {
    let settings = Settings {
      threads: 6,
      min_depth: depth,
      max_depth: 0,
    };

    let octree = Octree::build::<T>(tape, settings);
    let fidget_mesh = octree.walk_dual(settings);
    // let mesh = Self::prune_mesh();

    let vertices = fidget_mesh
      .vertices
      .iter()
      .map(|v| glam::Vec3A::new(v.x as f32, v.y as f32, v.z as f32))
      .collect();
    let triangles = fidget_mesh
      .triangles
      .iter()
      .map(|t| glam::UVec3::new(t[0] as u32, t[1] as u32, t[2] as u32))
      .collect();
    let normals = implicit_normals(&fidget_mesh, tape);

    FullMesh {
      vertices,
      triangles,
      normals,
    }
  }

  pub fn transform(&mut self, transform: &Transform) {
    self.vertices.iter_mut().for_each(|v| {
      *v = v.mul_add(transform.scale, transform.position);
    });
  }

  pub fn prune(&mut self) {
    // prune triangles outside of the -1 to 1 range on any axis
    const MESH_BLEED: [f32; 3] = [1.0, 1.0, 1.0];
    let violating_verts = self
      .vertices
      .iter()
      .enumerate()
      .filter(|(_, v)| v.abs().cmpgt(MESH_BLEED.into()).any())
      .map(|(i, _)| i)
      .collect::<Vec<usize>>();

    self.triangles.retain(|t| {
      violating_verts
        .iter()
        .all(|i| !t.to_array().iter().any(|x| *x == (*i as u32)))
    });
  }
}

impl From<FullMesh> for BevyMesh {
  fn from(mesh: FullMesh) -> Self {
    let mut bevy_mesh =
      BevyMesh::new(bevy_render::mesh::PrimitiveTopology::TriangleList);
    bevy_mesh.insert_attribute(
      BevyMesh::ATTRIBUTE_POSITION,
      mesh
        .vertices
        .into_iter()
        .map(|v| Into::<[f32; 3]>::into(v))
        .collect::<Vec<_>>(),
    );
    bevy_mesh.insert_attribute(
      BevyMesh::ATTRIBUTE_NORMAL,
      mesh
        .normals
        .into_iter()
        .map(|v| Into::<[f32; 3]>::into(v))
        .collect::<Vec<_>>(),
    );
    bevy_mesh.set_indices(Some(bevy_render::mesh::Indices::U32(
      mesh
        .triangles
        .into_iter()
        .map(|v| [v.x, v.y, v.z])
        .flatten()
        .collect(),
    )));
    bevy_mesh
  }
}

pub fn implicit_normals<T: Family>(
  mesh: &FidgetMesh,
  tape: &Tape<T>,
) -> Vec<glam::Vec3A> {
  let eval = tape.new_grad_slice_evaluator();
  let mut normals: Vec<glam::Vec3A> = vec![];

  for vertex in mesh.vertices.iter() {
    let grad = eval.eval(&[vertex.x], &[vertex.y], &[vertex.z], &[]);
    match grad {
      Err(_) => normals.push(glam::Vec3A::ZERO),
      Ok(grad) => {
        let normal = glam::Vec3A::new(grad[0].dx, grad[0].dy, grad[0].dz);
        normals.push(normal);
      }
    }
  }
  normals
}
