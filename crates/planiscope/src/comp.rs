use fidget::{context::Node, Context};

use crate::{csg::csg_translate, shape::Shape};

type Position = [f32; 3];

#[derive(Debug)]
pub struct CompilationSettings {
  pub min_voxel_size: f32,
}

pub struct Composition {
  shapes: Vec<(Shape, Position)>,
}

impl Default for Composition {
  fn default() -> Self {
    Self::new()
  }
}

impl Composition {
  pub fn new() -> Self {
    Composition { shapes: Vec::new() }
  }

  pub fn add_shape(&mut self, shape: Shape, translation: Position) {
    self.shapes.push((shape, translation));
  }

  pub fn compile_solid(
    &self,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    // compile a translated Node for each Shape
    let shapes = &self
      .shapes
      .iter()
      .map(|(shape, pos)| {
        let a = shape.compile_solid(ctx, settings);
        csg_translate(a, *pos, ctx)
      })
      .collect::<Vec<Node>>();

    binary_shape_tree(shapes.to_vec(), ctx, BinaryShapeTreeCombinator::Min)
  }

  pub fn compile_color(
    &self,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    // compile a translated Node for each Shape
    let shapes = &self
      .shapes
      .iter()
      .map(|(shape, pos)| {
        let a = shape.compile_color(ctx, settings);
        csg_translate(a, *pos, ctx)
      })
      .collect::<Vec<Node>>();

    binary_shape_tree(shapes.to_vec(), ctx, BinaryShapeTreeCombinator::Add)
  }
}

enum BinaryShapeTreeCombinator {
  Min,
  Add,
}

fn binary_shape_tree(
  nodes: Vec<Node>,
  ctx: &mut Context,
  combinator: BinaryShapeTreeCombinator,
) -> Node {
  let mut min_tree = nodes;
  while min_tree.len() > 1 {
    let mut new_tree = Vec::new();
    for i in (0..min_tree.len()).step_by(2) {
      let a = &min_tree[i];
      let b = if i + 1 < min_tree.len() {
        &min_tree[i + 1]
      } else {
        a
      };
      let node = match combinator {
        BinaryShapeTreeCombinator::Min => ctx.min(*a, *b).unwrap(),
        BinaryShapeTreeCombinator::Add => ctx.add(*a, *b).unwrap(),
      };
      new_tree.push(node);
    }
    min_tree = new_tree;
  }

  min_tree[0]
}
