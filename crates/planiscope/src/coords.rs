
use fidget::context::{Context, Node};

#[derive(Debug, Clone)]
pub struct LodCoords {
  pos: glam::UVec3,
  lod: u8,
}

impl LodCoords {
  pub fn new(pos: glam::UVec3, lod: u8) -> Self {
    LodCoords { pos, lod }
  }
  
  /// The center of the region, on a [-1, 1] scale.
  pub fn float_center_coords(&self) -> glam::Vec3A {
    let pos = self.pos.as_vec3a();
    let scale = self.float_size();
    (pos * scale + scale * 0.5) * 2.0 - 1.0
  }
  
  /// The size of the region, on a [0, 1] scale.
  pub fn float_size(&self) -> glam::Vec3A {
    let scale = 1.0 / (1 << self.lod) as f32;
    glam::Vec3A::splat(scale)
  }
  
  /// Build a new `LodCoords` from a float position and a lod.
  pub fn new_from_float(pos: glam::Vec3A, lod: u8) -> Self {
    let pos = (pos + 1.0) /  2.0;
    spatialtree::OctVec::from_float_coords(pos.into(), lod).into()
  }

  /// Build a new `LodCoords` from a world position and a lod.
  pub fn new_from_world(pos: glam::Vec3A, lod: u8, world_size: f32) -> Self {
  	// divide world_size by 2 because it's side length of the cube, not a radius
  	let pos = pos / (world_size / 2.0);
  	// // divide by the max of the max component and 1, to get a value in [-1, 1]
  	// pos /= pos.max(glam::Vec3A::splat(1.0));

  	Self::new_from_float(pos, lod)
  }
}

impl From<LodCoords> for spatialtree::OctVec {
  fn from(v: LodCoords) -> Self {
    spatialtree::OctVec::new(
      [v.pos.x as u8, v.pos.y as u8, v.pos.z as u8],
      v.lod,
    )
  }
}

impl From<spatialtree::OctVec> for LodCoords {
  fn from(v: spatialtree::OctVec) -> Self {
    LodCoords {
      pos: glam::UVec3::new(v.pos[0] as u32, v.pos[1] as u32, v.pos[2] as u32),
      lod: v.depth,
    }
  }
}


pub struct Transform {
  pub position: glam::Vec3A,
  pub scale: glam::Vec3A,
}

impl From<LodCoords> for Transform {
  fn from(v: LodCoords) -> Self {
    Transform {
      position: v.float_center_coords(),
      scale: v.float_size(),
    }
  }
}

pub fn transform_context(
  root: Node,
  ctx: &mut Context,
  transform: &Transform,
) -> Node {
  let (x, y, z) = (ctx.x(), ctx.y(), ctx.z());
  let scaled_x = ctx.mul(x, transform.scale.x).unwrap();
  let scaled_y = ctx.mul(y, transform.scale.y).unwrap();
  let scaled_z = ctx.mul(z, transform.scale.z).unwrap();
  let translated_x = ctx.add(scaled_x, transform.position.x).unwrap();
  let translated_y = ctx.add(scaled_y, transform.position.y).unwrap();
  let translated_z = ctx.add(scaled_z, transform.position.z).unwrap();
  ctx
    .remap_xyz(root, [translated_x, translated_y, translated_z])
    .unwrap()
}
