use fidget::context::{Context, Node};

use crate::{LodCoord, Template};

pub struct Transform {
  pub position: [f32; 3],
  pub scale: [f32; 3],
}

impl Transform {
  pub fn from_lodcoords_to_map(
    coords: LodCoord,
    template: &Template,
  ) -> Transform {
    let float_coords = coords.float_coords();
    let float_size: f32 = coords.float_size() * template.bleed;
    let scale = [float_size, float_size, float_size];
    let position = [
      (float_coords[0] + float_size / 2.0) * 2.0 - 1.0,
      (float_coords[1] + float_size / 2.0) * 2.0 - 1.0,
      (float_coords[2] + float_size / 2.0) * 2.0 - 1.0,
    ];
    Transform { position, scale }
  }

  pub fn from_lodcoords_to_world(
    coords: LodCoord,
    template: &Template,
  ) -> Transform {
    let mut transform = Transform::from_lodcoords_to_map(coords, template);
    transform.scale[0] *= template.volume_size;
    transform.scale[1] *= template.volume_size;
    transform.scale[2] *= template.volume_size;
    transform.position[0] *= template.volume_size;
    transform.position[1] *= template.volume_size;
    transform.position[2] *= template.volume_size;
    transform
  }

  pub fn invert(&self) -> Transform {
    let inv_scale = [
      1.0 / self.scale[0],
      1.0 / self.scale[1],
      1.0 / self.scale[2],
    ];
    let inv_position = [
      -self.position[0] / self.scale[0],
      -self.position[1] / self.scale[1],
      -self.position[2] / self.scale[2],
    ];
    Transform {
      position: inv_position,
      scale: inv_scale,
    }
  }

  pub fn transform_context(&self, root: Node, ctx: &mut Context) -> Node {
    let (x, y, z) = (ctx.x(), ctx.y(), ctx.z());
    let scaled_x = ctx.mul(x, self.scale[0]).unwrap();
    let scaled_y = ctx.mul(y, self.scale[1]).unwrap();
    let scaled_z = ctx.mul(z, self.scale[2]).unwrap();
    let translated_x = ctx.add(scaled_x, self.position[0]).unwrap();
    let translated_y = ctx.add(scaled_y, self.position[1]).unwrap();
    let translated_z = ctx.add(scaled_z, self.position[2]).unwrap();
    ctx
      .remap_xyz(root, [translated_x, translated_y, translated_z])
      .unwrap()
  }

  pub fn transform_point(&self, point: [f32; 3]) -> [f32; 3] {
    [
      point[0] * self.scale[0] + self.position[0],
      point[1] * self.scale[1] + self.position[1],
      point[2] * self.scale[2] + self.position[2],
    ]
  }

  pub fn transform_normal(&self, normal: [f32; 3]) -> [f32; 3] {
    [
      normal[0] / self.scale[0],
      normal[1] / self.scale[1],
      normal[2] / self.scale[2],
    ]
  }
}
