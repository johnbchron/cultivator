use bevy_render::mesh::Mesh as BevyMesh;
use fidget::{
  eval::{Family, Tape},
  mesh::{Mesh as FidgetMesh, Octree, Settings},
};

#[derive(Clone)]
pub struct FullMesh {
  pub vertices:  Vec<glam::Vec3A>,
  pub triangles: Vec<glam::UVec3>,
  pub normals:   Option<Vec<glam::Vec3A>>,
  pub colors:    Option<Vec<glam::Vec4>>,
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

    println!("building octree");
    let octree = Octree::build::<T>(solid_tape, settings);
    let fidget_mesh = octree.walk_dual(settings);
    println!("octree built");

    println!("transforming vertices");
    let vertices = fidget_mesh
      .vertices
      .iter()
      .map(|v| glam::Vec3A::new(v.x, v.y, v.z))
      .collect();
    println!("vertices transformed");

    println!("transforming triangles");
    let triangles = fidget_mesh
      .triangles
      .iter()
      .map(|t| glam::UVec3::new(t[0] as u32, t[1] as u32, t[2] as u32))
      .collect();
    println!("triangles transformed");
    println!("calculating normals from surface");
    let normals = implicit_normals(&fidget_mesh, solid_tape);
    println!("normals calculated");
    println!("calculating colors from surface");
    let colors = implicit_colors(&fidget_mesh, color_tape);
    println!("colors calculated");

    FullMesh {
      vertices,
      triangles,
      normals: Some(normals),
      // normals: None,
      colors: Some(colors),
      // colors: None,
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
    if let Some(normals) = mesh.normals {
      bevy_mesh.insert_attribute(
        BevyMesh::ATTRIBUTE_NORMAL,
        normals
          .into_iter()
          .map(Into::<[f32; 3]>::into)
          .collect::<Vec<_>>(),
      );
    } else {
      bevy_mesh.compute_flat_normals();
    }
    if let Some(colors) = mesh.colors {
      bevy_mesh.insert_attribute(
        BevyMesh::ATTRIBUTE_COLOR,
        colors
          .iter()
          .map(|c| [c.x, c.y, c.z, c.w])
          .collect::<Vec<_>>(),
      );
    }
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
  let grad = eval.eval(
    &mesh.vertices.iter().map(|v| v.x).collect::<Vec<_>>(),
    &mesh.vertices.iter().map(|v| v.y).collect::<Vec<_>>(),
    &mesh.vertices.iter().map(|v| v.z).collect::<Vec<_>>(),
    &[],
  );
  match grad {
    Err(_) => panic!("normal evaluation failed"),
    Ok(grad) => {
      for g in grad {
        normals.push(glam::Vec3A::new(g.dx, g.dy, g.dz));
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

  let grad = eval.eval(
    &mesh.vertices.iter().map(|v| v.x).collect::<Vec<_>>(),
    &mesh.vertices.iter().map(|v| v.y).collect::<Vec<_>>(),
    &mesh.vertices.iter().map(|v| v.z).collect::<Vec<_>>(),
    &[],
  );

  match grad {
    Err(_) => panic!("color evaluation failed"),
    Ok(grad) => {
      for g in grad {
        colors.push(transform_implicit_color(g));
      }
    }
  }

  colors
}

fn transform_implicit_color(val: f32) -> glam::Vec4 {
  // we offset the hue by a bit when it gets set to avoid sampling red when
  // sampling noise
  if val < 0.1 {
    return glam::Vec4::new(1.0, 1.0, 1.0, 1.0);
  }

  // put it back in the normal range
  let val = (val - 0.1) / 0.9;

  let val = val * (256_u32.pow(3)) as f32;
  // bit shift to get the original values
  let red = ((val as u32) >> 16) as f32;
  let green = (((val as u32) << 16) >> 24) as f32;
  let blue = (((val as u32) << 24) >> 24) as f32;

  glam::Vec4::new(red / 255.0, green / 255.0, blue / 255.0, 1.0)
}
