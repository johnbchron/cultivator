pub mod comp;
pub mod def;
pub mod op;
pub mod stdlib;

use core::fmt::Debug;

use dyn_clone::DynClone;
use fidget::{context::Node, Context};

/// All the field types that can be calculated for a `Shape`.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum FieldType {
  /// The implicit surface of the shape. The shape is defined as the set of
  /// points in space where this field is zero. The field is positive outside
  /// the shape and negative inside.
  Solid,
  /// The color field of the shape.
  Color,
  /// The identity field of the shape. This field describes the ID of the shape,
  /// for use in operational collisions. The field is one inside the shape and
  /// zero outside.
  Identity,
  /// Describes the semantic ID of the shape, for use in semantic collisions.
  /// The field is the identity field, but multiplied by a float generated from
  /// a UUID assigned to the `ShapeDef` that generated the shape.
  Semantic,
}

/// Settings for building a `Shape`.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BuildSettings {
  pub min_voxel_size: fixed::types::I16F16,
}

pub trait Shape: Debug + Sized + Send + Sync {
  fn solid(&self, ctx: &mut Context, settings: &BuildSettings) -> Node;
  fn bound_solid(&self, ctx: &mut Context, settings: &BuildSettings) -> Node {
    let shape = self.solid(ctx, settings);
    let steep_slope = ctx.constant(1000.0);
    let steep_shape = ctx.mul(shape, steep_slope).unwrap();
    let one = ctx.constant(1.0);
    let neg_one = ctx.constant(-1.0);
    let upper_bound = ctx.min(steep_shape, one).unwrap();
    let lower_bound = ctx.max(upper_bound, neg_one).unwrap();
    return lower_bound;
  }

  fn build(
    &self,
    ctx: &mut Context,
    field: FieldType,
    settings: &BuildSettings,
  ) -> Node;
}
