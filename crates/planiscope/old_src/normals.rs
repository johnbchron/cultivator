use fidget::eval::{Family, Tape};
use fidget::mesh::Mesh;

#[allow(dead_code)]
pub fn smooth_vertex_normals(mesh: &Mesh) -> Vec<[f32; 3]> {
  let mut normals = vec![[0.0; 3]; mesh.vertices.len()];
  for face in mesh.triangles.iter() {
    let a = mesh.vertices[face[0]];
    let b = mesh.vertices[face[1]];
    let c = mesh.vertices[face[2]];
    let normal = (b - a).cross(&(c - a));
    for i in 0..3 {
      normals[face[i]] = [
        normals[face[i]][0] + normal[0],
        normals[face[i]][1] + normal[1],
        normals[face[i]][2] + normal[2],
      ];
    }
  }
  normals
}

#[allow(dead_code)]
pub fn implicit_normals<T: Family>(
  mesh: &Mesh,
  tape: &Tape<T>,
) -> Vec<[f32; 3]> {
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
