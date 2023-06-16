use super::def::ShapeDef;
use super::{BuildSettings, FieldType};

use std::collections::HashMap;

use fidget::{context::Node, Context};

pub struct Composition {
  shapes: Vec<Box<dyn ShapeDef>>,
  cache: HashMap<(FieldType, BuildSettings), (Node, Context)>,
}

#[allow(dead_code)]
impl Composition {
  pub fn new() -> Self {
    Composition {
      shapes: Vec::new(),
      cache: HashMap::new(),
    }
  }

  pub fn add(&mut self, shape: Box<dyn ShapeDef>) {
    self.shapes.push(shape);
  }

  pub fn remove(&mut self, index: usize) {
    self.shapes.remove(index);
  }

  fn _build(
    &self,
    field: FieldType,
    settings: &BuildSettings,
  ) -> (Node, Context) {
    let mut ctx = Context::new();
    let shapes = &self
      .shapes
      .iter()
      .map(|shape| shape.build(&mut ctx, field, settings))
      .collect::<Vec<Node>>();

    // build a binary tree out of ctx.min() operations
    let mut min_tree = shapes.clone();
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

    (min_tree[0], ctx)
  }

  pub fn cache(
    &mut self,
    field: FieldType,
    settings: &BuildSettings,
  ) -> (Node, Context) {
    let key = (field, settings.clone());
    if let Some((node, ctx)) = self.cache.get(&key) {
      return (node.clone(), ctx.clone());
    }

    let (node, ctx) = self._build(field, settings);
    self.cache.insert(key, (node.clone(), ctx.clone()));
    (node, ctx)
  }
}

impl From<Vec<Box<dyn ShapeDef>>> for Composition {
  fn from(shapes: Vec<Box<dyn ShapeDef>>) -> Self {
    Composition {
      shapes,
      cache: HashMap::new(),
    }
  }
}
