
use crate::shape::Shape;
use crate::csg::csg_translate;

use fidget::{context::Node, Context};

type Position = [f32; 3];

#[derive(Debug)]
pub struct RenderSettings {
  pub min_voxel_size: f32,
}

pub struct Composition {
  shapes: Vec<(Shape, Position)>,
}

impl Composition {
  pub fn new() -> Self {
    Composition { shapes: Vec::new() }
  }

  pub fn add_shape(&mut self, shape: Shape, translation: Position) {
    self.shapes.push((shape, translation));
  }
  
  pub fn compile_solid(&self, ctx: &mut Context, settings: &RenderSettings) -> Node {
    // compile a translated Node for each Shape
    let shapes = &self
      .shapes
      .iter()
      .map(|(shape, pos) | {
        let a = shape.compile_solid(ctx, settings);
        csg_translate(a, *pos, ctx)
      })
      .collect::<Vec<Node>>();

    binary_shape_tree(shapes.to_vec(), ctx)
  }
  
  pub fn compile_color(&self, ctx: &mut Context, settings: &RenderSettings) -> Node {
    // compile a translated Node for each Shape
    let shapes = &self
      .shapes
      .iter()
      .map(|(shape, pos) | {
        let a = shape.compile_color(ctx, settings);
        csg_translate(a, *pos, ctx)
      })
      .collect::<Vec<Node>>();

    binary_shape_tree(shapes.to_vec(), ctx)
  }
}

fn binary_shape_tree(nodes: Vec<Node>, ctx: &mut Context) -> Node {
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
      let node = ctx.min(a.clone(), b.clone()).unwrap();
      new_tree.push(node);
    }
    min_tree = new_tree;
  }

  min_tree[0]
}