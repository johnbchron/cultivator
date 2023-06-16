pub mod primitives;

use super::{op::csg_translate, BuildSettings, FieldType, Shape};

use fidget::context::Node;
use fidget::Context;

/// A translation wrapper.
#[derive(Debug)]
pub struct Translate {
  pub shape: Box<dyn Shape>,
  pub pos: [f32; 3],
}

impl Shape for Translate {
  fn solid(&self, ctx: &mut Context, settings: &BuildSettings) -> Node {
    let shape = self.shape.solid(ctx, settings);
    csg_translate(shape, self.pos, ctx)
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
