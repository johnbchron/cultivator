
use crate::comp::RenderSettings;
use crate::csg::*;

use fidget::{context::Node, Context};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum FieldType {
  Solid,
  Color,
  Identity,
  Semantic,
}

#[derive(Debug)]
pub enum Shape {
  ShapeDef(ShapeDef),
  ShapeOp(ShapeOp),
}

impl Shape {
  pub fn compile_solid(&self, ctx: &mut Context, settings: &RenderSettings) -> Node {
    match self {
      Shape::ShapeDef(shape_def) => { shape_def.compile_solid(ctx, settings) }
      Shape::ShapeOp(shape_op) => { shape_op.compile_solid(ctx, settings) }
    }
  }
  pub fn compile_clamped_solid(&self, ctx: &mut Context, settings: &RenderSettings) -> Node {
    let shape = self.compile_solid(ctx, settings);
    csg_clamp(shape, ctx)
  }
  pub fn compile_identity(&self, ctx: &mut Context, settings: &RenderSettings) -> Node {
    let shape = self.compile_clamped_solid(ctx, settings);
    let zero = ctx.constant(0.0);
    ctx.max(shape, zero).unwrap()
  }
  pub fn compile_color(&self, ctx: &mut Context, settings: &RenderSettings) -> Node {
    match self {
      Shape::ShapeDef(shape_def) => { shape_def.compile_color(ctx, settings) }
      Shape::ShapeOp(shape_op) => { shape_op.compile_color(ctx, settings) }
    }
  }
}

#[derive(Clone, Debug)]
pub enum ShapeDef {
	SpherePrimitive { radius: f32 },
}

impl ShapeDef {
  pub fn compile_solid(&self, ctx: &mut Context, settings: &RenderSettings) -> Node {
    match self {
      Self::SpherePrimitive { radius } => {
        if *radius * 2.0 < settings.min_voxel_size {
          return ctx.constant(1.0.into());
        }

        let r = ctx.constant((*radius).into());
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
    }
  }
  pub fn compile_color(&self, ctx: &mut Context, settings: &RenderSettings) -> Node {
  	match self {
  		_ => {
  			let shape = self.compile_solid(ctx, settings);
  			let shape = csg_clamp(shape, ctx);
  			csg_color(shape, [255, 255, 255], ctx)
  		}
  	}
  }
}

#[derive(Debug)]
pub enum ShapeOp {
  UnaryOp(UnaryOp, Box<Shape>),
  BinaryOp(BinaryOp, Box<Shape>, Box<Shape>),
}

impl ShapeOp {
  pub fn compile_solid(&self, ctx: &mut Context, settings: &RenderSettings) -> Node {
    match self {
      ShapeOp::UnaryOp(unary_op, a ) => { unary_op.compile_solid(a.as_ref(), ctx, settings) }
      ShapeOp::BinaryOp(binary_op, a, b ) => { binary_op.compile_solid(a, b, ctx, settings) }
    }
  }
  pub fn compile_color(&self, ctx: &mut Context, settings: &RenderSettings) -> Node {
		match self {
			ShapeOp::UnaryOp(unary_op, a ) => { unary_op.compile_color(a.as_ref(), ctx, settings) }
			ShapeOp::BinaryOp(binary_op, a, b ) => { binary_op.compile_color(a, b, ctx, settings) }
		}
	}
}

#[derive(Debug)]
pub enum UnaryOp {
	Translate { pos: [f32; 3] },
  Scale { scale: [f32; 3] },
  MatrixTransform { matrix: [f32; 16] },
  Recolor { rgb: [u8; 3] },
  Abbreviate { threshold: f32 },
}

impl UnaryOp {
  pub fn compile_solid(&self, a: &Shape, ctx: &mut Context, settings: &RenderSettings) -> Node {
    match self {
      UnaryOp::Translate { pos } => {
        let shape = a.compile_solid(ctx, settings);
        csg_translate(shape, *pos, ctx)
      }
      UnaryOp::Scale { scale } => {
        let shape = a.compile_solid(ctx, settings);
        csg_scale(shape, *scale, ctx)
      }
      UnaryOp::MatrixTransform { matrix: _ } => {
        todo!();
      }
      UnaryOp::Recolor { .. } => a.compile_solid(ctx, settings),
      UnaryOp::Abbreviate { threshold } => {
        if settings.min_voxel_size < *threshold {
          a.compile_solid(ctx, settings)
        } else {
          ctx.constant(1.0)
        }
      }
    }
  }
  
  pub fn compile_color(&self, a: &Shape, ctx: &mut Context, settings: &RenderSettings) -> Node {
		match self {
			UnaryOp::Recolor { rgb } => {
				let shape = a.compile_solid(ctx, settings);
				let shape = csg_clamp(shape, ctx);
				csg_color(shape, *rgb, ctx)
			}
			_ => {
				a.compile_color(ctx, settings)
			}
		}
	}
}

#[derive(Debug)]
pub enum BinaryOp {
  Union,
  Difference,
  Intersection,
  Replacement,
}

impl BinaryOp {
  pub fn compile_solid(&self, a: &Shape, b: &Shape, ctx: &mut Context, settings: &RenderSettings) -> Node {
    match self {
      BinaryOp::Union => {
        let a = a.compile_solid(ctx, settings);
        let b = b.compile_solid(ctx, settings);
        csg_union(a, b, ctx)
      }
      BinaryOp::Difference => {
        let a = a.compile_solid(ctx, settings);
        let b = b.compile_solid(ctx, settings);
        csg_difference(a, b, ctx)
      }
      BinaryOp::Intersection => {
        let a = a.compile_solid(ctx, settings);
        let b = b.compile_solid(ctx, settings);
        csg_intersection(a, b, ctx)
      }
      BinaryOp::Replacement => {
        let a = a.compile_solid(ctx, settings);
        let b = b.compile_solid(ctx, settings);
        csg_replacement(a, b, ctx)
      }
    }
  }

  pub fn compile_color(&self, a: &Shape, b: &Shape, ctx: &mut Context, settings: &RenderSettings) -> Node {
  	match self {
  	  BinaryOp::Union => {
  	  	// copy from replacement
  	  	BinaryOp::Replacement.compile_color(a, b, ctx, settings)
  	  }
  	  BinaryOp::Difference => {
  	  	let solid = self.compile_solid(a, b, ctx, settings);
  	  	let color = a.compile_color(ctx, settings);
  	  	let shape = csg_clamp(solid, ctx);
  	  	ctx.mul(color, shape).unwrap()
  	  }
  	  BinaryOp::Intersection => {
		  	let solid = self.compile_solid(a, b, ctx, settings);
		  	csg_color(solid, [255, 255, 255], ctx)
		  }
		  BinaryOp::Replacement => {
  	  	let (a_shape, b_shape) = (a, b);
  	  	let a = a_shape.compile_solid(ctx, settings);
  	  	let b = b_shape.compile_solid(ctx, settings);
  	  	let b = csg_difference(b, a, ctx);
  	  	let b = csg_clamp(b, ctx);
  	  	let a_color = a_shape.compile_color(ctx, settings);
  	  	let b_color = b_shape.compile_color(ctx, settings);
  	  	let b_color = ctx.mul(b_color, b).unwrap();
  	  	ctx.add(a_color, b_color).unwrap()
		  }
  	}
  }
}