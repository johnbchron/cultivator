use rhai::{Dynamic, Engine, Scope};
use anyhow::{Result, Error};

use crate::{builder, shape::Shape};

#[derive(Clone)]
pub struct ShapeWithTranslate(Shape, [f32; 3]);

pub fn attach_translate(
  shape: Shape,
  translate: Vec<Dynamic>,
) -> ShapeWithTranslate {
  let mut pos = [0.0; 3];
  for (i, val) in translate.into_iter().enumerate() {
    pos[i] = val.as_float().unwrap();
  }
  ShapeWithTranslate(shape, pos)
}

pub fn eval(
  code: &str,
) -> Result<Vec<(Shape, [f32; 3])>> {
  let mut engine = Engine::new();

  engine.register_type::<Shape>();
  engine.register_fn("sphere", builder::sphere);
  engine.register_fn("translate", builder::translate);
  engine.register_fn("scale", builder::scale);
  engine.register_fn("matrix_transform", builder::matrix_transform);
  engine.register_fn("recolor", |shape: Shape, r: i64, g: i64, b: i64| {
    builder::recolor(
      shape,
      (r % 256).try_into().unwrap(),
      (g % 256).try_into().unwrap(),
      (b % 256).try_into().unwrap(),
    )
  });
  engine.register_fn("abbreviate", builder::abbreviate);
  engine.register_fn("union", builder::union);
  engine.register_fn("difference", builder::difference);
  engine.register_fn("intersection", builder::intersection);
  engine.register_fn("replacement", builder::replacement);
  engine.register_fn("shape", attach_translate);

  let ast = engine.compile(code)?;
  let mut scope = Scope::new();
  let shape_list =
    engine.eval_ast_with_scope::<Vec<Dynamic>>(&mut scope, &ast).map_err(|e| {
			Error::msg(format!("failed to eval code: {}", e))
		})?;

  let mut shapes = Vec::new();
  for shape in shape_list {
    let shape_with_translate = shape
      .try_cast::<ShapeWithTranslate>()
      .ok_or(Error::msg("failed to cast array contents to shape"))?;
    shapes.push((shape_with_translate.0, shape_with_translate.1));
  }

  Ok(shapes)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::builder::sphere;

  #[test]
  fn test_eval() {
    let shape = eval("[shape(sphere(1.0), [0.0, 0.0, 0.0])]").unwrap();
    assert_eq!(shape, vec![(sphere(1.0), [0.0, 0.0, 0.0])]);
  }
}
