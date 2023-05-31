
use fidget::eval::{Family, Tape};
use fidget::mesh::{Mesh, Octree, Settings};
use fidget::rhai::eval;
// use fidget::context::Node;
use fidget::vm;

pub struct MeshWrapper {
	pub mesh: Mesh,
	pub normals: Vec<[f32; 3]>,
}

// fn transform_viewport(root: Node, ctx: &mut fidget::context::Context, mat: [[f32; 4]; 4]) -> Node {
//   let (x, y, z) = (ctx.x(), ctx.y(), ctx.z());
//   let (x, y, z, w) = (
//     mat[0][0] * x + mat[0][1] * y + mat[0][2] * z + mat[0][3],
//     mat[1][0] * x + mat[1][1] * y + mat[1][2] * z + mat[1][3],
//     mat[2][0] * x + mat[2][1] * y + mat[2][2] * z + mat[2][3],
//     mat[3][0] * x + mat[3][1] * y + mat[3][2] * z + mat[3][3],
//   );
// }

pub fn mesh_from_surface(expr: &str, depth: u8) -> MeshWrapper {
  let (surface, ctx) = eval(expr).unwrap();
  let transformed_surface = surface;
  
  let tape = ctx.get_tape::<vm::Eval>(transformed_surface).unwrap();

  let settings = Settings {
    threads: 8,
    min_depth: depth,
    max_depth: 0,
  };
  let octree = Octree::build::<vm::Eval>(&tape, settings);
  let mesh = octree.walk_dual(settings);

  // let normals = smooth_vertex_normals(&mesh);
  let normals = implicit_normals(&mesh, tape);

  MeshWrapper {
    mesh,
    normals,
  }
}

#[allow(dead_code)]
fn smooth_vertex_normals(mesh: &Mesh) -> Vec<[f32; 3]> {
  let mut normals = vec![[0.0; 3]; mesh.vertices.len()];
  for face in mesh.triangles.iter() {
    let a = mesh.vertices[face[0] as usize];
    let b = mesh.vertices[face[1] as usize];
    let c = mesh.vertices[face[2] as usize];
    let normal = (b - a).cross(&(c - a));
    for i in 0..3 {
      normals[face[i] as usize] = [
        normals[face[i] as usize][0] + normal[0],
        normals[face[i] as usize][1] + normal[1],
        normals[face[i] as usize][2] + normal[2],
      ];
    }
  }
  normals
}

#[allow(dead_code)]
fn implicit_normals<T: Family>(mesh: &Mesh, tape: Tape<T>) -> Vec<[f32; 3]> {
  let eval = tape.new_grad_slice_evaluator();
  let mut normals: Vec<[f32; 3]> = vec![];

  for vertex in mesh.vertices.iter() {
    let grad = eval.eval(&[vertex.x], &[vertex.y], &[vertex.z], &[]);
    match grad {
      Err(_) => normals.push([0.0; 3]),
      Ok(grad) => {
        let normal = [grad[0].dx, grad[0].dy, grad[0].dz];
        normals.push(normal);
      }
    }
  }

  normals
}