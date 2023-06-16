//! Standard library of primitives.
//! 
//! This module contains the standard library of primitives. These are the
//! building blocks for most other shapes.

use super::super::{def::ShapeDef, BuildSettings, FieldType, Shape};

use fidget::context::Node;
use fidget::Context;

/// A rectangular prism.
#[derive(Debug)]
pub struct RectPrismPrimitive {
  pub size: [f32; 3],
}

impl Shape for RectPrismPrimitive {
  fn solid(&self, ctx: &mut Context, settings: &BuildSettings) -> Node {
    let [x, y, z] = self.size;
    if x < settings.min_voxel_size
      && y < settings.min_voxel_size
      && z < settings.min_voxel_size
    {
      return ctx.constant(1.0.into());
    }

    let s_x = ctx.constant(x.into());
    let s_y = ctx.constant(y.into());
    let s_z = ctx.constant(z.into());
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let x = ctx.sub(x, s_x).unwrap();
    let y = ctx.sub(y, s_y).unwrap();
    let z = ctx.sub(z, s_z).unwrap();
    let x = ctx.abs(x).unwrap();
    let y = ctx.abs(y).unwrap();
    let z = ctx.abs(z).unwrap();
    let max_xy = ctx.max(x, y).unwrap();
    let max_xyz = ctx.max(max_xy, z).unwrap();
    return max_xyz;
  }

  fn build(
    &self,
    ctx: &mut Context,
    _field: FieldType,
    settings: &BuildSettings,
  ) -> Node {
    self.solid(ctx, settings)
  }
}

impl ShapeDef for RectPrismPrimitive {
  fn semantic_id(&self) -> u128 {
    todo!()
  }
}

impl From<&CubePrimitive> for RectPrismPrimitive {
  fn from(cube: &CubePrimitive) -> Self {
    Self {
      size: [cube.size; 3],
    }
  }
}

/// A cube primitive.
#[derive(Debug)]
pub struct CubePrimitive {
  pub size: f32,
}

impl Shape for CubePrimitive {
  fn solid(&self, ctx: &mut Context, settings: &BuildSettings) -> Node {
    RectPrismPrimitive::from(self).solid(ctx, settings)
  }

  fn build(
    &self,
    ctx: &mut Context,
    _field: FieldType,
    settings: &BuildSettings,
  ) -> Node {
    self.solid(ctx, settings)
  }
}

impl ShapeDef for CubePrimitive {
  fn semantic_id(&self) -> u128 {
    todo!()
  }
}

/// A sphere primitive.
#[derive(Debug)]
pub struct SpherePrimitive {
  pub radius: f32,
}

impl Shape for SpherePrimitive {
  fn solid(&self, ctx: &mut Context, settings: &BuildSettings) -> Node {
    if self.radius * 2.0 < settings.min_voxel_size {
      return ctx.constant(1.0.into());
    }

    let r = ctx.constant(self.radius.into());
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let x_sq = ctx.square(x).unwrap();
    let y_sq = ctx.square(y).unwrap();
    let z_sq = ctx.square(z).unwrap();
    let r_sq = ctx.square(r).unwrap();
    let sum = ctx.add(x_sq, y_sq).unwrap();
    let sum = ctx.add(sum, z_sq).unwrap();
    let sub = ctx.sub(sum, r_sq).unwrap();
    return sub;
  }

  fn build(
    &self,
    ctx: &mut Context,
    _field: FieldType,
    settings: &BuildSettings,
  ) -> Node {
    self.solid(ctx, settings)
  }
}
