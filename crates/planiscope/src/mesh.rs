use core::cmp::{min, max};
use bevy_render::mesh::Mesh as BevyMesh;
use fidget::{
  eval::{Family, Tape},
  mesh::{Mesh as FidgetMesh, Octree, Settings},
};

#[derive(Clone)]
pub struct FullMesh {
  pub vertices:  Vec<glam::Vec3A>,
  pub triangles: Vec<glam::UVec3>,
  pub normals:   Vec<glam::Vec3A>,
  pub colors:    Vec<glam::Vec4>,
}

impl FullMesh {
  pub fn mesh_new<T: Family>(
    solid_tape: &Tape<T>,
    color_tape: &Tape<T>,
    depth: u8,
  ) -> Self {
    let settings = Settings {
      threads:   6,
      min_depth: depth,
      max_depth: 0,
    };

    let octree = Octree::build::<T>(solid_tape, settings);
    let fidget_mesh = octree.walk_dual(settings);

    let vertices = fidget_mesh
      .vertices
      .iter()
      .map(|v| glam::Vec3A::new(v.x, v.y, v.z))
      .collect();
    let triangles = fidget_mesh
      .triangles
      .iter()
      .map(|t| glam::UVec3::new(t[0] as u32, t[1] as u32, t[2] as u32))
      .collect();
    let normals = implicit_normals(&fidget_mesh, solid_tape);
    let colors = implicit_colors(&fidget_mesh, color_tape);

    FullMesh {
      vertices,
      triangles,
      normals,
      colors,
    }
  }

  pub fn denormalize(&mut self, pos: glam::Vec3A, size: glam::Vec3A) {
    self.vertices.iter_mut().for_each(|v| {
      *v = v.mul_add(size, pos);
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
        .map(Into::<[f32; 3]>::into)
        .collect::<Vec<_>>(),
    );
    bevy_mesh.insert_attribute(
      BevyMesh::ATTRIBUTE_NORMAL,
      mesh
        .normals
        .into_iter()
        .map(Into::<[f32; 3]>::into)
        .collect::<Vec<_>>(),
    );
    bevy_mesh.insert_attribute(
      BevyMesh::ATTRIBUTE_COLOR,
      mesh
        .colors
        .into_iter()
        .map(Into::<[f32; 4]>::into)
        .collect::<Vec<_>>(),
    );
    bevy_mesh.set_indices(Some(bevy_render::mesh::Indices::U32(
      mesh
        .triangles
        .into_iter()
        .flat_map(|v| [v.x, v.y, v.z])
        .collect(),
    )));
    bevy_mesh
  }
}

// TODO: refactor this to actually use bulk evaluators
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

// TODO: refactor this to actually use bulk evaluators
pub fn implicit_colors<T: Family>(
  mesh: &FidgetMesh,
  tape: &Tape<T>,
) -> Vec<glam::Vec4> {
  let eval = tape.new_float_slice_evaluator();
  let mut colors: Vec<glam::Vec4> = vec![];

  for vertex in mesh.vertices.iter() {
    let color = eval.eval(&[vertex.x], &[vertex.y], &[vertex.z], &[]);
    match color {
      Err(_) => panic!("color evaluation failed"),
      Ok(color) => {
        // code used to store the color as an f32:
        // ```
        // let rgb = (rgb[0] as u32) << 16 | (rgb[1] as u32) << 8 | (rgb[2] as u32);
        // let rgb = rgb as f64 / 16_777_215.0;
        // ```
        
        let normalized_color = color[0];
        let rgb: u32 = (normalized_color * 16_777_215.0).round().try_into().unwrap();
        let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
        let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
        let b = (rgb & 0xFF) as f32 / 255.0;
        let color = glam::Vec4::new(r, g, b, 1.0);
        colors.push(color);
      }
    }
  }
  colors
}
