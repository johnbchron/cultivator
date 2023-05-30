
use fidget::eval::{Family, Tape};
use fidget::mesh::{Mesh, Octree, Settings};
use fidget::rhai::eval;
use fidget::jit;

pub struct MeshWrapper {
	pub mesh: Mesh,
	pub normals: Vec<[f32; 3]>,
}

pub fn mesh_from_surface(expr: &str, depth: u8) -> MeshWrapper {
  let (surface, ctx) = eval(expr).unwrap();
  let tape = ctx.get_tape::<jit::Eval>(surface).unwrap();

  let settings = Settings {
    threads: 8,
    min_depth: depth,
    max_depth: 0,
  };
  let octree = Octree::build::<jit::Eval>(&tape, settings);
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