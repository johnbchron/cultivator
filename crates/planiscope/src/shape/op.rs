use std::todo;

use super::{BuildSettings, FieldType, Shape};

use fidget::context::Node;
use fidget::Context;

/// A shape operation.
#[allow(dead_code)]
#[derive(Debug)]
pub enum ShapeOp {
  Union(Box<dyn Shape>, Box<dyn Shape>),
  Difference(Box<dyn Shape>, Box<dyn Shape>),
  Intersection(Box<dyn Shape>, Box<dyn Shape>),
  Replacement(Box<dyn Shape>, Box<dyn Shape>),

  Translate {
    shape: Box<dyn Shape>,
    pos: [f32; 3],
  },
  Scale {
    shape: Box<dyn Shape>,
    scale: [f32; 3],
  },
  MatrixTransform {
    shape: Box<dyn Shape>,
    matrix: [f32; 16],
  },

  Recolor {
    shape: Box<dyn Shape>,
    rgb: [u8; 3],
  },
  Abbreviate {
    shape: Box<dyn Shape>,
    threshold_voxel_size: f32,
  },
}

pub fn csg_union(a: Node, b: Node, ctx: &mut Context) -> Node {
  ctx.max(a, b).unwrap()
}

pub fn csg_difference(a: Node, b: Node, ctx: &mut Context) -> Node {
  let b = ctx.neg(b).unwrap();
  ctx.max(a, b).unwrap()
}

pub fn csg_intersection(a: Node, b: Node, ctx: &mut Context) -> Node {
  ctx.min(a, b).unwrap()
}

pub fn csg_replacement(a: Node, b: Node, ctx: &mut Context) -> Node {
  let neg_a = ctx.neg(a).unwrap();
  let b = ctx.min(b, neg_a).unwrap();
  ctx.min(a, b).unwrap()
}

pub fn csg_translate(shape: Node, pos: [f32; 3], ctx: &mut Context) -> Node {
  let x = ctx.x();
  let y = ctx.y();
  let z = ctx.z();
  let pos_x = ctx.constant(pos[0].into());
  let pos_y = ctx.constant(pos[1].into());
  let pos_z = ctx.constant(pos[2].into());
  let new_x = ctx.sub(x, pos_x).unwrap();
  let new_y = ctx.sub(y, pos_y).unwrap();
  let new_z = ctx.sub(z, pos_z).unwrap();
  ctx.remap_xyz(shape, [new_x, new_y, new_z]).unwrap()
}

pub fn csg_scale(shape: Node, scale: [f32; 3], ctx: &mut Context) -> Node {
  let x = ctx.x();
  let y = ctx.y();
  let z = ctx.z();
  let scale_x = ctx.constant(scale[0].into());
  let scale_y = ctx.constant(scale[1].into());
  let scale_z = ctx.constant(scale[2].into());
  let new_x = ctx.mul(x, scale_x).unwrap();
  let new_y = ctx.mul(y, scale_y).unwrap();
  let new_z = ctx.mul(z, scale_z).unwrap();
  ctx.remap_xyz(shape, [new_x, new_y, new_z]).unwrap()
}

impl Shape for ShapeOp {
  fn solid(&self, ctx: &mut Context, settings: &BuildSettings) -> Node {
    match self {
      ShapeOp::Union(a, b) => {
        let a = a.solid(ctx, settings);
        let b = b.solid(ctx, settings);
        csg_union(a, b, ctx)
      }
      ShapeOp::Difference(a, b) => {
        let a = a.solid(ctx, settings);
        let b = b.solid(ctx, settings);
        csg_difference(a, b, ctx)
      }
      ShapeOp::Intersection(a, b) => {
        let a = a.solid(ctx, settings);
        let b = b.solid(ctx, settings);
        csg_intersection(a, b, ctx)
      }
      ShapeOp::Replacement(a, b) => {
        let a = a.solid(ctx, settings);
        let b = b.solid(ctx, settings);
        csg_replacement(a, b, ctx)
      }
      ShapeOp::Translate { shape, pos } => {
        let shape = shape.solid(ctx, settings);
        csg_translate(shape, *pos, ctx)
      }
      ShapeOp::Scale { shape, scale } => {
        let shape = shape.solid(ctx, settings);
        csg_scale(shape, *scale, ctx)
      }
      ShapeOp::MatrixTransform {
        shape: _,
        matrix: _,
      } => {
        todo!();
      }

      ShapeOp::Recolor { shape, .. } => shape.solid(ctx, settings),
      ShapeOp::Abbreviate {
        shape,
        threshold_voxel_size,
      } => {
        if settings.min_voxel_size < *threshold_voxel_size {
          shape.solid(ctx, settings)
        } else {
          ctx.constant(1.0)
        }
      }
    }
  }

  fn build(
    &self,
    ctx: &mut Context,
    field: FieldType,
    settings: &BuildSettings,
  ) -> Node {
    match field {
      FieldType::Solid => self.solid(ctx, settings),
      FieldType::Color => {
        match self {
          Self::Union(a, b) => {
            // perform a replacement instead of a union
            let a = a.build(ctx, field, settings);
            let b = b.build(ctx, field, settings);
            csg_replacement(a, b, ctx)
          }
          Self::Difference(..)
          | Self::Intersection(..)
          | Self::Replacement(..)
          | Self::Translate { .. }
          | Self::Scale { .. }
          | Self::MatrixTransform { .. }
          | Self::Abbreviate { .. } => self.bound_solid(ctx, settings),

          Self::Recolor { shape, rgb } => {
            let shape = shape.bound_solid(ctx, settings);
            let color_var = ctx
              .var(&format!("c({},{},{})", rgb[0], rgb[1], rgb[2]))
              .unwrap();
            ctx.min(shape, color_var).unwrap()
          }
        }
      }
      FieldType::Identity => self.bound_solid(ctx, settings),
      FieldType::Semantic => {
        todo!();
      }
    }
  }
}
