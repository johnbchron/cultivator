use crate::shape::{BinaryOp, Shape, ShapeDef, ShapeOp, UnaryOp};

// shape defs
pub fn sphere(radius: f32) -> Shape {
  Shape::ShapeDef(ShapeDef::SpherePrimitive { radius })
}
pub fn box_(x: f32, y: f32, z: f32) -> Shape {
  Shape::ShapeDef(ShapeDef::RectPrismPrimitive { x, y, z })
}
pub fn cube(size: f32) -> Shape {
  Shape::ShapeDef(ShapeDef::CubePrimitive { size })
}

// unary ops
pub fn translate(shape: Shape, x: f32, y: f32, z: f32) -> Shape {
  Shape::ShapeOp(ShapeOp::UnaryOp(
    UnaryOp::Translate { pos: [x, y, z] },
    Box::new(shape),
  ))
}
pub fn scale(shape: Shape, x: f32, y: f32, z: f32) -> Shape {
  Shape::ShapeOp(ShapeOp::UnaryOp(
    UnaryOp::Scale { scale: [x, y, z] },
    Box::new(shape),
  ))
}
pub fn matrix_transform(shape: Shape, matrix: [f32; 16]) -> Shape {
  Shape::ShapeOp(ShapeOp::UnaryOp(
    UnaryOp::MatrixTransform { matrix },
    Box::new(shape),
  ))
}
pub fn recolor(shape: Shape, r: u8, g: u8, b: u8) -> Shape {
  Shape::ShapeOp(ShapeOp::UnaryOp(
    UnaryOp::Recolor { rgb: [r, g, b] },
    Box::new(shape),
  ))
}
pub fn abbreviate(shape: Shape, threshold: f32) -> Shape {
  Shape::ShapeOp(ShapeOp::UnaryOp(
    UnaryOp::Abbreviate { threshold },
    Box::new(shape),
  ))
}

// binary ops
pub fn union(a: Shape, b: Shape) -> Shape {
  Shape::ShapeOp(ShapeOp::BinaryOp(BinaryOp::Union, Box::new(a), Box::new(b)))
}
pub fn difference(a: Shape, b: Shape) -> Shape {
  Shape::ShapeOp(ShapeOp::BinaryOp(
    BinaryOp::Difference,
    Box::new(a),
    Box::new(b),
  ))
}
pub fn intersection(a: Shape, b: Shape) -> Shape {
  Shape::ShapeOp(ShapeOp::BinaryOp(
    BinaryOp::Intersection,
    Box::new(a),
    Box::new(b),
  ))
}
pub fn replacement(a: Shape, b: Shape) -> Shape {
  Shape::ShapeOp(ShapeOp::BinaryOp(
    BinaryOp::Replacement,
    Box::new(a),
    Box::new(b),
  ))
}
