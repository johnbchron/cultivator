use nalgebra as na;
use tessellation::BoundingBox;
use tessellation::Mesh as TessMesh;
use bevy_render::mesh::Mesh as BevyMesh;

pub struct NoiseTessellator {
  pub noise: Box<dyn noise::NoiseFn<f64, 3>>,
  pub b_box: BoundingBox<f64>,
}

impl tessellation::ImplicitFunction<f64> for NoiseTessellator {
  fn bbox(&self) -> &BoundingBox<f64> {
    &self.b_box
  }
  fn value(&self, p: &na::Point3<f64>) -> f64 {
    na::Vector3::new(p.x, p.y, p.z).magnitude() - 1.0
    // (self.noise.get([p.x, p.y, p.z]) - 0.5) * 2.0
  }
  fn normal(&self, p: &na::Point3<f64>) -> na::Vector3<f64> {
    let dx = 0.001;
    let dy = 0.001;
    let dz = 0.001;
    let x = self.value(&na::Point3::new(p.x + dx, p.y, p.z))
      - self.value(&na::Point3::new(p.x - dx, p.y, p.z));
    let y = self.value(&na::Point3::new(p.x, p.y + dy, p.z))
      - self.value(&na::Point3::new(p.x, p.y - dy, p.z));
    let z = self.value(&na::Point3::new(p.x, p.y, p.z + dz))
      - self.value(&na::Point3::new(p.x, p.y, p.z - dz));
    na::Vector3::new(x, y, z).normalize()
    // return na::Vector3::new(p.x, p.y, p.z).normalize();
  }
}

pub fn noise_tessellate(
  noise_tessellator: NoiseTessellator,
  resolution: f32,
  relative_error: f32,
) -> Option<TessMesh<f64>> {
  let mut mdc = tessellation::ManifoldDualContouring::new(
    &noise_tessellator,
    resolution.into(),
    relative_error.into(),
  );
  let mesh = mdc.tessellate();
  mesh
}

pub fn sample_tessellate() -> TessMesh<f64> {
  let noise: noise::Perlin = noise::Perlin::new(1_u32);
  let noise_tessellator = NoiseTessellator {
    noise: Box::new(noise),
    b_box: BoundingBox {
      min: na::Point3::new(-1.0, -1.0, -1.0),
      max: na::Point3::new(1.0, 1.0, 1.0),
    },
  };
  let mesh = noise_tessellate(noise_tessellator,  0.02, 0.01).unwrap();
  mesh
}

pub fn to_bevy_mesh(input: TessMesh<f64>) -> BevyMesh {
	let mut mesh = BevyMesh::new(bevy_render::mesh::PrimitiveTopology::TriangleList);
	mesh.insert_attribute(
		BevyMesh::ATTRIBUTE_POSITION,
		input
			.vertices
			.iter()
			.map(|v| [v[0] as f32, v[1] as f32, v[2] as f32])
			.collect::<Vec<[f32; 3]>>(),
	);
	mesh.set_indices(Some(
		bevy_render::mesh::Indices::U32(
			input
				.faces
				// flatten the array of arrays
				.iter()
				.flatten()
				.map(|i| *i as u32)
				.collect::<Vec<u32>>(),
		),
	));
  
  mesh.duplicate_vertices();
  mesh.compute_flat_normals();
  
	mesh
}